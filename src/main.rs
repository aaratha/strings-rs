// main.rs

use macroquad::prelude::*;
use std::sync::{Arc, Mutex};

mod audio;
mod clock;
mod constants;
mod delay;
mod karplus_strong;
mod string;
mod visuals;

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
        // Map mouse x-position to a frequency between 110 Hz and 880 Hz.
        let (mouse_x, _) = mouse_position();
        let min_freq = 110.0;
        let max_freq = 880.0;
        let freq = min_freq + (mouse_x / screen_width()) * (max_freq - min_freq);

        // Update the Karplus–Strong string's frequency.
        {
            let mut ks = audio_state.ks.lock().unwrap();
        }

        // On mouse click, pluck the string.
        for string in &mut visuals_state.strings {
            if string.plucked {
                let mut ks = audio_state.ks.lock().unwrap();
                ks.set_frequency(string.freq, audio_state.sample_rate);
                ks.pluck();
                string.plucked = false;
            }
        }
        update_visuals(&mut visuals_state).await;
    }
}
