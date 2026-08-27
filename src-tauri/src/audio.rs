//! Plays the synthesised reminder tones. Kept out of `core` because it talks
//! to the OS audio device; `core::sound` does the rendering and is pure.

use std::num::NonZero;
use std::time::Duration;

use crate::core::sound::{render, SoundSetting, SAMPLE_RATE};

/// Fire and forget. The device stream is opened per play and dropped once the
/// tone is over — a reminder rings a few times an hour, so holding a stream
/// open (and the audio device awake) in between is not worth it. Any failure
/// (no output device, exclusive-mode app) is silently ignored: a reminder
/// still shows, it just does not ring.
pub fn play(setting: SoundSetting) {
    let samples = render(setting);
    if samples.is_empty() {
        return;
    }
    std::thread::spawn(move || {
        let Ok(stream) = rodio::DeviceSinkBuilder::open_default_sink() else {
            return;
        };
        let secs = samples.len() as f32 / SAMPLE_RATE as f32;
        let buffer = rodio::buffer::SamplesBuffer::new(
            NonZero::new(1).expect("one channel"),
            NonZero::new(SAMPLE_RATE).expect("non-zero rate"),
            samples,
        );
        stream.mixer().add(buffer);
        std::thread::sleep(Duration::from_secs_f32(secs + 0.15));
    });
}
