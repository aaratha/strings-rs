use crate::constants::*;

#[derive(Debug)]
pub struct Clock {
    pub beat_time: f32,
    pub beat_duration: f32,
}

impl Clock {
    pub fn new(bpm: f32) -> Self {
        let beat_duration = 60.0 / CLOCK_BPM;
        Self {
            beat_time: 0.0,
            beat_duration,
        }
    }

    // Update the clock with a time delta.
    pub fn update(&mut self, dt: f32) {
        self.beat_time += dt;
        if self.beat_time >= self.beat_duration {
            self.beat_time -= self.beat_duration;
        }
    }

    // Return the current beat position (0.0 to 1.0).
    pub fn get_beat(&self) -> f32 {
        self.beat_time / self.beat_duration
    }
}
