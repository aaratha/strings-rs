use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use macroquad::prelude::*;
use std::f32::consts::TAU;
use std::sync::{Arc, Mutex};

const STRING_COUNT: usize = 5;
const GRAB_DISTANCE: f32 = 20.0;

// string object
struct String {
    start: Vec2,
    end: Vec2,
    points: Vec<Vec2>,
    prev_points: Vec<Vec2>, // Stores previous positions for Verlet integration
    rest_length: f32,
    thickness: f32,
    color: Color,
}

impl String {
    fn new(
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
        // apply elasticity
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

    fn update(&mut self, dt: f32) {
        // Verlet integration: move points based on previous positions
        for i in 1..self.points.len() - 1 {
            let temp = self.points[i];
            let velocity = self.points[i] - self.prev_points[i];
            self.points[i] += velocity; // Apply gravity
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

            // Ensure fixed points remain unchanged
            self.points[0] = self.start;
            let last_index = self.points.len() - 1;
            self.points[last_index] = self.end;
        }

        // Mouse interaction
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_pos = vec2(mouse_position().0, mouse_position().1);
            for i in 1..self.points.len() - 1 {
                if (mouse_pos - self.points[i]).length() < GRAB_DISTANCE {
                    self.points[i] = mouse_pos;
                }
            }
        }
    }

    fn draw(&self) {
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

fn set_freq(freq: f32, phase_step: &Arc<Mutex<f32>>, sample_rate: f32) {
    *phase_step.lock().unwrap() = freq * TAU / sample_rate;
}

#[macroquad::main("Realtime Synth")]
async fn main() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    let config = device.default_output_config().unwrap();

    let sample_rate = config.sample_rate().0 as f32;

    // Shared state for phase and phase_step (frequency control)
    let phase = Arc::new(Mutex::new(0.0_f32));
    let phase_step = Arc::new(Mutex::new(440.0 * TAU / sample_rate));

    let phase_clone = Arc::clone(&phase);
    let phase_step_clone = Arc::clone(&phase_step);

    // Provide `None` as the last argument for build_output_stream
    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut phase = phase_clone.lock().unwrap();
                let phase_step = *phase_step_clone.lock().unwrap();
                for sample in data.iter_mut() {
                    *sample = (phase.sin() * 0.2) as f32;
                    *phase = (*phase + phase_step) % TAU;
                }
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )
        .unwrap();

    stream.play().unwrap();

    let center = vec2(screen_width() / 2.0, screen_height() / 2.0);

    let mut strings: Vec<String> = Vec::new();

    let gap: f32 = 60.0;
    let length: f32 = 300.0;

    for i in 0..5 {
        let x_pos: f32 = (i - 2) as f32 * gap;
        strings.push(String::new(
            center + vec2(x_pos, length / 2.0),
            center + vec2(x_pos, -length / 2.0),
            20,
            0.5,
            2.0,
            GRAY,
        ));
    }

    loop {
        // Update string
        clear_background(WHITE);
        let (mouse_x, mouse_y) = mouse_position();
        // Map mouse_x to a frequency range (200Hz to 1000Hz)
        let freq = 200.0 + (mouse_x / screen_width()) * 800.0;
        set_freq(freq, &phase_step, sample_rate);
        draw_circle(mouse_x, mouse_y, 20.0, GREEN);
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 20.0, 20.0, BLACK);

        // draw all stirngs
        for string in &mut strings {
            string.update(get_frame_time());
            string.draw();
        }

        next_frame().await;
    }
}
