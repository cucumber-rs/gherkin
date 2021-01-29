extern crate cucumber;

use async_trait::async_trait;
use std::convert::Infallible;

pub struct MyWorld {}

#[async_trait(?Send)]
impl cucumber::World for MyWorld {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        Ok(MyWorld {})
    }
}

mod t {
    use cucumber::Steps;

    pub fn steps() -> Steps<crate::MyWorld> {
        let builder: Steps<crate::MyWorld> = Steps::new();
        builder
    }
}

fn main() {
    // Do any setup you need to do before running the Cucumber runner.
    // e.g. setup_some_db_thing()?;

    let runner = cucumber::Cucumber::<MyWorld>::new()
        .features(&["./tests/features"])
        .steps(t::steps());

    // You may choose any executor you like (Tokio, async-std, etc)
    // You may even have an async main, it doesn't matter. The point is that
    // Cucumber is composable. :)
    futures::executor::block_on(runner.run());
}
