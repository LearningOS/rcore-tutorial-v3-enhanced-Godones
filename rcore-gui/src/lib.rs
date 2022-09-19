#![no_std]

mod gui;
mod basic;
pub use basic::*;
pub use gui::*;

extern crate rcore_sync;
extern crate alloc;

pub use log::{debug, error, info, trace, warn};
pub use rcore_sync::UPIntrFreeCell;

use alloc::{sync::Arc, vec::Vec};
use core::any::Any;
use embedded_graphics::pixelcolor::Rgb888;
use lazy_static::lazy_static;
use tinybmp::Bmp;
use virtio_drivers::{Hal, PhysAddr, VirtAddr, VirtIOGpu, VirtIOHeader};



pub trait GPUDevice: Send + Sync + Any {
    fn update_cursor(&self);
    fn get_frame_buffer(&self) -> &mut [u8];
    fn flush(&self);
}

pub struct VirtIOGPU<H:Hal> {
    gpu: Option<UPIntrFreeCell<VirtIOGpu<'static,H>>>,
    fb: Option<&'static [u8]>,
}
static BMP_DATA: &[u8] = include_bytes!("../assert/mouse.bmp");


lazy_static::lazy_static!(
    pub static ref GPU_DEVICE:UPIntrFreeCell<Arc<dyn GPUDevice>> = unsafe{
        UPIntrFreeCell::new(Arc::new(VirtIOGPU::<FakeHal>::new()))
    };
);
lazy_static!(
  pub static ref VIRTGPU_XRES:UPIntrFreeCell<u32> = unsafe {
        UPIntrFreeCell::new(1280)
    };
);

lazy_static!(
    pub static ref VIRTGPU_YRES:UPIntrFreeCell<u32> = unsafe {
            UPIntrFreeCell::new(800)
        };
);

pub fn init_gpu<H:Hal+Send+Sync+'static>(addr:usize){
    VirtIOGPU::<H>::init(addr);
}

impl <H:Hal+Send+Sync+'static> VirtIOGPU<H> {
    fn new()->Self{
        Self{
            gpu:None,
            fb:None,
        }
    }

    fn init(addr:usize){
        unsafe {
            let mut virtio = VirtIOGpu::<H>::new(&mut *(addr as *mut VirtIOHeader)).unwrap();
            let fbuffer = virtio.setup_framebuffer().unwrap();
            let len = fbuffer.len();
            let ptr = fbuffer.as_mut_ptr();
            let fb = core::slice::from_raw_parts_mut(ptr, len);

            let bmp = Bmp::<Rgb888>::from_slice(BMP_DATA).unwrap();
            let raw = bmp.as_raw();
            let mut b = Vec::new();
            for i in raw.image_data().chunks(3) {
                let mut v = i.to_vec();
                b.append(&mut v);
                if i == [255, 255, 255] {
                    b.push(0x0)
                } else {
                    b.push(0xff)
                }
            }
            virtio.setup_cursor(b.as_slice(), 50, 50, 50, 50).unwrap();
            let build_gpu = VirtIOGPU{
                gpu: Some(UPIntrFreeCell::new(virtio)),
                fb:Some(fb),
            };
            let mut gpu = GPU_DEVICE.exclusive_access();
            *gpu = Arc::new(build_gpu);
        }
    }
}

impl <H:Hal+Send+'static+Sync>GPUDevice for VirtIOGPU<H> {
    fn update_cursor(&self) {}
    fn get_frame_buffer(&self) -> &mut [u8] {
        unsafe {
            let ptr = self.fb.as_ref().unwrap() as *const _ as *mut u8;
            core::slice::from_raw_parts_mut(ptr, self.fb.as_ref().unwrap().len())
        }
    }
    fn flush(&self) {
        self.gpu.as_ref().unwrap().exclusive_access().flush().unwrap();
    }
}

struct FakeHal{}
impl Hal for FakeHal{
    fn dma_alloc(_pages: usize) -> PhysAddr {
        0
    }
    fn dma_dealloc(_paddr: PhysAddr, _pages: usize) -> i32 {
        0
    }
    fn phys_to_virt(_paddr: PhysAddr) -> VirtAddr {
        0
    }
    fn virt_to_phys(_vaddr: VirtAddr) -> PhysAddr {
        0
    }
}
