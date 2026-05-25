use concurrent::Backoff;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::hint::black_box;

fn bench_spin_step_zero(c: &mut Criterion) {
    c.bench_function("backoff/spin/step=0 (1 PAUSE)", |b| {
        b.iter_batched(
            Backoff::new,
            |backoff| black_box(&backoff).spin(),
            BatchSize::SmallInput,
        );
    });
}

fn bench_spin_step_saturated(c: &mut Criterion) {
    c.bench_function("backoff/spin/step=SPIN_LIMIT (64 PAUSEs)", |b| {
        b.iter_batched(
            || {
                let backoff = Backoff::new();
                for _ in 0..7 {
                    backoff.spin();
                }
                backoff
            },
            |backoff| black_box(&backoff).spin(),
            BatchSize::SmallInput,
        );
    });
}

fn bench_snooze_yield_branch(c: &mut Criterion) {
    c.bench_function("backoff/snooze/yield branch (step>SPIN_LIMIT)", |b| {
        b.iter_batched(
            || {
                let backoff = Backoff::new();
                for _ in 0..7 {
                    backoff.snooze();
                }
                backoff
            },
            |backoff| black_box(&backoff).snooze(),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_spin_step_zero,
    bench_spin_step_saturated,
    bench_snooze_yield_branch
);
criterion_main!(benches);
