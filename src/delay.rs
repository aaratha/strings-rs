// delay.rs

pub struct Delay {
    buffer: Vec<f32>,
    write_index: usize,
    delay_samples: usize,
    feedback: f32,
    mix: f32,
}

impl Delay {
    /// Create a new Delay.
    /// - `delay_time` is in seconds.
    /// - `sample_rate` is the number of samples per second.
    /// - `feedback` is the amount of the delayed signal fed back (0.0 to 1.0).
    /// - `mix` determines the balance between dry (original) and wet (delayed) signal.
    pub fn new(delay_time: f32, sample_rate: f32, feedback: f32, mix: f32) -> Self {
        let delay_samples = (delay_time * sample_rate).round() as usize;
        // The buffer length is delay_samples + 1 to handle wrap-around.
        let buffer_length = delay_samples + 1;
        Self {
            buffer: vec![0.0; buffer_length],
            write_index: 0,
            delay_samples,
            feedback,
            mix,
        }
    }

    /// Process an input sample and return the delayed output.
    pub fn process(&mut self, input: f32) -> f32 {
        let buffer_length = self.buffer.len();
        // Calculate the read index for the delayed sample.
        let read_index = (self.write_index + buffer_length - self.delay_samples) % buffer_length;
        let delayed_sample = self.buffer[read_index];

        // Write the current input plus feedback from the delayed sample into the buffer.
        self.buffer[self.write_index] = input + delayed_sample * self.feedback;

        // Increment the write index, wrapping around the buffer.
        self.write_index = (self.write_index + 1) % buffer_length;

        // Output a mix of the dry input and the wet (delayed) signal.
        (1.0 - self.mix) * input + self.mix * delayed_sample
    }

    pub fn set_delay_time(&mut self, delay_time: f32, sample_rate: f32) {
        self.delay_samples = (delay_time * sample_rate).round() as usize;
        let buffer_length = self.delay_samples + 1;
        self.buffer = vec![0.0; buffer_length];
        self.write_index = 0;
    }
}
