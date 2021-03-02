use addr::DomainName;
use criterion::{criterion_group, criterion_main, Criterion};
use psl::{List, Psl};

fn psl(c: &mut Criterion) {
    c.bench_function("psl", |b| {
        b.iter(|| {
            List.domain("example.com").unwrap();
        })
    });

    c.bench_function("addr", |b| {
        b.iter(|| {
            "example.com".parse::<DomainName>().unwrap();
        })
    });
}

criterion_group!(benches, psl);
criterion_main!(benches);
