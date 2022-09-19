use alloc::{collections::VecDeque, sync::Arc};
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{Point, Primitive, RgbColor, Size},
    primitives::{PrimitiveStyle, Rectangle},
    Drawable,
};
use log::info;
use crate::{GPU_DEVICE,UPIntrFreeCell};

use super::{Component, Graphics};

pub struct Panel {
    inner: UPIntrFreeCell<PanelInner>,
}
struct PanelInner {
    back_color: Rgb888,
    graphic: Graphics,
    comps: VecDeque<Arc<dyn Component>>,
}

impl Panel {
    pub fn new(size: Size, point: Point) -> Self {
        Self {
            inner: unsafe {
                UPIntrFreeCell::new(PanelInner {
                    back_color: Rgb888::WHITE,
                    graphic: Graphics {
                        size,
                        point,
                        drv: GPU_DEVICE.exclusive_access().clone(),
                    },
                    comps: VecDeque::new(),
                })
            },
        }
    }
    pub fn with_color(self, color: Rgb888) -> Self {
        self.inner.exclusive_access().back_color = color;
        self
    }
}

impl Component for Panel {
    fn paint(&self) {
        let mut inner = self.inner.exclusive_access();
        Rectangle::new(Point::new(0, 0), inner.graphic.size)
            .into_styled(PrimitiveStyle::with_fill(inner.back_color))
            .draw(&mut inner.graphic)
            .unwrap();

        let len = inner.comps.len();
        info!("paint rect over :{}",len);
        drop(inner);
        for i in 0..len {
            let mut inner = self.inner.exclusive_access();
            let comp = Arc::downgrade(&inner.comps[i]);
            drop(inner);
            comp.upgrade().unwrap().paint();
        }
    }

    fn add(&self, comp:Arc<dyn Component>) {
        info!("add comp");
        let mut inner = self.inner.exclusive_access();
        inner.comps.push_back(comp);
    }

    fn bound(&self) -> (Size, Point) {
        let inner = self.inner.exclusive_access();
        (inner.graphic.size, inner.graphic.point)
    }
}
