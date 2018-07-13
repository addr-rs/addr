extern crate psl;
#[macro_use]
extern crate criterion;

use psl::{Psl, List};
use criterion::Criterion;

fn criterion_benchmark(c: &mut Criterion) {
    let list = List::new();

    c.bench_function("public suffix", move |b| {
        b.iter(|| { list.suffix("example.gb.com").unwrap(); } )
    });

    c.bench_function("registrable domain", move |b| {
        b.iter(|| { list.domain("example.dyndns-server.com").unwrap(); } )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
