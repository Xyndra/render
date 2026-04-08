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

static MOUSE_POS: LazyLock<Mutex<(i32, i32)>> = LazyLock::new(|| Mutex::new((0, 0)));
#[derive(Debug)]
pub enum Events {
    Hover {
        x: i32,
        y: i32,
    },
    PrimaryClick {
        x: i32,
        y: i32,
        click_device: ClickDevice,
    },
    SecondaryClick {
        x: i32,
        y: i32,
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

pub fn update_mouse_position(x: i32, y: i32) {
    let mut pos = MOUSE_POS.lock().unwrap();
    *pos = (x, y);
}

pub fn get_mouse_position() -> (i32, i32) {
    let pos = MOUSE_POS.lock().unwrap();
    *pos
}
