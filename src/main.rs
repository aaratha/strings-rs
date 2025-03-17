// main.rs

use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

mod audio;
mod visuals;
mod string;
mod clock;
mod constants;

use audio::{init_audio, set_frequency, AudioState};
use visuals::{init_visuals, update_visuals, VisualsState};

#[macroquad::main("Realtime Synth")]
async fn main() {
    // Initialize visuals state (which includes the shared clock).
    let mut visuals_state = init_visuals();
    // Use the same clock for audio.
    let clock = Arc::clone(&visuals_state.clock);
    let audio_state: AudioState = init_audio(clock);

    loop {
        // Map mouse x-position to a frequency between 200 Hz and 1000 Hz.
        let (mouse_x, _) = mouse_position();
        let freq = 200.0 + (mouse_x / screen_width()) * 800.0;
        set_frequency(freq, &audio_state.phase_step, audio_state.sample_rate);

        // Update visuals.
        update_visuals(&mut visuals_state).await;
    }
}
