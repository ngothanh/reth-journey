use concurrent::Semaphore;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::future::Future;
use std::hint::black_box;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

fn grant_one_waiter(c: &mut Criterion) {
    let semaphore = Semaphore::new(0);
    c.bench_function("bench_semaphore_grant_one_waiter", |b| {
        b.iter_batched(
            || {
                while let Ok(p) = semaphore.try_acquire() {
                    std::mem::forget(p);
                }
                let mut fut = Box::pin(semaphore.acquire());
                let mut ctx = Context::from_waker(Waker::noop());
                assert!(fut.as_mut().poll(&mut ctx).is_pending());
                fut
            },
            |fut| {
                semaphore.add_permits(1);
                fut
            },
            BatchSize::SmallInput,
        )
    });
}

fn counter_round_trip(c: &mut Criterion) {
    let semaphore = Semaphore::new(1);
    c.bench_function("bench_semaphore_counter_round_trip", |b| {
        b.iter(|| {
            let permit = semaphore.try_acquire().unwrap();
            drop(black_box(permit));
        });
    });
}
fn poll_fast_path(c: &mut Criterion) {
    let semaphore = Semaphore::new(1);
    let mut ctx = Context::from_waker(Waker::noop());
    c.bench_function("bench_semaphore_poll_fast_path", |b| {
        b.iter(|| {
            let acquire = pin!(semaphore.acquire());
            match acquire.poll(&mut ctx) {
                Poll::Ready(p) => {
                    drop(black_box(p));
                }
                Poll::Pending => {}
            }
        })
    });
}

criterion_group!(
    benches,
    counter_round_trip,
    poll_fast_path,
    grant_one_waiter
);

criterion_main!(benches);
