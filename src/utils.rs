use macroquad::math::Vec2;

pub fn lerp1d(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn lerp2d(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    Vec2::new(lerp1d(a.x, b.x, t), lerp1d(a.y, b.y, t))
}
