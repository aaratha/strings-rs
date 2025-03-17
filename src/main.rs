use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use macroquad::prelude::*;
use std::f32::consts::TAU;
use std::sync::{Arc, Mutex};

const STRING_COUNT: usize = 5;
const STRING_GRAB_DISTANCE: f32 = 20.0;
const STRING_ELASTICITY: f32 = 0.4;
const STRING_GAP: f32 = 60.0;
const STRING_LENGTH: f32 = 300.0;
const STRING_POINT_COUNT: usize = 20;
const STRING_THICKNESS: f32 = 2.0;
const PHYSICS_MULTIPLIER: f32 = 160.0;

// String object for our visual simulation.
struct String {
    start: Vec2,
    end: Vec2,
    points: Vec<Vec2>,
    prev_points: Vec<Vec2>, // Previous positions for Verlet integration
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

    fn update(&mut self, dt: f32) {
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

// Clock for managing the beat cycle.
#[derive(Debug)]
struct Clock {
    beat_time: f32,
    beat_duration: f32,
}

impl Clock {
    fn new(bpm: f32) -> Self {
        let beat_duration = 60.0 / bpm;
        Self {
            beat_time: 0.0,
            beat_duration,
        }
    }

    // Update the clock with a time delta.
    fn update(&mut self, dt: f32) {
        self.beat_time += dt;
        if self.beat_time >= self.beat_duration {
            self.beat_time -= self.beat_duration;
        }
    }

    // Return the current beat position (0.0 to 1.0).
    fn get_beat(&self) -> f32 {
        self.beat_time / self.beat_duration
    }
}

// Helper to update the phase step based on frequency.
fn set_freq(freq: f32, phase_step: &Arc<Mutex<f32>>, sample_rate: f32) {
    *phase_step.lock().unwrap() = freq * TAU / sample_rate;
}

#[macroquad::main("Realtime Synth")]
async fn main() {
    // Audio setup with CPAL.
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    let config = device.default_output_config().unwrap();
    let sample_rate = config.sample_rate().0 as f32;

    // Shared state for phase and phase_step.
    let phase = Arc::new(Mutex::new(0.0_f32));
    let phase_step = Arc::new(Mutex::new(440.0 * TAU / sample_rate));

    let phase_clone = Arc::clone(&phase);
    let phase_step_clone = Arc::clone(&phase_step);

    // Shared clock using Arc<Mutex<>>.
    let clock = Arc::new(Mutex::new(Clock::new(120.0)));
    let clock_clone = Arc::clone(&clock);

    // Build the audio stream.
    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let dt_sample = 1.0 / sample_rate;

                // Lock once to get clock state.
                let (beat_duration, mut local_beat_time) = {
                    let clock = clock_clone.lock().unwrap();
                    (clock.beat_duration, clock.beat_time)
                };

                // Process each sample.
                for sample in data.iter_mut() {
                    let beat_position = local_beat_time / beat_duration;
                    let is_playing = beat_position < 0.5; // Tone plays in first half of beat.

                    // Lock phase and phase_step once per sample.
                    let phase_step_value = *phase_step_clone.lock().unwrap();
                    let mut phase_val = phase_clone.lock().unwrap();

                    *sample = if is_playing {
                        (phase_val.sin() * 0.2) as f32
                    } else {
                        0.0
                    };
                    *phase_val = (*phase_val + phase_step_value) % TAU;

                    // Update local beat time.
                    local_beat_time += dt_sample;
                    if local_beat_time >= beat_duration {
                        local_beat_time -= beat_duration;
                    }
                }

                // Write the updated clock state back with a single lock.
                {
                    let mut clock = clock_clone.lock().unwrap();
                    clock.beat_time = local_beat_time;
                }
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )
        .unwrap();

    stream.play().unwrap();

    // Visuals setup with macroquad.
    let center = vec2(screen_width() / 2.0, screen_height() / 2.0);
    let mut strings: Vec<String> = Vec::new();

    // Create a few strings.
    for i in 0..STRING_COUNT {
        let x_pos: f32 = (i as f32 - 2.0) * STRING_GAP;
        strings.push(String::new(
            center + vec2(x_pos, STRING_LENGTH / 2.0),
            center + vec2(x_pos, -STRING_LENGTH / 2.0),
            STRING_POINT_COUNT,
            STRING_ELASTICITY,
            STRING_THICKNESS,
            GRAY,
        ));
    }

    // Main loop for visuals.
    loop {
        clear_background(WHITE);
        let (mouse_x, mouse_y) = mouse_position();

        // Map mouse x-position to a frequency between 200 Hz and 1000 Hz.
        let freq = 200.0 + (mouse_x / screen_width()) * 800.0;
        set_freq(freq, &phase_step, sample_rate);

        draw_circle(mouse_x, mouse_y, 20.0, GREEN);
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 20.0, 20.0, BLACK);

        // Update clock with frame time (for visuals, not audio).
        clock.lock().unwrap().update(get_frame_time());

        // Update and draw all strings.
        for string in &mut strings {
            string.update(get_frame_time());
            string.draw();
        }

        next_frame().await;
    }
}
