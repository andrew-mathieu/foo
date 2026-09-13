#[derive(Clone, Copy, PartialEq)]
enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Adsr {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,

    sample_rate: f32,

    stage: EnvelopeStage,
    level: f32,
}

impl Adsr {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            attack: 0.01,
            decay: 0.15,
            sustain: 0.8,
            release: 0.3,

            sample_rate,

            stage: EnvelopeStage::Idle,
            level: 0.0,
        }
    }

    pub fn note_on(&mut self) {
        self.stage = EnvelopeStage::Attack;
    }

    pub fn note_off(&mut self) {
        self.stage = EnvelopeStage::Release;
    }

    pub fn next_sample(&mut self) -> f32 {
        match self.stage {
            EnvelopeStage::Idle => {
                self.level = 0.0;
            }

            EnvelopeStage::Attack => {
                let increment = 1.0 / (self.attack * self.sample_rate);

                self.level += increment;

                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
            }

            EnvelopeStage::Decay => {
                let decrement = (1.0 - self.sustain) / (self.decay * self.sample_rate);

                self.level -= decrement;

                if self.level <= self.sustain {
                    self.level = self.sustain;
                    self.stage = EnvelopeStage::Sustain;
                }
            }

            EnvelopeStage::Sustain => {
                self.level = self.sustain;
            }

            EnvelopeStage::Release => {
                let decrement = self.sustain / (self.release * self.sample_rate);

                self.level -= decrement;

                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
            }
        }

        self.level
    }

    pub fn is_finished(&self) -> bool {
        self.stage == EnvelopeStage::Idle
    }
}
