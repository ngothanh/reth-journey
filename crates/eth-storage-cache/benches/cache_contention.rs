use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use eth_primitives::{Address, FixedBytes};
use eth_storage_cache::{
    Account, MutexCache, NoOpEviction, RwLockCache, ShardedCache,
};
use std::sync::Arc;
use std::thread;

const OPS_PER_THREAD: usize = 10_000;

/// Spawn `threads` workers; each runs `op(cache, i)` `OPS_PER_THREAD` times.
fn run_concurrent<C, F>(cache: Arc<C>, threads: usize, op: F)
where
    C: Send + Sync + 'static,
    F: Fn(&C, usize) + Copy + Send + 'static,
{
    let handles: Vec<_> = (0..threads)
        .map(|t| {
            let c = Arc::clone(&cache);
            thread::spawn(move || {
                for i in 0..OPS_PER_THREAD {
                    op(&c, t * OPS_PER_THREAD + i);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}

/// Build an address whose first byte is `i % 256` so writes scatter across shards.
fn addr_for(i: usize) -> Address {
    let mut bytes = [0u8; 20];
    bytes[0] = (i % 256) as u8;
    bytes[1] = (i >> 8) as u8;
    bytes[2] = (i >> 16) as u8;
    FixedBytes(bytes)
}

fn bench_concurrent_inserts(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_inserts");

    for &threads in &[1, 2, 4, 8] {
        group.bench_with_input(BenchmarkId::new("Mutex", threads), &threads, |b, &t| {
            b.iter(|| {
                let cache = Arc::new(MutexCache::new());
                run_concurrent(cache, t, |c, i| {
                    c.insert_account(addr_for(i), Account::default());
                });
            });
        });

        group.bench_with_input(BenchmarkId::new("RwLock", threads), &threads, |b, &t| {
            b.iter(|| {
                let cache = Arc::new(RwLockCache::new());
                run_concurrent(cache, t, |c, i| {
                    c.insert_account(addr_for(i), Account::default());
                });
            });
        });

        group.bench_with_input(BenchmarkId::new("Sharded16", threads), &threads, |b, &t| {
            b.iter(|| {
                let cache: Arc<ShardedCache<16, NoOpEviction>> =
                    Arc::new(ShardedCache::default());
                run_concurrent(cache, t, |c, i| {
                    c.insert(addr_for(i), Account::default());
                });
            });
        });

        group.bench_with_input(BenchmarkId::new("Sharded64", threads), &threads, |b, &t| {
            b.iter(|| {
                let cache: Arc<ShardedCache<64, NoOpEviction>> =
                    Arc::new(ShardedCache::default());
                run_concurrent(cache, t, |c, i| {
                    c.insert(addr_for(i), Account::default());
                });
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_concurrent_inserts);
criterion_main!(benches);
