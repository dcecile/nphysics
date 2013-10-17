use rsfml::graphics::render_window;
use rsfml::graphics::circle_shape::CircleShape;
use rsfml::graphics::Color;
use rsfml::system::vector2;
use nalgebra::na::Vec3;
use nalgebra::na;
use draw_helper::DRAW_SCALE;
use nphysics::aliases::dim2;

struct Ball<'self> {
    priv color: Vec3<u8>,
    priv delta: dim2::Transform2d<f32>,
    priv body:  @mut dim2::Body2d<f32>,
    priv gfx:   CircleShape<'self>
}

impl<'self> Ball<'self> {
    pub fn new(body:   @mut dim2::Body2d<f32>,
               delta:  dim2::Transform2d<f32>,
               radius: f32,
               color:  Vec3<u8>) -> Ball {
        let dradius = radius as f32 * DRAW_SCALE;

        let mut res = Ball {
            color: color,
            delta: delta,
            gfx:   CircleShape::new().unwrap(),
            body:  body
        };

        res.gfx.set_fill_color(&Color::new_RGB(color.x, color.y, color.z));
        res.gfx.set_radius(dradius);
        res.gfx.set_origin(&vector2::Vector2f { x: dradius, y: dradius }); 

        res
    }
}

impl<'self> Ball<'self> {
    pub fn update(&mut self) {
        let body = self.body.to_rigid_body_or_fail();
        let transform = body.transform_ref() * self.delta;
        let pos = na::translation(&transform);
        let rot = na::rotation(&transform);

        self.gfx.set_position(&vector2::Vector2f {
            x: pos.x as f32 * DRAW_SCALE,
            y: pos.y as f32 * DRAW_SCALE
        });
        self.gfx.set_rotation(rot.x.to_degrees() as f32);

        if body.is_active() {
            self.gfx.set_fill_color(
                &Color::new_RGB(self.color.x, self.color.y, self.color.z));
        }
        else {
            self.gfx.set_fill_color(
                &Color::new_RGB(self.color.x / 4, self.color.y / 4, self.color.z / 4));
        }
    }

    pub fn draw(&self, rw: &mut render_window::RenderWindow) {
        rw.draw(&self.gfx);
    }
}
