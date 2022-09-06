
pub fn test_gui(){
    info!("test gui");
    rcore_gui::init_gpu::<VirtioHal>(0x10007000);
    let windows = Arc::new(Windows::new(Size::new(500, 500), Point::new(40, 40)));
    windows.with_name("windows").paint();
    let windows1 = Arc::new(Windows::new(Size::new(500, 500), Point::new(500, 200)));
    windows1.with_name("Terminal").paint();
}

use alloc::sync::Arc;
use crate::UPIntrFreeCell;
use alloc::vec::Vec;
use embedded_graphics::prelude::{Point, Size};
use crate::mm::{FrameTracker, kernel_token, PageTable, PhysAddr, VirtAddr};
use lazy_static::lazy_static;
use log::{info, trace};
use rcore_gui::{Component, GPU_DEVICE, Windows};
use virtio_drivers::{Hal};
use crate::drivers::bus::virtio::VirtioHal;
use crate::mm::{frame_alloc, frame_dealloc, PhysPageNum, StepByOne};

lazy_static! {
    static ref QUEUE_FRAMES: UPIntrFreeCell<Vec<FrameTracker>> =
        unsafe { UPIntrFreeCell::new(Vec::new()) };
}
