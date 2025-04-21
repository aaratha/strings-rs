// karplus_strong.rs

pub struct KarplusStrong {
    buffer: Vec<f32>,
    read_index: usize,
    decay: f32,
    initialized: bool,
}

impl KarplusStrong {
    /// Create a new Karplus–Strong string.
    /// `buffer_size` is the length of the delay line (in samples).
    /// `decay` is the feedback decay factor (usually slightly below 1.0).
    pub fn new(buffer_size: usize, decay: f32) -> Self {
        Self {
            buffer: vec![0.0; buffer_size],
            read_index: 0,
            decay,
            initialized: false,
        }
    }

    /// Pluck the string by filling the buffer with random noise.
    pub fn pluck(&mut self) {
        for sample in self.buffer.iter_mut() {
            *sample = macroquad::rand::gen_range(-1.0, 1.0);
        }
        self.initialized = true;
    }

    /// Process one sample using the Karplus–Strong algorithm.
    /// Returns the next output sample.
    pub fn process(&mut self) -> f32 {
        if !self.initialized || self.buffer.is_empty() {
            return 0.0;
        }
        // Get the current sample and the next sample (with wrap-around)
        let current = self.buffer[self.read_index];
        let next_index = (self.read_index + 1) % self.buffer.len();
        let next_sample = self.buffer[next_index];

        // Compute the new sample with a simple averaging filter and decay.
        let new_sample = self.decay * 0.5 * (current + next_sample);

        // Replace the current sample with the new computed value.
        self.buffer[self.read_index] = new_sample;

        // Advance the read pointer.
        self.read_index = next_index;

        // Return the original sample (the “pluck” sound)
        current
    }

    /// Update the buffer size based on a new frequency and sample rate.
    /// This function reinitializes the buffer (and resets the read pointer).
    /// (You may choose to retain buffer data for smooth transitions.)
    pub fn set_frequency(&mut self, frequency: f32, sample_rate: f32) {
        let new_size = (sample_rate / frequency).max(2.0) as usize;
        if new_size != self.buffer.len() {
            self.buffer = vec![0.0; new_size];
            self.read_index = 0;
            self.initialized = false;
        }
    }
}
