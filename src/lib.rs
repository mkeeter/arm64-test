#![feature(naked_functions)]

use std::arch::asm;

pub fn sum_slice(f: &[f32]) -> f32 {
    f.iter().sum()
}

pub unsafe fn sum_ptr(f: *const f32, n: usize) -> f32 {
    let mut out = 0.0;
    for i in 0..n {
        out += unsafe { *f.add(i) };
    }
    out
}

#[naked]
pub unsafe extern "C" fn sum_ptr_asm_matched(f: *const f32, n: usize) -> f32 {
    asm!(
        "
        stp   x29, x30, [sp, #-16]!
        fmov s0, #0.0

        // Setup
        2:
            cmp x1, #0
            b.eq 5f // Jump to function exit
            sub x1, x1, #1
            ldr s1, [x0], #4

            bl 4f
            b 2b

        4: // Body of the loop: do one addition, then jump back to setup
            fadd s0, s0, s1
            // In practice, there's an arbitrary amount of code here, which
            // is why we use `bl` / `ret` instead of a fixed-size jump
            ret

        5: // Function exit
            ldp   x29, x30, [sp], #16
            ret
    ",
        options(noreturn)
    );
}

#[naked]
pub unsafe extern "C" fn sum_ptr_asm_mismatched(
    f: *const f32,
    n: usize,
) -> f32 {
    asm!(
        "
        stp   x29, x30, [sp, #-16]!
        fmov s0, #0.0

        bl 3f // set link register to the next instruction

        3:
            cmp x1, #0
            b.eq 5f // Jump to function exit
            sub x1, x1, #1
            ldr s1, [x0], #4

            fadd s0, s0, s1
            // In practice, there's an arbitrary amount of code here, which
            // is why we use `bl` / `ret` instead of a fixed-size jump

            ret // keep looping, back to 3:

        5: // function exit
            ldp   x29, x30, [sp], #16
            ret
    ",
        options(noreturn)
    );
}

#[naked]
pub unsafe extern "C" fn sum_ptr_asm_mismatched_br(
    f: *const f32,
    n: usize,
) -> f32 {
    asm!(
        "
        stp   x29, x30, [sp, #-16]!
        fmov s0, #0.0

        bl 3f // set link register to the next instruction

        3:
            cmp x1, #0
            b.eq 5f // Jump to function exit
            sub x1, x1, #1
            ldr s1, [x0], #4

            fadd s0, s0, s1
            // In practice, there's an arbitrary amount of code here, which
            // is why we use `bl` / `ret` instead of a fixed-size jump

            br x30

        5: // function exit
            ldp   x29, x30, [sp], #16
            ret
    ",
        options(noreturn)
    );
}

#[naked]
pub unsafe extern "C" fn sum_ptr_asm_branch(f: *const f32, n: usize) -> f32 {
    asm!(
        "
        stp   x29, x30, [sp, #-16]!
        fmov s0, #0.0

        3:
            subs x1, x1, #1
            b.mi 5f
            ldr s1, [x0], #4

            fadd s0, s0, s1
            b 3b

        5: // function exit
            ldp   x29, x30, [sp], #16
            ret
    ",
        options(noreturn)
    );
}

#[naked]
pub unsafe extern "C" fn sum_ptr_asm_simd(f: *const f32, n: usize) -> f32 {
    asm!(
        "
        stp   x29, x30, [sp, #-16]!

        fmov s0, #0.0
        dup v1.4s, v0.s[0]
        dup v2.4s, v0.s[0]

        2:  // 1x per loop
            ands xzr, x1, #3
            b.eq 3f

            sub x1, x1, #1
            ldr s3, [x0], #4

            fadd s0, s0, s3
            b 2b

        3:  // 4x SIMD per loop
            ands xzr, x1, #7
            b.eq 4f

            sub x1, x1, #4
            ldp d3, d4, [x0], #16
            mov v3.d[1], v4.d[0]

            fadd v1.4s, v1.4s, v3.4s

            b 3b

        4:  // 2 x 4x SIMD per loop
            cmp x1, #0
            b.eq 5f // Jump to function exit

            sub x1, x1, #8

            ldp d3, d4, [x0], #16
            mov v3.d[1], v4.d[0]
            fadd v1.4s, v1.4s, v3.4s

            ldp d5, d6, [x0], #16
            mov v5.d[1], v6.d[0]
            fadd v2.4s, v2.4s, v5.4s

            b 4b

        5: // function exit
            fadd v2.4s, v2.4s, v1.4s
            mov s1, v2.s[0]
            fadd s0, s0, s1
            mov s1, v2.s[1]
            fadd s0, s0, s1
            mov s1, v2.s[2]
            fadd s0, s0, s1
            mov s1, v2.s[3]
            fadd s0, s0, s1

            ldp   x29, x30, [sp], #16
            ret
    ",
        options(noreturn)
    );
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_sums() {
        for size in [0, 1, 2, 3, 1024, 1025] {
            let d = (0..size).map(|i| i as f32).collect::<Vec<f32>>();
            let v = sum_slice(&d);
            let ptr = d.as_ptr();
            let len = d.len();
            assert_eq!(v, unsafe { sum_ptr(ptr, len) });
            assert_eq!(v, unsafe { sum_ptr_asm_matched(ptr, len) });
            assert_eq!(v, unsafe { sum_ptr_asm_mismatched(ptr, len) });
            assert_eq!(v, unsafe { sum_ptr_asm_branch(ptr, len) });
            assert_eq!(v, unsafe { sum_ptr_asm_simd(ptr, len) });
        }
    }
}
