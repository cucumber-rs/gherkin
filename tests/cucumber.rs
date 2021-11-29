use std::convert::Infallible;

use async_trait::async_trait;
use cucumber::WorldInit;
use futures::executor;

#[derive(Debug, WorldInit)]
pub struct MyWorld;

#[async_trait(?Send)]
impl cucumber::World for MyWorld {
    type Error = Infallible;

    async fn new() -> Result<Self, Self::Error> {
        Ok(MyWorld {})
    }
}

fn main() {
    // Do any setup you need to do before running the Cucumber runner.
    // e.g. setup_some_db_thing()?;

    let runner = MyWorld::cucumber().run_and_exit("tests/features");

    // You may choose any executor you like (`tokio`, `async-std`, etc)
    // You may even have an async main, it doesn't matter. The point is that
    // Cucumber is composable. :)
    executor::block_on(runner);
}
