//! 提示音 — the tones a reminder can ring with, and how they are rendered.
//!
//! No audio files ship with the app. Each tone is a few hundred milliseconds
//! of synthesised PCM: 木鱼 is a damped click-and-body, 风铃 a long ringing
//! partial pair, 滴 a plain short beep. That keeps the bundle free of
//! licensing questions and lets the same code serve every platform.

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SoundTone {
    None,
    #[default]
    Woodblock,
    Chime,
    Beep,
}

impl SoundTone {
    pub const ALL: [SoundTone; 4] = [
        SoundTone::None,
        SoundTone::Woodblock,
        SoundTone::Chime,
        SoundTone::Beep,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SoundTone::None => "无",
            SoundTone::Woodblock => "木鱼",
            SoundTone::Chime => "风铃",
            SoundTone::Beep => "滴",
        }
    }

    fn from_label(label: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.label() == label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundSetting {
    pub tone: SoundTone,
    /// 0–100.
    pub volume: u8,
}

impl Default for SoundSetting {
    fn default() -> Self {
        Self {
            tone: SoundTone::Woodblock,
            volume: 30,
        }
    }
}

impl SoundSetting {
    /// The rule row's text, e.g. 木鱼 · 30%.
    pub fn label(self) -> String {
        match self.tone {
            SoundTone::None => "无".to_string(),
            tone => format!("{} · {}%", tone.label(), self.volume),
        }
    }

    /// The pre-struct form was exactly `label()`'s output stored as a string.
    pub fn parse_legacy(text: &str) -> Option<Self> {
        let text = text.trim();
        if text == "无" {
            return Some(Self {
                tone: SoundTone::None,
                volume: 0,
            });
        }
        let (name, pct) = text.split_once(" · ")?;
        let tone = SoundTone::from_label(name)?;
        let volume = pct.trim_end_matches('%').trim().parse::<u8>().ok()?;
        Some(Self {
            tone,
            volume: volume.min(100),
        })
    }

    pub fn clamped(self) -> Self {
        Self {
            tone: self.tone,
            volume: self.volume.min(100),
        }
    }
}

/// Accept either the struct or the old `"木鱼 · 30%"` string; anything
/// unreadable falls back to the default rather than refusing the whole file.
pub fn deserialize_sound<'de, D: Deserializer<'de>>(d: D) -> Result<SoundSetting, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Setting(SoundSetting),
        Legacy(String),
    }
    Ok(match Raw::deserialize(d) {
        Ok(Raw::Setting(s)) => s.clamped(),
        Ok(Raw::Legacy(text)) => SoundSetting::parse_legacy(&text).unwrap_or_default(),
        Err(_) => SoundSetting::default(),
    })
}

pub const SAMPLE_RATE: u32 = 44_100;

/// Render the tone as mono f32 samples at `SAMPLE_RATE`, already scaled by
/// volume. `None` (or volume 0) renders nothing, so callers can skip playback.
pub fn render(setting: SoundSetting) -> Vec<f32> {
    let setting = setting.clamped();
    if setting.tone == SoundTone::None || setting.volume == 0 {
        return Vec::new();
    }
    // 100% is still comfortably under clipping.
    let gain = setting.volume as f32 / 100.0 * 0.8;
    let (secs, partials, decay): (f32, &[(f32, f32)], f32) = match setting.tone {
        SoundTone::Woodblock => (0.22, &[(880.0, 1.0), (1320.0, 0.35), (2200.0, 0.15)], 28.0),
        SoundTone::Chime => (1.1, &[(1760.0, 1.0), (2640.0, 0.5), (4400.0, 0.2)], 5.0),
        SoundTone::Beep => (0.14, &[(1000.0, 1.0)], 10.0),
        SoundTone::None => unreachable!(),
    };
    let n = (secs * SAMPLE_RATE as f32) as usize;
    let attack = (0.004 * SAMPLE_RATE as f32).max(1.0);
    // A short linear release so the short tones do not end on a click.
    let release = (0.02 * SAMPLE_RATE as f32).max(1.0);
    let norm: f32 = partials.iter().map(|p| p.1).sum();
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = (t * decay).exp().recip()
                * (i as f32 / attack).min(1.0)
                * ((n - i) as f32 / release).min(1.0);
            let body: f32 = partials
                .iter()
                .map(|(f, a)| a * (2.0 * std::f32::consts::PI * f * t).sin())
                .sum::<f32>()
                / norm;
            // The woodblock's "knock": a short burst of noise-like harmonics.
            let click = if setting.tone == SoundTone::Woodblock && t < 0.006 {
                0.5 * ((t * 24_000.0).sin() * (t * 17_000.0).cos())
            } else {
                0.0
            };
            gain * env * (body + click)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_is_the_specs_woodblock_at_thirty_percent() {
        assert_eq!(SoundSetting::default().label(), "木鱼 · 30%");
    }

    #[test]
    fn the_legacy_string_round_trips_through_the_label() {
        for tone in SoundTone::ALL {
            let s = SoundSetting { tone, volume: 45 };
            let back = SoundSetting::parse_legacy(&s.label()).unwrap();
            assert_eq!(back.tone, tone);
            if tone != SoundTone::None {
                assert_eq!(back.volume, 45);
            }
        }
        assert_eq!(
            SoundSetting::parse_legacy("木鱼 · 30%"),
            Some(SoundSetting::default())
        );
        assert_eq!(SoundSetting::parse_legacy("gong · 30%"), None);
    }

    #[test]
    fn either_shape_deserialises_and_junk_falls_back() {
        #[derive(Deserialize)]
        struct Holder {
            #[serde(deserialize_with = "deserialize_sound")]
            sound: SoundSetting,
        }
        let legacy: Holder = serde_json::from_str(r#"{"sound":"风铃 · 60%"}"#).unwrap();
        assert_eq!(legacy.sound.tone, SoundTone::Chime);
        assert_eq!(legacy.sound.volume, 60);
        let modern: Holder =
            serde_json::from_str(r#"{"sound":{"tone":"beep","volume":250}}"#).unwrap();
        assert_eq!(modern.sound.tone, SoundTone::Beep);
        assert_eq!(modern.sound.volume, 100);
        let junk: Holder = serde_json::from_str(r#"{"sound":42}"#).unwrap();
        assert_eq!(junk.sound, SoundSetting::default());
    }

    #[test]
    fn none_and_zero_volume_render_nothing() {
        assert!(render(SoundSetting {
            tone: SoundTone::None,
            volume: 80
        })
        .is_empty());
        assert!(render(SoundSetting {
            tone: SoundTone::Chime,
            volume: 0
        })
        .is_empty());
    }

    #[test]
    fn tones_render_bounded_samples_that_scale_with_volume() {
        for tone in [SoundTone::Woodblock, SoundTone::Chime, SoundTone::Beep] {
            let loud = render(SoundSetting { tone, volume: 100 });
            let quiet = render(SoundSetting { tone, volume: 30 });
            assert!(!loud.is_empty());
            assert_eq!(loud.len(), quiet.len());
            let peak = |v: &[f32]| v.iter().fold(0f32, |m, s| m.max(s.abs()));
            assert!(peak(&loud) <= 1.0, "{tone:?} clips");
            assert!(peak(&loud) > 0.2, "{tone:?} is inaudible");
            assert!(peak(&quiet) < peak(&loud));
            // Every tone decays to (near) silence by its end.
            let tail = peak(&loud[loud.len() - 200..]);
            assert!(tail < 0.05, "{tone:?} does not decay: {tail}");
        }
    }
}
