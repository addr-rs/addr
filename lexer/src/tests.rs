extern crate rspec;

use List;
use self::rspec::context::rdescribe;

#[test]
fn list_behaviour() {
    let list = List::fetch().unwrap();

    rdescribe("the list", |ctx| {
        ctx.it("should not be empty", || {
            assert!(!list.all().is_empty());
        });

        ctx.it("should have ICANN domains", || {
            assert!(!list.icann().is_empty());
        });

        ctx.it("should have private domains", || {
            assert!(!list.private().is_empty());
        });

        ctx.it("should have at least 1000 domains", || {
            assert!(list.all().len() > 1000);
        });
    });
}
