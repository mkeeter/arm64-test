use arm64_test::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let slice = (0..1024).map(|i| i as f32).collect::<Vec<f32>>();
    c.bench_function("sum_slice", |b| b.iter(|| sum_slice(black_box(&slice))));
    c.bench_function("sum_ptr", |b| {
        b.iter(|| unsafe {
            sum_ptr(black_box(slice.as_ptr()), black_box(slice.len()))
        })
    });
    c.bench_function("sum_ptr_asm", |b| {
        b.iter(|| unsafe {
            sum_ptr_asm(black_box(slice.as_ptr()), black_box(slice.len()))
        })
    });
    c.bench_function("sum_ptr_asm2", |b| {
        b.iter(|| unsafe {
            sum_ptr_asm2(black_box(slice.as_ptr()), black_box(slice.len()))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
