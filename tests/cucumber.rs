#[macro_use]
extern crate cucumber;
use std::default::Default;

pub struct MyWorld {}

impl cucumber::World for MyWorld {}
impl Default for MyWorld {
    fn default() -> MyWorld {
        MyWorld {}
    }
}

mod t {
    steps!(crate::MyWorld => {
        given "I just started" |_world, _step| {
            println!("HELO");
        };

        when "Hello" |_world, _step| {

        };
    });
}

cucumber! {
    features: "./tests/features",
    world: crate::MyWorld,
    steps: &[
        t::steps
    ]
}
