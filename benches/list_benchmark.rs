extern crate psl;
#[macro_use]
extern crate criterion;

use criterion::Criterion;
use psl::{List, Psl};

fn psl(c: &mut Criterion) {
    let list = List::new();

    c.bench_function("public suffix", move |b| {
        b.iter(|| {
            list.suffix("example.com").unwrap();
        })
    });

    c.bench_function("registrable domain", move |b| {
        b.iter(|| {
            list.domain("example.com").unwrap();
        })
    });
}

criterion_group!(benches, psl);
criterion_main!(benches);
