#![feature(unboxed_closures)]
extern crate ucontext;
use ucontext::*;
use std::mem::*;

fn main() {
    ctest();
}

fn ctest() {
    let mut child = UContext::new();
    unsafe {getcontext(&mut child);}
    let a = &mut [0us;4096];
    let start: u64 = unsafe {transmute(a)};
    child.set_stack(start as *const u8, (start + 4096) as *const u8);
    let mut main = UContext::new();
    child.set_link(&main);
    let captured_value = "hehehe".to_string();
    child.make_context(move|| println!("closure invoked!, captured: {}", captured_value));

    unsafe{swapcontext(&mut main, &child);};

    println!("main done");
}
