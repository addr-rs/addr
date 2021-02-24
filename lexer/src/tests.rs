use crate::List;

lazy_static::lazy_static! {
    static ref LIST: List = List::fetch().unwrap();
}

#[test]
fn list_behaviour() {
    rspec::run(&rspec::given("a list", (), |ctx| {
        ctx.it("should not be empty", |_| {
            assert!(!LIST.all().is_empty());
        });

        ctx.it("should have ICANN TLDs", |_| {
            assert!(!LIST.icann().is_empty());
        });

        ctx.it("should have private TLDs", |_| {
            assert!(!LIST.private().is_empty());
        });

        ctx.it("should have at least 1000 TLDs", |_| {
            assert!(LIST.all().len() > 1000);
        });
    }));
}
