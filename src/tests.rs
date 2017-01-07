extern crate rspec;
extern crate slog_term;

use std::thread;
use std::time::Duration;

use {init, get, set_logger};
use publicsuffix::LIST_URL;
use slog::{Logger, DrainExt};
use self::rspec::context::rdescribe;

macro_rules! pass {
    () => { Ok(()) as Result<(), ()> }
}

#[test]
fn cache_behaviour() {
    rdescribe("initial cache", |ctx| {
        ctx.it("should start off with an empty list", || {
            assert!(get().all().is_empty());
            pass!()
        });
    });

    rdescribe("initialised cache", |ctx| {
        init(LIST_URL, Duration::from_secs(10)).unwrap();

        ctx.it("should have ICANN domains", || {
            assert!(!get().icann().is_empty());
            pass!()
        });

        ctx.it("should have private domains", || {
            assert!(!get().private().is_empty());
            pass!()
        });

        ctx.it("should download a new list at the given interval", || {
            let log = Logger::root(slog_term::streamer().build().fuse(), o!("test" => "updating"));
            set_logger(&log);
            thread::sleep(Duration::from_secs(60));
            assert!(!get().all().is_empty());
            pass!()
        });
    });
}
