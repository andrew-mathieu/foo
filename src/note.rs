#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Note {
    pub midi: u8,
}

impl Note {
    pub fn frequency(&self) -> f32 {
        440.0 * 2.0_f32.powf((self.midi as f32 - 69.0) / 12.0)
    }
}
