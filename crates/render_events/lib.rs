use std::sync::{LazyLock, Mutex};

#[derive(Debug)]
pub enum ClickDevice {
    Mouse,
    Touch,
    Stylus,
}

#[derive(Debug)]
pub enum SecondaryClickType {
    LongClick,
    MiddleClick,
    RightClick,
    LongPressTouch,
    LongPressStylus,
    StylusButton(i32), // button index? (todo!)
}

#[derive(Debug)]
pub enum Events {
    Hover {
        x: u32,
        y: u32,
    },
    PrimaryClick {
        x: u32,
        y: u32,
        click_device: ClickDevice,
    },
    SecondaryClick {
        x: u32,
        y: u32,
        secondary_click_type: SecondaryClickType,
    },
    Resize {
        width: u32,
        height: u32,
    },
    // Move is seperate from Resize animations and drag and drop, which are already resource-intensive.
    Move {
        x: u32,
        y: u32,
    },
    KeyPress {
        key: char,
    },
}

static MOUSE_POS: LazyLock<Mutex<(u32, u32)>> = LazyLock::new(|| Mutex::new((0, 0)));
pub fn update_mouse_position(x: u32, y: u32) {
    let mut pos = MOUSE_POS.lock().unwrap();
    *pos = (x, y);
}

pub fn get_mouse_position() -> (u32, u32) {
    let pos = MOUSE_POS.lock().unwrap();
    *pos
}
