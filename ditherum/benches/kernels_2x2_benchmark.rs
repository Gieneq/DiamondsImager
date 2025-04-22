use std::{hint::black_box, time::Duration};
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

use ditherum::algorithms::kernel;

fn kernel_2x2_benchmarking_gen_data() -> Vec<Vec<f32>> {
    let (width, height) = (1200, 800);
    vec![vec![black_box(127f32); width]; height]
}

#[inline(never)]
pub fn example_kernel2x2_dummy_float(kernel: kernel::MutKernel2x2<f32>) {
    let delta = black_box(*kernel.tl * 0.5 + *kernel.tr * 0.3 + *kernel.bl * 0.2 + *kernel.br * 0.1);
    *kernel.tl += delta;
    *kernel.tr -= delta;
    *kernel.bl += delta;
    *kernel.br -= delta;
}

fn linkedlist_push_back_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Kernel2x2_comparison");
    let loops = 1;
    let mut matrix = kernel_2x2_benchmarking_gen_data();


    group.bench_with_input(BenchmarkId::new("Kernel Unsafe Dummy", loops), &loops, |b, &_loops| {
        b.iter(|| {
            kernel::apply_2x2_kernel_processing(&mut matrix, example_kernel2x2_dummy_float);
        });
    });
}

fn configure_criterion() -> Criterion {
    Criterion::default()
    .warm_up_time(Duration::new(3, 0))
    .measurement_time(Duration::new(10, 0))
    .sample_size(100)
}

criterion_group!(
    name = benches;
    config = configure_criterion();
    targets = linkedlist_push_back_benchmark
);
criterion_main!(benches);