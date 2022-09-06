#![allow(unused)]
pub mod block;
pub mod chardev;

pub mod gui;
#[cfg(feature = "board_qemu")]
pub mod input;
pub mod plic;
pub mod rtc;
pub mod bus;

pub use block::BLOCK_DEVICE;
#[cfg(feature = "board_qemu")]
pub use chardev::UART;
use lazy_static::lazy_static;

pub use gui::*;
#[cfg(feature = "board_qemu")]
pub use input::*;

use crate::mm::FrameTracker;
use crate::mm::{
    frame_alloc, frame_dealloc, kernel_token, PageTable, PhysAddr, PhysPageNum, StepByOne, VirtAddr,
};
use crate::UPIntrFreeCell;
use alloc::vec::Vec;

lazy_static! {
    static ref QUEUE_FRAMES: UPIntrFreeCell<Vec<FrameTracker>> =
        unsafe { UPIntrFreeCell::new(Vec::new()) };
}

#[no_mangle]
pub extern "C" fn virtio_dma_alloc(pages: usize) -> PhysAddr {
    let mut ppn_base = PhysPageNum(0);
    for i in 0..pages {
        let frame = frame_alloc().unwrap();
        if i == 0 {
            ppn_base = frame.ppn;
        }
        assert_eq!(frame.ppn.0, ppn_base.0 + i);
        QUEUE_FRAMES.exclusive_access().push(frame);
    }
    ppn_base.into()
}

#[no_mangle]
pub extern "C" fn virtio_dma_dealloc(pa: PhysAddr, pages: usize) -> i32 {
    let mut ppn_base: PhysPageNum = pa.into();
    for _ in 0..pages {
        frame_dealloc(ppn_base);
        ppn_base.step();
    }
    0
}

#[no_mangle]
pub extern "C" fn virtio_phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    VirtAddr(paddr.0)
}

#[no_mangle]
pub extern "C" fn virtio_virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    PageTable::from_token(kernel_token())
        .translate_va(vaddr)
        .unwrap()
}


