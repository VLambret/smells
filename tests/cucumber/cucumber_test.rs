use crate::cucumber_test::AnimalWorld;
use cucumber::World;

fn main() {
    // Run the cucumber test
    futures::executor::block_on(AnimalWorld::run("tests/cucumber/features"));
}

mod cucumber_test {
    use cucumber::{given, then, when, World};
    // These `Cat` definitions would normally be inside your project's code,
    // not test code, but we create them here for the show case.
    #[derive(Debug, Default)]
    struct Cat {
        pub hungry: bool,
    }

    impl Cat {
        fn feed(&mut self) {
            self.hungry = false;
        }
    }

    // `World` is your shared, likely mutable state.
    // Cucumber constructs it via `Default::default()` for each scenario.
    #[derive(Debug, Default, World)]
    pub struct AnimalWorld {
        cat: Cat,
    }

    #[given(expr = "a {word} cat")]
    fn hungry_cat(world: &mut AnimalWorld, state: String) {
        match state.as_str() {
            "hungry" => world.cat.hungry = true,
            "satiated" => world.cat.hungry = false,
            s => panic!("expected 'hungry' or 'satiated', found: {s}"),
        }
    }

    #[when("I feed the cat")]
    fn feed_cat(world: &mut AnimalWorld) {
        world.cat.feed();
    }

    #[then("the cat is not hungry")]
    fn cat_is_fed(_world: &mut AnimalWorld) {
        assert!(true);
    }
}
