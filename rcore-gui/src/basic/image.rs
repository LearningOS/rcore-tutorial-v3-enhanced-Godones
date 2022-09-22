use alloc::{sync::Arc};
use embedded_graphics::{
    image::Image,
    pixelcolor::Rgb888,
    prelude::{Point, Size},
    Drawable,
};
use log::info;
use tinybmp::Bmp;

use crate::{
    GPU_DEVICE,
    UPIntrFreeCell,
};

use super::{Component, Graphics};

pub struct ImageComp {
    inner: UPIntrFreeCell<ImageInner>,
}
#[allow(unused)]
pub struct ImageInner {
    image: &'static [u8],
    graphic: Graphics,
    parent: Option<Arc<dyn Component>>,
}

impl ImageComp {
    pub fn new(
        size: Size,
        point: Point,
        v: &'static [u8],
        parent: Option<Arc<dyn Component>>,
    ) -> Self {
        unsafe {
            ImageComp {
                inner: UPIntrFreeCell::new(ImageInner {
                    parent,
                    image: v,
                    graphic: Graphics {
                        size,
                        point,
                        drv: GPU_DEVICE.exclusive_access().clone(),
                    },
                }),
            }
        }
    }
}

impl Component for ImageComp {
    fn paint(&self) {
        info!("paint image");
        let mut inner = self.inner.exclusive_access();
        let b = unsafe {
            let len = inner.image.len();
            let ptr = inner.image.as_ptr() as *const u8;
            core::slice::from_raw_parts(ptr, len)
        };
        let bmp = Bmp::<Rgb888>::from_slice(b).unwrap();
        info!("bmp size: {:?}", b.len());
        Image::new(&bmp, Point::new(0, 0))
            .draw(&mut inner.graphic)
            .expect("make image error");
        info!("paint image done");
    }

    fn add(&self, _comp: alloc::sync::Arc<dyn Component>) {
        todo!()
    }

    fn bound(&self) -> (Size, Point) {
        let inner = self.inner.exclusive_access();
        (inner.graphic.size, inner.graphic.point)
    }
}