use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use macroquad::prelude::*;
use std::sync::{Arc, Mutex};
use std::f32::consts::TAU;

fn set_freq(freq: f32, phase_step: &Arc<Mutex<f32>>, sample_rate: f32) {
    *phase_step.lock().unwrap() = freq * TAU / sample_rate;
}

#[macroquad::main("Realtime Synth")]
async fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device available");
    let config = device.default_output_config().unwrap();

    let sample_rate = config.sample_rate().0 as f32;

    // Shared state for phase and phase_step (frequency control)
    let phase = Arc::new(Mutex::new(0.0_f32));
    let phase_step = Arc::new(Mutex::new(440.0 * TAU / sample_rate));

    let phase_clone = Arc::clone(&phase);
    let phase_step_clone = Arc::clone(&phase_step);

    // Provide `None` as the last argument for build_output_stream
    let stream = device.build_output_stream(
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
    ).unwrap();

    stream.play().unwrap();

    loop {
        clear_background(BLACK);
        let (mouse_x, mouse_y) = mouse_position();
        // Map mouse_x to a frequency range (200Hz to 1000Hz)
        let freq = 200.0 + (mouse_x / screen_width()) * 800.0;
        set_freq(freq, &phase_step, sample_rate);
        draw_circle(mouse_x, mouse_y, 20.0, RED);
        draw_text(&format!("FPS: {}", get_fps()), 10.0, 20.0, 20.0, WHITE);
        next_frame().await;
    }
}
