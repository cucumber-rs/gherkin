#[macro_use]
extern crate cucumber_rust;
use std::default::Default;


pub struct World { }

impl Default for World {
    fn default() -> World {
        World {}
    }
}
    
cucumber! {
    features: "./tests/features";
    world: World;

    given "I just started" |world| {
        println!("HELO");
    };

    when "Hello" |world| {

    };
}