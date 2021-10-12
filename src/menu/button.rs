use raylib::prelude::*;
use std::ffi::CString;

pub struct Button {
    pub text: CString,
    pub extends: Vector2,
    pub position: Vector2,
}

impl Button {
    pub fn new(text: String, extends: Vector2, position: Vector2) -> Self {
        let text = CString::new(text).unwrap();
        Self {
            text,
            extends,
            position,
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) -> bool {
        let rec = Rectangle::new(
            self.position.x - self.extends.x * 0.5,
            self.position.y - self.extends.y * 0.5,
            self.extends.x,
            self.extends.y,
        );
        d.gui_button(rec, Some(self.text.as_c_str()))
    }
}
