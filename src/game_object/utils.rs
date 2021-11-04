#[macro_export]
macro_rules! impl_spatial {
    ($x:tt) => {
        impl Spatial for $x {
            fn get_position(&self) -> Vector2 {
                self.game_object.get_position()
            }
            fn set_position(&mut self, position: Vector2) {
                self.game_object.set_position(position);
            }
            fn get_rotation(&self) -> f32 {
                self.game_object.get_rotation()
            }
            fn set_rotation(&mut self, rotation: f32) {
                self.game_object.set_rotation(rotation);
            }

            fn translate(&mut self, vector: Vector2) {
                self.set_position(self.get_position() + vector);
            }
        }
    };
}

#[macro_export]
macro_rules! impl_drawable {
    ($x:tt) => {
        impl Drawable for $x {
            fn draw(
                &self,
                rl: &mut DrawHandle!(),
            ) {
                self.game_object.draw(rl);
            }

            fn get_scale(&self) -> f32 {
                self.game_object.get_scale()
            }

            fn set_scale(&mut self, scale: f32) {
                self.game_object.set_scale(scale);
            }

            fn set_tint(&mut self, tint: Color) {
                self.game_object.set_tint(tint);
            }

            fn get_transform(&self) -> Transform2D {
                self.game_object.get_transform()
            }
        }
    };
}
