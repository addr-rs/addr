use addr::domain::Name;
use criterion::{criterion_group, criterion_main, Criterion};

fn psl(c: &mut Criterion) {
    c.bench_function("psl", |b| {
        b.iter(|| {
            psl::domain(b"example.com").unwrap();
        })
    });

    c.bench_function("addr", |b| {
        b.iter(|| {
            Name::parse("example.com").unwrap();
        })
    });
}

criterion_group!(benches, psl);
criterion_main!(benches);
