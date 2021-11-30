use raylib::prelude::*;
use std::ffi::CString;

use crate::SHIP_NAMES;

mod button;
use button::Button;

pub struct Menu<'a> {
    rl: &'a mut RaylibHandle,
    thread: &'a RaylibThread,
    window_size: (i16, i16),
    center: Vector2,
    bg_tex: Texture2D,
    start_button: Button,
    quit_button: Button,
    selected_length: f32,
    font: Font,
    random_levels: bool,
    fuel_mode: bool,
    pub selected_ship: usize,
    ship_prev: Button,
    ship_next: Button,
    ship_textures: Vec<WeakTexture2D>,
    popup_texture: Texture2D,
    popup_open: bool,
    popup_button: Button,
}

const SHIP_SELECT_POS: Vector2 = Vector2 { x: 200.0, y: 440.0 };
const POPUP_POS: Vector2 = Vector2 { x: 500.0, y: 300.0 };

const POPUP_TEXT: &str = "
Controls:
  WSAD - acceleration
  IO - rotation
  KL - zoom in/out
Rules:
  Go through a gate = +30 points
  Touch anything = -10 points
  Have less than 0 points = fail
  Go through all gates = completion
";

impl<'a> Menu<'a> {
    pub fn new(
        rl: &'a mut RaylibHandle,
        thread: &'a RaylibThread,
        window_width: i16,
        window_height: i16,
        random_levels: bool,
        fuel_mode: bool,
        selected_ship: usize,
        selected_length: f32,
    ) -> Self {
        rl.show_cursor();
        let center = rvec2((window_width / 2) as f32, (window_height / 2) as f32);

        let mut line = 0.;
        let start_button = Button::new(
            "Start".to_string(),
            rvec2(120., 40.),
            center + rvec2(0., 50. * line),
        );
        line += 1.;
        let quit_button = Button::new(
            "Quit".to_string(),
            rvec2(120., 40.),
            center + rvec2(0., 50. * line),
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

        let popup_button = Button::new(
            "ABOUT".to_string(),
            rvec2(130., 60.),
            POPUP_POS + rvec2(900., 590.),
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

        let popup_texture = rl
            .load_texture(thread, "resources/textures/popup.png")
            .unwrap();

        Menu {
            rl,
            thread,
            window_size: (window_width, window_height),
            center,
            bg_tex,
            selected_length,
            start_button,
            quit_button,
            font,
            random_levels,
            fuel_mode,
            selected_ship,
            ship_prev,
            ship_next,
            ship_textures,
            popup_texture,
            popup_open: false,
            popup_button,
        }
    }

    pub fn step(&mut self) -> Option<MenuAction> {
        if self.rl.is_window_resized() {
            let window_width = self.rl.get_screen_width() as i16;
            let window_height = self.rl.get_screen_height() as i16;
            self.window_size = (window_width, window_height);
            self.center = rvec2(
                (self.window_size.0 / 2) as f32,
                (self.window_size.1 / 2) as f32,
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
            rrect(self.center.x - 250., 10., 200., 200.),
            Some(rstr!("RAYVARUST")),
        );
        d.gui_set_style(
            GuiControl::DEFAULT,
            GuiDefaultProperty::TEXT_SIZE as i32,
            22,
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

        // Disable guis under popup
        if self.popup_open {
            d.gui_disable();
        }

        // Random levels toggle
        let mut toggle_text = rstr!("Random levels: ON");
        if !self.random_levels {
            toggle_text = rstr!("Random levels: OFF");
        }
        self.random_levels = d.gui_toggle(
            rrect(1200., 700., 200., 50.),
            Some(toggle_text),
            self.random_levels,
        );

        // Fuel mode toggle
        let mut toggle_text = rstr!("Fuel mode: ON");
        if !self.fuel_mode {
            toggle_text = rstr!("Fuel mode: OFF");
        }
        self.fuel_mode = d.gui_toggle(
            rrect(1200., 760., 200., 50.),
            Some(toggle_text),
            self.fuel_mode,
        );

        // Level length slider
        self.selected_length = d
            .gui_slider(
                rrect(
                    self.start_button.position.x - 100.0,
                    self.start_button.position.y - 60.,
                    200.,
                    30.,
                ),
                Some(&CString::new(self.selected_length.to_string()).unwrap()),
                None,
                self.selected_length,
                6.,
                32.,
            )
            .round();

        // Start level
        let start = self.start_button.draw(&mut d);
        if start {
            return Some(MenuAction::Start(
                self.selected_length as u16,
                self.random_levels,
                self.fuel_mode,
            ));
        }

        // Quit game
        let quit_b_pressed = self.quit_button.draw(&mut d);
        if quit_b_pressed || esc_pressed {
            return Some(MenuAction::Quit);
        }

        d.gui_enable();

        // Toggle popup
        let popup_button_pressed = self.popup_button.draw(&mut d);
        self.popup_open ^= popup_button_pressed;

        if self.popup_open {
            // Draw popup
            d.draw_texture_v(&self.popup_texture, POPUP_POS, Color::WHITE);

            // Draw popup text
            d.draw_text_ex(
                &self.font,
                POPUP_TEXT,
                POPUP_POS + rvec2(45., -10.),
                35.0,
                0.0,
                Color::RAYWHITE,
            );

            d.draw_text_ex(
                &self.font,
                "(Both hands on keyboard recommended)",
                POPUP_POS + rvec2(345., 40.),
                35.0,
                0.0,
                Color::RAYWHITE,
            );
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
    Start(u16, bool, bool),
    Quit,
}
