use raylib::prelude::*;

use crate::SHIP_NAMES;

mod button;
use button::Button;

pub struct Menu<'a> {
    rl: &'a mut RaylibHandle,
    thread: &'a RaylibThread,
    window_size: (i16, i16),
    center: Vector2,
    bg_tex: Texture2D,
    short_button: Button,
    medium_button: Button,
    long_button: Button,
    quit_button: Button,
    font: Font,
    random_levels: bool,
    fuel_mode: bool,
    pub selected_ship: usize,
    ship_prev: Button,
    ship_next: Button,
    ship_textures: Vec<WeakTexture2D>,
}

const SHIP_SELECT_POS: Vector2 = Vector2 { x: 200.0, y: 440.0 };

impl<'a> Menu<'a> {
    pub fn new(
        rl: &'a mut RaylibHandle,
        thread: &'a RaylibThread,
        window_width: i16,
        window_height: i16,
        random_levels: bool,
        fuel_mode: bool,
        selected_ship: usize,
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

        let ship_prev = Button::new(
            "<".to_string(),
            rvec2(40., 40.),
            SHIP_SELECT_POS + rvec2(90.0, 280.),
        );

        let ship_next = Button::new(
            ">".to_string(),
            rvec2(40., 40.),
            SHIP_SELECT_POS + rvec2(170., 280.),
        );

        let font = rl
            .load_font_ex(
                thread,
                "resources/fonts/Roboto-Regular.ttf",
                100,
                FontLoadEx::Default(0),
            )
            .expect("Couldn't load font");

        let mut ship_textures = vec![];
        for ship_name in SHIP_NAMES {
            let tex = rl
                .load_texture(
                    thread,
                    &("resources/textures/ships/".to_owned() + ship_name + ".png"),
                )
                .unwrap();
            let weak = unsafe { tex.make_weak() };
            ship_textures.push(weak);
        }

        let bg_tex = rl
            .load_texture(thread, "resources/textures/title_screen.png")
            .unwrap();

        Menu {
            rl,
            thread,
            window_size: (window_width, window_height),
            center,
            bg_tex,
            short_button,
            medium_button,
            long_button,
            quit_button,
            font,
            random_levels,
            fuel_mode,
            selected_ship,
            ship_prev,
            ship_next,
            ship_textures,
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
        }

        let esc_pressed = self.rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);

        let mut d = self.rl.begin_drawing(self.thread);
        d.clear_background(Color::DARKPURPLE);
        d.draw_texture(&self.bg_tex, 0, 0, Color::WHITE);

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
        self.random_levels = d.gui_toggle(
            Rectangle::new(1200., 700., 200., 50.),
            Some(toggle_text),
            self.random_levels,
        );
        let mut toggle_text = rstr!("Fuel mode: ON");
        if !self.fuel_mode {
            toggle_text = rstr!("Fuel mode: OFF");
        }
        self.fuel_mode = d.gui_toggle(
            Rectangle::new(1200., 760., 200., 50.),
            Some(toggle_text),
            self.fuel_mode,
        );

        // Buttons for selecting ship
        let ship_p = self.ship_prev.draw(&mut d);
        if ship_p {
            if self.selected_ship == 0 {
                self.selected_ship = SHIP_NAMES.len();
            }
            self.selected_ship -= 1;
        }
        let ship_n = self.ship_next.draw(&mut d);
        if ship_n {
            self.selected_ship += 1;
            if self.selected_ship >= SHIP_NAMES.len() {
                self.selected_ship = 0;
            }
        }

        // Draw selected ship
        d.draw_texture_v(
            self.ship_textures[self.selected_ship].clone(),
            SHIP_SELECT_POS,
            Color::WHITE,
        );

        // Select short level
        let short = self.short_button.draw(&mut d);
        if short {
            return Some(MenuAction::Start(0, self.random_levels, self.fuel_mode));
        }
        // Select medium level
        let medium = self.medium_button.draw(&mut d);
        if medium {
            return Some(MenuAction::Start(1, self.random_levels, self.fuel_mode));
        }
        // Select long level
        let long = self.long_button.draw(&mut d);
        if long {
            return Some(MenuAction::Start(2, self.random_levels, self.fuel_mode));
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

    pub fn unload(&mut self) {
        unsafe {
            for tex in &self.ship_textures {
                self.rl.unload_texture(self.thread, tex.clone());
            }
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum MenuAction {
    Start(usize, bool, bool),
    Quit,
}
