mod audio;
mod envelope;
mod gui;
mod note;
mod oscillator;
mod synth;
mod voice;
use eframe::egui;

use std::sync::{Arc, Mutex};

use crate::synth::SynthEngine;

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "host-grotesk".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/HostGrotesk-VariableFont_wght.ttf"
        ))
        .into(),
    );

    fonts.families.insert(
        egui::FontFamily::Proportional,
        vec!["host-grotesk".to_owned()],
    );

    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result<()> {
    let synth = Arc::new(Mutex::new(SynthEngine::new(48_000.0)));

    let _audio_stream = audio::test_audio(Arc::clone(&synth));

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Rust Synth",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);

            Ok(Box::new(gui::SynthApp::new(Arc::clone(&synth))))
        }),
    )
}
