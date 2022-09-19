use crate::{Bar, Component, Graphics, ImageComp, Panel};
use crate::UPIntrFreeCell;
use crate::GPU_DEVICE;
use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::text::Text;
use embedded_graphics::text::{Baseline};
use embedded_graphics::Drawable;
use log::info;


pub struct Windows {
    inner: UPIntrFreeCell<WindowsInner>,
}

struct WindowsInner {
    name: String,
    component: VecDeque<Arc<dyn Component>>,
    graphic: Graphics,
}

const TITLE_BAR_HEIGHT: i32 = 20;
const TITLE_ICON_WH: i32 = 16;
const TITLE_ICON_SPACE: i32 = 4;
impl Windows {
    pub fn new(size: Size, point: Point) -> Self {
        // 加一个bar
        let bar_size = Size::new(size.width, TITLE_BAR_HEIGHT as u32);
        let bar_point = Point::new(point.x, point.y - TITLE_BAR_HEIGHT);
        let bar = Arc::new(Bar::new(bar_size, bar_point, None));
        static CLOSE_IMG: &[u8] = include_bytes!("../../assert/close-circle.bmp");
        let close_img = Arc::new(ImageComp::new(
            Size::new(TITLE_ICON_WH as u32, TITLE_ICON_WH as u32),
            Point::new(
                point.x + size.width as i32 - TITLE_ICON_WH,
                point.y - TITLE_BAR_HEIGHT + 2,
            ),
            CLOSE_IMG,
            Some(bar.clone()),
        ));
        static MAX_IMG: &[u8] = include_bytes!("../../assert/maximize.bmp");
        let max_img = Arc::new(ImageComp::new(
            Size::new(TITLE_ICON_WH as u32, TITLE_ICON_WH as u32),
            Point::new(
                point.x + size.width as i32 - TITLE_ICON_WH * 2 - TITLE_ICON_SPACE,
                point.y - TITLE_BAR_HEIGHT + 2,
            ),
            MAX_IMG,
            Some(bar.clone()),
        ));
        static MIN_IMG: &[u8] = include_bytes!("../../assert/reminder_minus_circle.bmp");
        let min_img = Arc::new(ImageComp::new(
            Size::new(TITLE_ICON_WH as u32, TITLE_ICON_WH as u32),
            Point::new(
                point.x + size.width as i32 - TITLE_ICON_WH * 3 - TITLE_ICON_SPACE * 2,
                point.y - TITLE_BAR_HEIGHT + 2,
            ),
            MIN_IMG,
            Some(bar.clone()),
        ));
        bar.add(min_img);
        bar.add(close_img);
        bar.add(max_img);

        let windows = Arc::new(Panel::new(size, point));
        Self {
            inner: unsafe {
                UPIntrFreeCell::new(WindowsInner {
                    name: "".to_string(),
                    component: {
                        let mut v: VecDeque<Arc<dyn Component>> = VecDeque::new();
                        v.push_back(bar);
                        v.push_back(windows);
                        v
                    },
                    graphic: Graphics {
                        size,
                        point,
                        drv: GPU_DEVICE.exclusive_access().clone(),
                    },
                })
            },
        }
    }
    pub fn with_name(&self, name: &str) -> &Self {
        let mut inner = self.inner.exclusive_access();
        inner.name = name.to_string();
        self
    }
}

impl Component for Windows {
    fn paint(&self) {
        // 由一个bar + panel组成
        let inner = self.inner.exclusive_access();
        let mut gra = Graphics::new(
            Size::new(100, 20),
            Point::new(
                inner.graphic.point.x,
                inner.graphic.point.y - TITLE_BAR_HEIGHT,
            ),
        );
        inner.component.iter().for_each(|com| {
            com.paint();
        });
        // 渲染窗口名称
        Text::with_baseline(
            &inner.name,
            Point::new(0, 0),
            MonoTextStyle::new(&FONT_10X20, Rgb888::BLACK),
            Baseline::Top,
        )
        .draw(&mut gra)
        .unwrap();
    }
    fn add(&self, comp: Arc<dyn Component>) {
        let mut inner = self.inner.exclusive_access();
        inner.component.push_back(comp);
    }
    fn bound(&self) -> (Size, Point) {
        let inner = self.inner.exclusive_access();
        (inner.graphic.size, inner.graphic.point)
    }
}
