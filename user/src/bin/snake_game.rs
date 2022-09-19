#![no_std]
#![no_main]

use user_lib::{create_desktop, snake_game};

#[macro_use]
extern crate user_lib;

#[no_mangle]
fn main(){
    println!("Snake Game Running");
    snake_game();
}