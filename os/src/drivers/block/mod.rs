// mod sdcard;
mod virtio_blk;

// pub use sdcard::SDCardWrapper;
pub use virtio_blk::VirtIOBlock;

use crate::board::BlockDeviceImpl;
use crate::fs::easy_fs::BlockDevice;
use alloc::sync::Arc;
use lazy_static::*;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = {
        kprintln!("[KERN] drivers::block::lazy_static!BLOCK_DEVICE begin");
        Arc::new(BlockDeviceImpl::new())
    };
}

#[allow(unused)]
pub fn block_device_test() {
    let block_device = BLOCK_DEVICE.clone();
    let mut write_buffer = [0u8; 512];
    let mut read_buffer = [0u8; 512];
    for i in 0..512 {
        for byte in write_buffer.iter_mut() {
            *byte = i as u8;
        }
        block_device.write_block(i as usize, &write_buffer);
        block_device.read_block(i as usize, &mut read_buffer);
        assert_eq!(write_buffer, read_buffer);
    }
    println!("block device test passed!");
}
