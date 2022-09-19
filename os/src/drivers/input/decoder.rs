use crate::gui::{SCREEN_MANAGER, SNAKE};
use crate::syscall::PAD;
use crate::GPU_DEVICE;
use alloc::string::ToString;
use log::info;
use virtio_input_decoder::{DecodeType, Key, KeyType, Mouse};

pub fn decode(dtype: DecodeType) {
    match dtype {
        virtio_input_decoder::DecodeType::Key(key, r#type) => {
            println!("{:?} {:?}", key, r#type);
            if r#type == KeyType::Press {
                decode_keyboard(key);
            }
            if r#type == KeyType::Press {
                decode_mouse_p_r(key);
            }
        }
        virtio_input_decoder::DecodeType::Mouse(mouse) => decode_mouse(mouse),
    }
}
fn decode_mouse_p_r(key: Key) {
    match key {
        Key::MouseLeft => {
            // SCREEN_MANAGER.exclusive_access().mouse_left_press();
            GPU_DEVICE.update_cursor()
        }
        Key::MouseRight => {
            // SCREEN_MANAGER.exclusive_access().mouse_right_press();
        }
        _ => {}
    }
}

fn decode_keyboard(key: Key) {
    let inner = PAD.exclusive_access();
    let a = inner.as_ref().unwrap();
    if key == virtio_input_decoder::Key::BackSpace {
        a.add_special_char(127);
    }
    match key.to_char() {
        Ok(k) => {
            if k == '\r' {
                a.add_str(&(k.to_string() + "\n"));
            } else {
                a.add_str(k.to_string().as_str());
                SNAKE
                    .exclusive_access()
                    .as_ref()
                    .unwrap()
                    .receive_event(key);
            }
        }
        Err(_) => {}
    }
}

fn decode_mouse(mouse: Mouse) {
    println!("mouse: {:?}", mouse);
    // let screen_manager = SCREEN_MANAGER.exclusive_access();
}
