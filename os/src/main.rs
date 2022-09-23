#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![allow(unused_must_use)]
#![feature(linkage)]
#[cfg(feature = "board_qemu")]
use crate::drivers::{KEYBOARD_DEVICE, MOUSE_DEVICE};
use dtb_walker::Property::Status;
use k210_pac::fft::STATUS;

extern crate alloc;

#[macro_use]
extern crate bitflags;
extern crate time;


#[cfg(feature = "board_k210")]
#[path = "boards/k210.rs"]
mod board;
#[cfg(not(any(feature = "board_k210")))]
#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod drivers;
mod dtb;
mod fs;
mod lang_items;
mod logging;
mod mm;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;
// #[cfg(feature = "board_qemu")]
pub mod gui;
mod trace;

// use syscall::create_desktop; //for test

core::arch::global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

use lazy_static::*;
use sync::UPIntrFreeCell;

lazy_static! {
    pub static ref DEV_NON_BLOCKING_ACCESS: UPIntrFreeCell<bool> =
        unsafe { UPIntrFreeCell::new(false) };
}

use crate::drivers::GPU_DEVICE;
use crate::gui::Snake;
pub use log::{debug, error, info, trace, warn};
use riscv::register::sstatus;

#[no_mangle]
pub fn rust_main(_hartid: usize, device_tree_paddr: usize) -> ! {
    clear_bss();
    logging::init();
    mm::init();

    dtb::init_dtb(device_tree_paddr);
    dtb::init_device();

    // panic!("DON'T USE THIS");
    println!("KERN: init gpu");
    #[cfg(feature = "board_qemu")]
    GPU_DEVICE.clone();
    // init_gui::test_gui();
    println!("KERN: init keyboard");
    #[cfg(feature = "board_qemu")]
    KEYBOARD_DEVICE.clone();
    println!("KERN: init mouse");
    #[cfg(feature = "board_qemu")]
    MOUSE_DEVICE.clone();

    println!("KERN: init trap");
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    board::device_init();
    fs::list_apps();

    // #[cfg(feature = "STACK")]
    // init_kernel_data();    // initialize kernel data for stack_trace
    // #[cfg(feature = "KSYMBOL")]
    // trace::init_kernel_trace();

    syscall::create_desktop();

    task::add_initproc();
    *DEV_NON_BLOCKING_ACCESS.exclusive_access() = true;
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
