use addr::{DnsName, DomainName};
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

    c.bench_function("addr::DomainName::parse", |b| {
        b.iter(|| {
            DomainName::parse("example.com").unwrap();
        })
    });

    c.bench_function("addr::DnsName::parse", |b| {
        b.iter(|| {
            DnsName::parse("_example.com").unwrap();
        })
    });
}

criterion_group!(benches, psl);
criterion_main!(benches);
