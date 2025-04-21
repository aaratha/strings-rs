// visuals.rs

use crate::clock::Clock;
use crate::constants::*;
use crate::string::String;
use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

pub struct VisualsState {
    pub strings: Vec<String>,
    // Shared clock for visual updates.
    pub clock: Arc<Mutex<Clock>>,
}

pub fn init_visuals() -> VisualsState {
    // Create the shared clock.
    let clock = Arc::new(Mutex::new(Clock::new(120.0)));
    // Use the current screen dimensions to center the visuals.
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
            STRING_FREQUENCIES[i],
            GRAY,
        ));
    }

    VisualsState { strings, clock }
}

pub async fn update_visuals(state: &mut VisualsState) {
    clear_background(WHITE);
    let (mouse_x, mouse_y) = mouse_position();

    // Draw simple visuals.
    let b = 0.2;
    let gray = Color::new(b, b, b, 0.3);
    draw_circle(mouse_x, mouse_y, 20.0, gray);
    draw_text(&format!("FPS: {}", get_fps()), 10.0, 20.0, 20.0, BLACK);

    // Update the clock (for visuals).
    {
        let mut clock = state.clock.lock().unwrap();
        clock.update(get_frame_time());
    }

    // Update and draw all strings.
    for string in &mut state.strings {
        string.update(get_frame_time());
        string.draw();
    }

    next_frame().await;
}
