extern crate ucontext;
use ucontext::*;
use std::mem;
use std::default::Default;

fn main() {
    let mut a: UContext = Default::default();
    a.get_context();
    println!("1");
    a.set_context();
}


