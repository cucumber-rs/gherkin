#[macro_use]
extern crate cucumber_rust;
use std::default::Default;


pub struct MyWorld { }

impl cucumber_rust::World for MyWorld {}
impl Default for MyWorld {
    fn default() -> MyWorld {
        MyWorld {}
    }
}

mod t {    
    steps! {
        world: ::MyWorld;

        given "I just started" |world, step| {
            println!("HELO");
        };

        when "Hello" |world, step| {

        };
    }
}

cucumber! {
    features: "./tests/features";
    world: ::MyWorld;
    steps: &[
        t::steps
    ]
}