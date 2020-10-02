
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn benchmarks(c: &mut Criterion) {
    let db = Database::open("/tmp/rocksdb_benchmark");
    db.insert(b"existingkey", 0u32, 1000u64, b"1234567890");

    c.bench_function("get_not_existing", |b| b.iter(|| {
        db.get(black_box(vec![b"myKey"]), false);
    }));
    c.bench_function("get_existing", |b| b.iter(|| {
        db.get(black_box(vec![b"existingkey"]), false);
    }));
    c.bench_function("set_existing", |b| b.iter(|| {
        db.insert(b"newkey", 0u32, 1000u64, b"1234567890");
    }));
}

use rockscached_db::db::Database;



criterion_group!(benches, benchmarks);
criterion_main!(benches);