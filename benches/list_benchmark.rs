extern crate psl;
extern crate psl_compiled;
#[macro_use]
extern crate criterion;

use psl::Psl;
use psl_compiled::List;
use criterion::Criterion;

fn criterion_benchmark(c: &mut Criterion) {
    let list = List::new();

    c.bench_function(".com domain", move |b| {
        b.iter(|| { list.registrable_domain(&"eXample.cOm".to_lowercase()).unwrap(); } )
    });

    c.bench_function(".co.uk domain", move |b| {
        b.iter(|| { list.registrable_domain(&"eXample.cO.uk".to_lowercase()).unwrap(); } )
    });

    c.bench_function(".co.zw domain", move |b| {
        b.iter(|| { list.registrable_domain(&"eXample.cO.zw".to_lowercase()).unwrap(); } )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
