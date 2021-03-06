use addr::{dns, domain, email};
use criterion::{criterion_group, criterion_main, Criterion};

fn psl(c: &mut Criterion) {
    c.bench_function("psl::suffix", |b| {
        b.iter(|| {
            psl::suffix(b"example.com").unwrap();
        })
    });

    c.bench_function("psl::domain", |b| {
        b.iter(|| {
            psl::domain(b"example.com").unwrap();
        })
    });

    c.bench_function("addr::domain::Name::parse", |b| {
        b.iter(|| {
            domain::Name::parse("example.com").unwrap();
        })
    });

    c.bench_function("addr::dns::Name::parse", |b| {
        b.iter(|| {
            dns::Name::parse("_example.com").unwrap();
        })
    });

    c.bench_function("addr::email::Address::parse", |b| {
        b.iter(|| {
            email::Address::parse("john.doe@example.com").unwrap();
        })
    });
}

criterion_group!(benches, psl);
criterion_main!(benches);
