use criterion::{criterion_group, criterion_main, Criterion};

fn psl(c: &mut Criterion) {
    c.bench_function("psl::suffix", |b| {
        use psl::{Psl, List};

        b.iter(|| {
            List.suffix(b"example.com").unwrap();
        })
    });

    c.bench_function("psl::domain", |b| {
        use psl::{Psl, List};

        b.iter(|| {
            List.domain(b"example.com").unwrap();
        })
    });

    c.bench_function("addr::parser::DomainName", |b| {
        use addr::parser::DomainName;
        use psl::List;

        b.iter(|| {
            List.parse_domain_name("example.com").unwrap();
        })
    });

    c.bench_function("addr::parser::DnsName", |b| {
        use addr::parser::DnsName;
        use psl::List;

        b.iter(|| {
            List.parse_dns_name("_example.com").unwrap();
        })
    });

    c.bench_function("addr::email::Address::parse", |b| {
        use addr::parser::EmailAddress;
        use psl::List;

        b.iter(|| {
            List.parse_email_address("john.doe@example.com").unwrap();
        })
    });
}

criterion_group!(benches, psl);
criterion_main!(benches);
