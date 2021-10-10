use crate::math::Transform2D;
use crate::Rectangle;
use crate::Vector2;
use raylib::prelude::*;

pub struct Sprite {
    texture: WeakTexture2D,
    pub tint: Color,
    scale: f32,
    source_rec: Rectangle,
    dest_rec: Rectangle,
    centered: bool,
    origin: Vector2,
}

#[allow(dead_code)]
impl Sprite {
    pub fn new(texture: WeakTexture2D, centered: bool, scale: f32) -> Sprite {
        let (source_rec, dest_rec) = Sprite::get_recs(&texture, scale);
        let mut sprite = Sprite {
            texture,
            tint: Color::WHITE,
            scale,
            source_rec,
            dest_rec,
            centered,
            origin: Vector2::zero(),
        };

        sprite.update_origin();
        sprite
    }

    fn get_recs(texture: &WeakTexture2D, scale: f32) -> (Rectangle, Rectangle) {
        let source_rec = Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32);
        let dest_rec = Rectangle::new(
            0.0,
            0.0,
            scale * texture.width as f32,
            scale * texture.height as f32,
        );

        (source_rec, dest_rec)
    }

    pub fn set_texture(&mut self, texture: WeakTexture2D) {
        {
            self.source_rec = Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32);
            self.dest_rec = Rectangle::new(
                0.0,
                0.0,
                self.scale * texture.width as f32,
                self.scale * texture.height as f32,
            );
        }

        self.texture = texture;
    }

    pub fn get_scale(&self) -> f32 {
        self.scale
    }

    pub fn set_centered(&mut self, centered: bool) {
        self.centered = centered;
        self.update_origin();
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        self.dest_rec.width = self.scale * self.texture.width as f32;
        self.dest_rec.height = self.scale * self.texture.height as f32;
        self.update_origin();
    }

    pub fn set_tint(&mut self, tint: Color) {
        self.tint = tint;
    }

    fn update_origin(&mut self) {
        if self.centered {
            self.origin.x = self.texture.width as f32 * self.scale * 0.5;
            self.origin.y = self.texture.height as f32 * self.scale * 0.5;
        } else {
            self.origin.x = 0.0;
            self.origin.y = 0.0;
        }
    }

    pub fn draw(&self, rl: &mut RaylibMode2D<RaylibDrawHandle>, transform: &Transform2D) {
        let mut dest_rec = self.dest_rec;
        dest_rec.x = transform.position.x;
        dest_rec.y = transform.position.y;

        rl.draw_texture_pro(
            &self.texture,
            self.source_rec,
            dest_rec,
            self.origin,
            RAD2DEG as f32 * transform.rotation,
            self.tint,
        );
    }
}
