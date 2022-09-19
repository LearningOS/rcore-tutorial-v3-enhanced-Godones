use crate::drivers::gui::{VIRTGPU_XRES, VIRTGPU_YRES};
use crate::drivers::rtc::QEMU_RTC;
use crate::fs::ROOT_INODE;
use crate::gui::{
    Bar, Button, Component, GodTerminal, IconController, ImageComp, Panel, Status, Windows,
    SCREEN_MANAGER, SNAKE,
};
use crate::{Snake, UPIntrFreeCell};
use alloc::{string::ToString, sync::Arc};
use embedded_graphics::prelude::{Point, Size};
use log::info;
use virtio_input_decoder::Key;

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
    info!("create desktop");
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
    info!("create god terminal");
    let god_terminal = GodTerminal::new(Size::new(500, 500), Point::new(400, 100));
    let mut pad = PAD.exclusive_access();
    *pad = Some(Arc::new(god_terminal));
}

pub fn create_windows() {
    let desktop = DESKTOP.exclusive_access();
    let windows = Windows::new(Size::new(500, 500), Point::new(40, 40));
    windows.set_title("windows").paint();
    let windows1 = Windows::new(Size::new(500, 500), Point::new(500, 200));
    windows1.set_title("Terminal").paint();
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

pub fn snake_game() -> isize {
    let ans = Snake::new().run();
    loop {
        let event = SNAKE.exclusive_access().as_ref().unwrap().get_event();
        if ans == Status::Dead {
            if let Some(key) = event {
                if key == Key::R {
                    Snake::new().run();
                }
            }
        }
    }
    0
}
