// audio.rs

use crate::clock::Clock;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::TAU;
use std::sync::{Arc, Mutex};

pub struct AudioState {
    // Shared audio state.
    pub phase: Arc<Mutex<f32>>,
    pub phase_step: Arc<Mutex<f32>>,
    pub sample_rate: f32,
    // We store the stream to keep it alive.
    _stream: cpal::Stream,
}

pub fn init_audio(clock: Arc<Mutex<Clock>>) -> AudioState {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available");
    let config = device.default_output_config().unwrap();
    let sample_rate = config.sample_rate().0 as f32;

    // Shared state for the oscillator.
    let phase = Arc::new(Mutex::new(0.0_f32));
    let phase_step = Arc::new(Mutex::new(440.0 * TAU / sample_rate));

    let phase_clone = Arc::clone(&phase);
    let phase_step_clone = Arc::clone(&phase_step);
    let clock_clone = Arc::clone(&clock);

    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let dt_sample = 1.0 / sample_rate;
                // Obtain the current beat duration and beat time.
                let (beat_duration, mut local_beat_time) = {
                    let clock = clock_clone.lock().unwrap();
                    (clock.beat_duration, clock.beat_time)
                };

                // Process each audio sample.
                for sample in data.iter_mut() {
                    let beat_position = local_beat_time / beat_duration;
                    let is_playing = beat_position < 0.5; // Tone plays in first half of beat.

                    let phase_step_value = *phase_step_clone.lock().unwrap();
                    let mut phase_val = phase_clone.lock().unwrap();

                    *sample = if is_playing {
                        phase_val.sin() * 0.2
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
                // Write the updated beat time back into the clock.
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

    AudioState {
        phase,
        phase_step,
        sample_rate,
        _stream: stream,
    }
}

// Update the phase step based on the given frequency.
pub fn set_frequency(freq: f32, phase_step: &Arc<Mutex<f32>>, sample_rate: f32) {
    *phase_step.lock().unwrap() = freq * TAU / sample_rate;
}
