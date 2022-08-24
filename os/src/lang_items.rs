use alloc::string::String;
use alloc::vec::Vec;
use crate::sbi::shutdown;
use crate::task::current_kstack_top;
use core::arch::asm;
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use log::trace;
use stack_trace::{Trace};
use crate::fs::{list_apps, open_file, OpenFlags, ROOT_INODE};
use crate::sync::UPIntrFreeCell;
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
    unsafe {
        if KERNEL_DATA.exclusive_access().is_empty(){
            shutdown(255);
        }
        backtrace();
    }
    shutdown(255)
}

lazy_static!{
    static ref KERNEL_DATA: UPIntrFreeCell<Vec<u8>> = unsafe{UPIntrFreeCell::new(Vec::new())};
}
pub fn init_kernel_data(){
    let mut os_name:Vec<&str> = Vec::new();
    let all_file = ROOT_INODE.ls();
    all_file.iter().for_each(|x| {
        if x.contains("os") {
            os_name.push(x);
        }
    });
    os_name.sort();
    os_name.iter().for_each(|name|{
        let mut file = open_file(*name,OpenFlags::RDONLY).unwrap();
        let d = file.read_all();
        trace!("name: {} {}",name,d.len());
        KERNEL_DATA.exclusive_access().extend_from_slice(d.as_slice());
    });
}

unsafe fn backtrace() {
    let mut trace = Trace::new();
    trace.init(KERNEL_DATA.exclusive_access().as_slice());
    let road = trace.trace();
    road.iter().for_each(|s|{
        println!("{}",s);
    });
}
