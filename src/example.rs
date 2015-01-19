#![feature(unboxed_closures)]
extern crate ucontext;
use ucontext::*;
use std::mem::*;

fn main() {
    ctest();
}

fn dtest() {
    let mut i = 0;
    if let Ok(a) = UContext::get_context() {
        if i != 2 {
            println!("le");
            i += 1;
        } else {
            return;
        }
        a.set_context();
    } else {
        println!("get context fail");
    }
    
}

        
fn ctest() {
    let a = &mut [0us;4096];
    let mut child = UContext::get_context().unwrap();
    child.set_stack(a);
    let mut main = UContext::new();
    //unsafe {child.set_link(transmute(0is));};
    
    child.make_context(ss);
    
    main.swap_context(&child);
    println!("main done");
}

fn ss() {
    println!("I am sub thread");
}

