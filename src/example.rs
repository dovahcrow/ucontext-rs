extern crate ucontext;
use ucontext::*;
use std::mem;
use std::default::Default;

fn main() {
    let mut a: UContext = Default::default();
    let mut b: UContext = Default::default();
    a.get_context();
    b.get_context();
    println!("1");
    a.swap_context(&b);
}


