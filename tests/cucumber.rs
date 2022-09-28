use std::convert::Infallible;

use cucumber::World;
use futures::executor;

#[derive(Debug, Default, World)]
pub struct MyWorld;

fn main() {
    // Do any setup you need to do before running the Cucumber runner.
    // e.g. setup_some_db_thing()?;

    let runner = MyWorld::cucumber().run_and_exit("tests/features");

    // You may choose any executor you like (`tokio`, `async-std`, etc)
    // You may even have an async main, it doesn't matter. The point is that
    // Cucumber is composable. :)
    executor::block_on(runner);
}
