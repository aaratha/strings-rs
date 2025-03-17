use crate::constants::*;
use macroquad::prelude::*;

pub struct String {
    start: Vec2,
    end: Vec2,
    points: Vec<Vec2>,
    prev_points: Vec<Vec2>, // Previous positions for Verlet integration
    rest_length: f32,
    thickness: f32,
    color: Color,
}

impl String {
    pub fn new(
        start: Vec2,
        end: Vec2,
        segments: usize,
        elasticity: f32,
        thickness: f32,
        color: Color,
    ) -> Self {
        let mut points = vec![start];
        let mut prev_points = vec![start]; // Initialize previous positions

        for i in 1..segments {
            let t = i as f32 / segments as f32;
            let pos = start.lerp(end, t);
            points.push(pos);
            prev_points.push(pos);
        }

        points.push(end);
        prev_points.push(end);

        let rest_length = start.distance(end) / segments as f32;
        // Apply elasticity factor
        let rest_length = rest_length * elasticity;

        Self {
            start,
            end,
            points,
            prev_points,
            rest_length,
            thickness,
            color,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Verlet integration: update points based on their previous positions
        for i in 1..self.points.len() - 1 {
            let temp = self.points[i];
            let velocity = self.points[i] - self.prev_points[i];
            self.points[i] += velocity * dt * PHYSICS_MULTIPLIER; // Gravity factor
            self.prev_points[i] = temp;
        }

        // Constraint relaxation (stick physics)
        for _ in 0..5 {
            for i in 0..self.points.len() - 1 {
                let dir = self.points[i + 1] - self.points[i];
                let length = dir.length();
                let diff = (length - self.rest_length) / length;
                let offset = dir * 0.5 * diff;

                if i > 0 {
                    self.points[i] += offset;
                }
                if i < self.points.len() - 2 {
                    self.points[i + 1] -= offset;
                }
            }

            // Keep fixed endpoints intact
            self.points[0] = self.start;
            let last_index = self.points.len() - 1;
            self.points[last_index] = self.end;
        }

        // Mouse interaction: allow grabbing points if the mouse is close enough
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_pos = vec2(mouse_position().0, mouse_position().1);
            for i in 1..self.points.len() - 1 {
                if (mouse_pos - self.points[i]).length() < STRING_GRAB_DISTANCE {
                    self.points[i] = mouse_pos;
                }
            }
        }
    }

    pub fn draw(&self) {
        for i in 0..self.points.len() - 1 {
            draw_line(
                self.points[i].x,
                self.points[i].y,
                self.points[i + 1].x,
                self.points[i + 1].y,
                self.thickness,
                self.color,
            );
        }
    }
}
