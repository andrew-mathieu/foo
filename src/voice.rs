use crate::envelope::Adsr;
use crate::note::Note;
use crate::oscillator::{Oscillator, Waveform};

pub struct Voice {
    note: Note,
    oscillator: Oscillator,
    envelope: Adsr,
}

impl Voice {
    pub fn new(note: Note, sample_rate: f32, waveform: Waveform) -> Self {
        Self {
            note,

            oscillator: Oscillator::new(note.frequency(), sample_rate, waveform),

            envelope: Adsr::new(sample_rate),
        }
    }

    pub fn note_on(&mut self) {
        self.envelope.note_on();
    }

    pub fn note_off(&mut self) {
        self.envelope.note_off();
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        self.oscillator.process(buffer);

        for sample in buffer.iter_mut() {
            let envelope = self.envelope.next_sample();

            *sample *= envelope;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    pub fn note(&self) -> Note {
        self.note
    }
}
