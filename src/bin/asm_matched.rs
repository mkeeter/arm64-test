use arm64_test::sum_ptr_asm_matched;

fn main() {
    let n: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let d = (0..n).map(|i| i as f32).collect::<Vec<f32>>();
    println!("{}", unsafe { sum_ptr_asm_matched(d.as_ptr(), d.len()) });
}
