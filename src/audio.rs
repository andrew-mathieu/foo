use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::synth::SynthEngine;

pub fn test_audio(synth: Arc<Mutex<SynthEngine>>) -> cpal::Stream {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("Aucun périphérique audio trouvé");

    let config = device
        .default_output_config()
        .expect("Impossible de récupérer la configuration audio");

    println!("Sample rate : {} Hz", config.sample_rate());
    println!("Canaux : {}", config.channels());
    println!("Format : {:?}", config.sample_format());

    println!("Création du stream...");

    let stream = device
        .build_output_stream(
            config.into(),
            move |data: &mut [f32], _info| {
                let mut synth = synth.lock().unwrap();

                synth.process(data);
            },
            |err| {
                eprintln!("Erreur audio : {err}");
            },
            None,
        )
        .expect("Impossible de créer le flux audio");

    println!("Stream créé !");

    stream.play().expect("Impossible de démarrer le flux audio");

    println!("Stream démarré !");

    stream
}
