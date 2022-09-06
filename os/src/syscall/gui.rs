use crate::drivers::gui::{VIRTGPU_XRES, VIRTGPU_YRES};
use alloc::{string::ToString, sync::Arc};
use embedded_graphics::{
    prelude::{Point, Size},
};
use log::info;

use crate::drivers::rtc::QEMU_RTC;
use crate::gui::{Bar, Button, Component, GodTerminal, IconController, ImageComp, Panel, Windows};
use crate::{fs::ROOT_INODE, sync::UPIntrFreeCell};

static DT: &[u8] = include_bytes!("../assert/desktop.bmp");

lazy_static::lazy_static!(
    pub static ref DESKTOP:UPIntrFreeCell<Arc<dyn Component>> = unsafe {
        UPIntrFreeCell::new(Arc::new(Panel::new(Size::new(VIRTGPU_XRES, VIRTGPU_YRES), Point::new(0, 0))))
    };
    pub static ref PAD:UPIntrFreeCell<Option<Arc<GodTerminal>>> = unsafe {
        UPIntrFreeCell::new(None)
    };
    pub static ref TIMER:UPIntrFreeCell<Option<Arc<Button>>> = unsafe {
        UPIntrFreeCell::new(None)
    };
);

pub fn create_desktop() -> isize {
    let p: Arc<dyn Component + 'static> = Arc::new(Panel::new(
        Size::new(VIRTGPU_XRES, VIRTGPU_YRES),
        Point::new(0, 0),
    ));
    let image = ImageComp::new(
        Size::new(VIRTGPU_XRES, VIRTGPU_YRES),
        Point::new(0, 0),
        DT,
        Some(p.clone()),
    );
    let icon = IconController::new(ROOT_INODE.ls(), Some(p.clone()));
    p.add(Arc::new(image));
    p.add(Arc::new(icon));
    let mut desktop = DESKTOP.exclusive_access();
    *desktop = p;
    desktop.paint();
    drop(desktop);
    // create_terminal();
    create_desktop_bar();
    // create_windows();
    create_god_terminal();
    1
}

pub fn create_god_terminal() {
    let god_terminal = GodTerminal::new(Size::new(500,  500), Point::new(100, 100));
    god_terminal.add_str("hello world")
        .add_str("\n")
        .add_str("Godterminal");
    let mut pad = PAD.exclusive_access();
    *pad = Some(Arc::new(god_terminal));
}


pub fn create_windows() {
    let desktop = DESKTOP.exclusive_access();
    let windows = Arc::new(Windows::new(Size::new(500, 500), Point::new(40, 40)));
    windows.with_name("windows").paint();
    let windows1 = Arc::new(Windows::new(Size::new(500, 500), Point::new(500, 200)));
    windows1.with_name("Terminal").paint();
    desktop.add(windows);
    desktop.add(windows1);
}


fn create_desktop_bar() {
    info!("create desktop bar");
    let desktop = DESKTOP.exclusive_access();
    let bar = Arc::new(Bar::new(
        Size::new(VIRTGPU_XRES, 48),
        Point::new(0, 752),
        Some(desktop.clone()),
    ));
    static MENU_BMP: &[u8] = include_bytes!("../assert/rust.bmp");
    let img = ImageComp::new(
        Size::new(48, 48),
        bar.bound().1,
        MENU_BMP,
        Some(bar.clone()),
    );
    bar.add(Arc::new(img));
    let rtc_time = unsafe { QEMU_RTC.get().unwrap().read_time() };

    let time_button = Arc::new(Button::new(
        Size::new(100, 48),
        Point::new(VIRTGPU_XRES as i32 - 100, 0),
        Some(bar.clone()),
        rtc_time.to_string(),
    ));
    let mut timer = TIMER.exclusive_access();
    *timer = Some(time_button.clone());
    // bar.add(Arc::new(img));
    bar.add(time_button);
    // bar.add(Arc::new(img1));
    bar.paint();
    // img.paint();
    desktop.add(bar.clone());
}
