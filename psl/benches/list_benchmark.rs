extern crate psl;
#[macro_use]
extern crate criterion;

use psl::{Psl, List};
use criterion::Criterion;

fn criterion_benchmark(c: &mut Criterion) {
    let list = List::new();

    c.bench_function("first rule", move |b| {
        b.iter(|| { list.registrable_domain(&"eXample.gb.com".to_lowercase()).unwrap(); } )
    });

    c.bench_function("deeply nested", move |b| {
        b.iter(|| { list.registrable_domain(&"eXample.dyndns-server.com".to_lowercase()).unwrap(); } )
    });

    c.bench_function("popular rule", move |b| {
        b.iter(|| { list.registrable_domain(&"eXample.com".to_lowercase()).unwrap(); } )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
