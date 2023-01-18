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
pub unsafe extern "C" fn sum_ptr_asm(f: *const f32, n: usize) -> f32 {
    asm!(
        "
        stp   x29, x30, [sp, #-16]!

        fmov s0, #0.0

        // Jump into setup
        b 3f

        2: // Jump into the body of the loop, setting the link register
            bl 4f

        3: // Setup: update pointers and break if we hit zero
            cmp x1, #0
            b.eq 5f // Jump to function exit
            sub x1, x1, #1
            ldr s1, [x0], #4
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
pub unsafe extern "C" fn sum_ptr_asm2(f: *const f32, n: usize) -> f32 {
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_sums() {
        let d = (0..1024).map(|i| i as f32).collect::<Vec<f32>>();
        let v = sum_slice(&d);
        assert_eq!(v, unsafe { sum_ptr(d.as_ptr(), d.len()) });
        assert_eq!(v, unsafe { sum_ptr_asm(d.as_ptr(), d.len()) });
        assert_eq!(v, unsafe { sum_ptr_asm2(d.as_ptr(), d.len()) });
    }
}
