use raylib::prelude::*;

mod button;
use button::Button;

pub struct Menu<'a> {
    rl: &'a mut RaylibHandle,
    thread: &'a RaylibThread,
    window_size: (i16, i16),
    center: Vector2,
    short_button: Button,
    medium_button: Button,
    long_button: Button,
    quit_button: Button,
    font: Font,
    random_levels: bool,
}

impl<'a> Menu<'a> {
    pub fn new(
        rl: &'a mut RaylibHandle,
        thread: &'a RaylibThread,
        window_width: i16,
        window_height: i16,
        random_levels: bool,
    ) -> Self {
        rl.show_cursor();
        let center = Vector2::new((window_width / 2).into(), (window_height / 2).into());

        let mut line = 0.;
        let short_button = Button::new(
            "Short".to_string(),
            Vector2::new(120., 40.),
            center + Vector2::new(0., 50. * line),
        );
        line += 1.;
        let medium_button = Button::new(
            "Medium".to_string(),
            Vector2::new(120., 40.),
            center + Vector2::new(0., 50. * line),
        );
        line += 1.;
        let long_button = Button::new(
            "Long".to_string(),
            Vector2::new(120., 40.),
            center + Vector2::new(0., 50. * line),
        );
        line += 1.;
        let quit_button = Button::new(
            "Quit".to_string(),
            Vector2::new(120., 40.),
            center + Vector2::new(0., 50. * line),
        );

        let font = rl
            .load_font_ex(
                thread,
                "resources/fonts/Roboto-Regular.ttf",
                100,
                FontLoadEx::Default(0),
            )
            .expect("Couldn't load font");

        Menu {
            rl,
            thread,
            window_size: (window_width, window_height),
            center,
            short_button,
            medium_button,
            long_button,
            quit_button,
            font,
            random_levels,
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
            self.short_button.position = self.center;
        }

        let esc_pressed = self.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);

        let mut d = self.rl.begin_drawing(self.thread);
        d.clear_background(Color::GOLD);
        d.gui_set_font(&self.font);
        d.gui_set_style(
            GuiControl::DEFAULT,
            GuiDefaultProperty::TEXT_SIZE as i32,
            100,
        );
        d.gui_label(
            Rectangle::new(self.center.x - 250., 10., 200., 200.),
            Some(rstr!("RAYVARUST")),
        );
        d.gui_set_style(
            GuiControl::DEFAULT,
            GuiDefaultProperty::TEXT_SIZE as i32,
            22,
        );

        let mut toggle_text = rstr!("Random levels: ON");
        if !self.random_levels {
            toggle_text = rstr!("Random levels: OFF");
        }
        self.random_levels = d.gui_toggle(Rectangle::new(300.,300., 200., 50.), Some(toggle_text), self.random_levels);

        // Select short level
        let short = self.short_button.draw(&mut d);
        if short {
            return Some(MenuAction::Start(0, self.random_levels));
        }
        // Select medium level
        let medium = self.medium_button.draw(&mut d);
        if medium {
            return Some(MenuAction::Start(1, self.random_levels));
        }
        // Select long level
        let long = self.long_button.draw(&mut d);
        if long {
            return Some(MenuAction::Start(2, self.random_levels));
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
    Start(usize, bool),
    Quit,
}
