extern crate rspec;
extern crate slog;
extern crate slog_term;
extern crate slog_async;

use std::thread;
use std::time::Duration;

use {init, get};
use publicsuffix::LIST_URL;
use slog::{Logger, Drain};
use self::rspec::context::rdescribe;

#[test]
fn cache_behaviour() {
    rdescribe("initial cache", |ctx| {
        ctx.it("should start off with an empty list", || {
            assert!(get().all().is_empty());
        });
    });

    rdescribe("initialised cache", |ctx| {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let log = Logger::root(drain, o!("test" => "updating"));
        init(LIST_URL, Duration::from_secs(20), log).unwrap();

        ctx.it("should have ICANN domains", || {
            assert!(!get().icann().is_empty());
        });

        ctx.it("should have private domains", || {
            assert!(!get().private().is_empty());
        });

        ctx.it("should download a new list at the given interval", || {
            thread::sleep(Duration::from_secs(30));
            assert!(!get().all().is_empty());
        });
    });
}
