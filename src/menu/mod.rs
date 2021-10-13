use raylib::prelude::*;

mod button;
use button::Button;

pub struct Menu<'a> {
    rl: &'a mut RaylibHandle,
    thread: &'a RaylibThread,
    window_size: (i16, i16),
    center: Vector2,
    start_button: Button,
    quit_button: Button,
}

impl<'a> Menu<'a> {
    pub fn new(
        rl: &'a mut RaylibHandle,
        thread: &'a RaylibThread,
        window_width: i16,
        window_height: i16,
    ) -> Self {
        rl.show_cursor();
        let center = Vector2::new((window_width / 2).into(), (window_height / 2).into());

        let start_button = Button::new("Start Game".to_string(), Vector2::new(100., 50.), center);
        let quit_button = Button::new(
            "Quit".to_string(),
            Vector2::new(100., 50.),
            center + Vector2::new(0., 75.),
        );

        Menu {
            rl,
            thread,
            window_size: (window_width, window_height),
            center,
            start_button,
            quit_button,
        }
    }

    pub fn step(&mut self) -> Option<MenuAction> {
        if self.rl.is_window_resized() {
            let window_width = self.rl.get_screen_width() as i16;
            let window_height = self.rl.get_screen_height() as i16;
            self.window_size = (window_width, window_height);
            self.center = Vector2::new(
                (self.window_size.0 / 2).into(),
                (self.window_size.1 / 2).into(),
            );
            self.start_button.position = self.center;
        }

        let esc_pressed = self.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);

        let mut d = self.rl.begin_drawing(self.thread);
        d.clear_background(Color::GOLD);
        let start = self.start_button.draw(&mut d);
        if start {
            return Some(MenuAction::Start);
        }
        let quit_b_pressed = self.quit_button.draw(&mut d);

        if quit_b_pressed || esc_pressed {
            return Some(MenuAction::Quit);
        }
        None
    }

    pub fn run(&mut self) -> MenuAction {
        while !self.rl.window_should_close() {
            let action = self.step();
            if let Some(action) = action {
                return action;
            }
        }
        MenuAction::Quit
    }
}

#[derive(PartialEq, Eq)]
pub enum MenuAction {
    Start,
    Quit,
}