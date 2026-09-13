use crate::note::Note;
use crate::oscillator::Waveform;
use crate::voice::Voice;

pub struct SynthEngine {
    voices: Vec<Voice>,
    sample_rate: f32,
    waveform: Waveform,
}

impl SynthEngine {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            voices: Vec::new(),
            sample_rate,
            waveform: Waveform::Saw,
        }
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn note_on(&mut self, note: Note) {
        let voice = Voice::new(note, self.sample_rate, self.waveform);

        self.voices.push(voice);
    }

    pub fn note_off(&mut self, note: Note) {
        for voice in &mut self.voices {
            if voice.note() == note {
                voice.note_off();
            }
        }
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        buffer.fill(0.0);

        self.voices.retain(|voice| !voice.is_finished());
        let mut voice_buffer = vec![0.0f32; buffer.len()];

        for voice in &mut self.voices {
            voice_buffer.fill(0.0);

            voice.process(&mut voice_buffer);

            for (output, voice_sample) in buffer.iter_mut().zip(voice_buffer.iter()) {
                *output += *voice_sample;
            }
        }
    }
}
