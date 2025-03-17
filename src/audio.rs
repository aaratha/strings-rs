// audio.rs

use crate::clock::Clock;
use crate::karplus_strong::KarplusStrong;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::TAU;
use std::sync::{Arc, Mutex};

pub struct AudioState {
    // Shared Karplus–Strong instance.
    pub ks: Arc<Mutex<KarplusStrong>>,
    pub sample_rate: f32,
    // Keep the stream alive.
    _stream: cpal::Stream,
}

pub fn init_audio(clock: Arc<Mutex<Clock>>) -> AudioState {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    let config = device.default_output_config().unwrap();
    let sample_rate = config.sample_rate().0 as f32;

    // Initial frequency and parameters for the Karplus–Strong string.
    let frequency = 440.0; // A4 as a starting point
    let buffer_size = (sample_rate / frequency).max(2.0) as usize;
    let decay = 0.996; // Typical decay factor

    // Create and pluck the Karplus–Strong string.
    let ks = Arc::new(Mutex::new(KarplusStrong::new(buffer_size, decay)));
    ks.lock().unwrap().pluck();

    let ks_clone = Arc::clone(&ks);
    let clock_clone = Arc::clone(&clock);

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // For each sample in the output buffer, get the KS synth sample.
                for sample in data.iter_mut() {
                    *sample = {
                        let mut ks = ks_clone.lock().unwrap();
                        ks.process()
                    };
                }
                // (You can also use the clock here if you wish to drive timing.)
            },
            |err| eprintln!("Stream error: {}", err),
            None,
        )
        .unwrap();

    stream.play().unwrap();

    AudioState {
        ks,
        sample_rate,
        _stream: stream,
    }
}

// Update the phase step based on the given frequency.
pub fn set_frequency(freq: f32, phase_step: &Arc<Mutex<f32>>, sample_rate: f32) {
    *phase_step.lock().unwrap() = freq * TAU / sample_rate;
}
