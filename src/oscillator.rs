#[derive(Clone, Copy)]
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
}

pub struct Oscillator {
    frequency: f32,
    sample_rate: f32,
    phase: f32,
    waveform: Waveform,
    triangle: f32,
}

impl Oscillator {
    pub fn new(frequency: f32, sample_rate: f32, waveform: Waveform) -> Self {
        Self {
            frequency,
            sample_rate,
            phase: 0.0,
            waveform,
            triangle: -1.0,
        }
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.generate_sample();

            self.phase += self.frequency / self.sample_rate;

            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
    }

    fn generate_sample(&mut self) -> f32 {
        let dt = self.frequency / self.sample_rate;

        match self.waveform {
            Waveform::Sine => (2.0 * std::f32::consts::PI * self.phase).sin(),

            Waveform::Saw => 2.0 * self.phase - 1.0 - poly_blep(self.phase, dt),

            Waveform::Square => {
                let mut value = if self.phase < 0.5 { 1.0 } else { -1.0 };

                value += poly_blep(self.phase, dt);

                value -= poly_blep((self.phase + 0.5) % 1.0, dt);

                value
            }

            Waveform::Triangle => {
                let square = if self.phase < 0.5 { 1.0 } else { -1.0 };

                let square =
                    square + poly_blep(self.phase, dt) - poly_blep((self.phase + 0.5) % 1.0, dt);

                self.triangle += 4.0 * dt * square;

                self.triangle
            }
        }
    }
}

fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;

        return t + t - t * t - 1.0;
    }

    if t > 1.0 - dt {
        let t = (t - 1.0) / dt;

        return t * t + t + t + 1.0;
    }

    0.0
}
