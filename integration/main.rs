pub mod tests;
use tests::IntegrationTest;

fn setup() {
    println!("setup")
}

fn teardown() {
    println!("teardown")
}

fn main() {
    setup();

    //TODO: run the test
    for test in inventory::iter::<IntegrationTest> {
        (test.test_fn)()
    }

    teardown();
}
