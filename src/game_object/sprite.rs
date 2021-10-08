use crate::math::Transform2D;
use crate::Rectangle;
use crate::Vector2;
use raylib::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Sprite {
    texture: Rc<RefCell<WeakTexture2D>>,
    scale: f32,
    source_rec: Rectangle,
    dest_rec: Rectangle,
    centered: bool,
    origin: Vector2,
}

#[allow(dead_code)]
impl Sprite {
    pub fn new(texture: Rc<RefCell<WeakTexture2D>>, centered: bool, scale: f32) -> Sprite {
        let (source_rec, dest_rec) = Sprite::get_recs(&texture, scale);
        let mut sprite = Sprite {
            texture,
            scale,
            source_rec,
            dest_rec,
            centered,
            origin: Vector2::zero(),
        };

        sprite.update_origin();
        sprite
    }

    fn get_recs(texture: &Rc<RefCell<WeakTexture2D>>, scale: f32) -> (Rectangle, Rectangle) {
        let tex = texture.as_ref().borrow();
        let source_rec = Rectangle::new(0.0, 0.0, tex.width as f32, tex.height as f32);
        let dest_rec = Rectangle::new(
            0.0,
            0.0,
            scale * tex.width as f32,
            scale * tex.height as f32,
        );

        (source_rec, dest_rec)
    }

    pub fn set_texture(&mut self, texture: Rc<RefCell<WeakTexture2D>>) {
        {
            let tex = texture.as_ref().borrow();
            self.source_rec = Rectangle::new(0.0, 0.0, tex.width as f32, tex.height as f32);
            self.dest_rec = Rectangle::new(
                0.0,
                0.0,
                self.scale * tex.width as f32,
                self.scale * tex.height as f32,
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
        {
            let tex = self.texture.as_ref().borrow();
            self.scale = scale;
            self.dest_rec.width = self.scale * tex.width as f32;
            self.dest_rec.height = self.scale * tex.height as f32;
        }
        self.update_origin();
    }

    fn update_origin(&mut self) {
        let tex = self.texture.as_ref().borrow();
        if self.centered {
            self.origin.x = tex.width as f32 * self.scale * 0.5;
            self.origin.y = tex.height as f32 * self.scale * 0.5;
        } else {
            self.origin.x = 0.0;
            self.origin.y = 0.0;
        }
    }

    pub fn draw(&self, rl: &mut RaylibMode2D<RaylibDrawHandle>, transform: &Transform2D) {
        let tex = &*self.texture.borrow();
        let mut dest_rec = self.dest_rec;
        dest_rec.x = transform.position.x;
        dest_rec.y = transform.position.y;

        rl.draw_texture_pro(
            tex,
            self.source_rec,
            dest_rec,
            self.origin,
            RAD2DEG as f32 * transform.rotation,
            Color::WHITE,
        );
    }
}
