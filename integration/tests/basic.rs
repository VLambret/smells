use super::IntegrationTest;

fn basic_test() {
    println!("running basic test");
}

fn basic_test2222() {
    println!("basic test 2222");
}

inventory::submit!(IntegrationTest {
    name: "basic",
    test_fn: basic_test
});

inventory::submit!(IntegrationTest {
    name: "basic2222",
    test_fn: basic_test2222
});
