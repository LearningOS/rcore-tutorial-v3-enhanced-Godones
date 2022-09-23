use crate::fs::{open_file, OpenFlags, ROOT_INODE};
use crate::sbi::shutdown;
use crate::sync::UPIntrFreeCell;
use alloc::vec::Vec;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use log::trace;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("[kernel] Panicked: {}", info.message().unwrap());
    }
    stack_trace();
    shutdown(255)
}

#[no_mangle]
fn stack_trace() {
    let info = crate::trace::init_kernel_trace();
    let func_info = unsafe { trace_lib::my_trace(info) };
    func_info.iter().for_each(|x| {
        println!("{}", x);
    });
}
