use alloc::{string::String, sync::Arc, vec::Vec};
use embedded_graphics::{
    image::Image,
    pixelcolor::Rgb888,
    prelude::{Point, RgbColor, Size},
    Drawable,
};
use log::{error, info};
use tinybmp::Bmp;

use super::{Component, Graphics};
use crate::{VIRTGPU_XRES, VIRTGPU_YRES};
use crate::{GPU_DEVICE, UPIntrFreeCell};
use crate::TextEdit;

static FILEICON: &[u8] = include_bytes!("../../assert/file.bmp");

pub struct IconController {
    inner: UPIntrFreeCell<IconControllerInner>,
}
#[allow(unused)]
pub struct IconControllerInner {
    files: Vec<String>,
    graphic: Graphics,
    parent: Option<Arc<dyn Component>>,
}

impl IconController {
    pub fn new(files: Vec<String>, parent: Option<Arc<dyn Component>>) -> Self {
        // 将整个桌面作为图床
        IconController {
            inner: unsafe {
                UPIntrFreeCell::new(IconControllerInner {
                    files,
                    graphic: Graphics {
                        size: Size::new(*VIRTGPU_XRES.exclusive_access(), *VIRTGPU_YRES.exclusive_access()),
                        point: Point::new(0, 0),
                        drv: GPU_DEVICE.exclusive_access().clone(),
                    },
                    parent,
                })
            },
        }
    }

}

impl Component for IconController {
    fn paint(&self) {
        let mut inner = self.inner.exclusive_access();
        let mut x = 10;
        let mut y = 10;
        let v = inner.files.clone();
        for file in v {
            info!("file: {}", file);
            let bmp = Bmp::<Rgb888>::from_slice(FILEICON).unwrap();
            Image::new(&bmp, Point::new(x, y))
                .draw(&mut inner.graphic)
                .expect("make image error");
            let edit = TextEdit::new(Size::new(64,20),Point::new(x,y+64),None);
            edit.with_font_color(Rgb888::WHITE).add_str(file.as_str()).repaint();
            // info!("creat icon success");
            if y >= 600 {
                x = x + 70;
                y = 10;
            } else {
                y = y + 90;
            }
        }
        info!("paint icon controller success");
    }

    fn add(&self, _comp: Arc<dyn Component>) {
        todo!()
    }

    fn bound(&self) -> (Size, Point) {
        todo!()
    }
}
