use crate::engine::SOUND_VOLUME;
use std::sync::atomic::Ordering;
use std::thread;
use winapi::um::playsoundapi::PlaySoundW;

const SND_NODEFAULT: u32 = 0x0002;
const SND_MEMORY: u32 = 0x0004;

fn build_tone_wav(tones: &[(f32, f32)], volume: f32) -> Vec<u8> {
    const RATE: u32 = 44_100;
    let gain = volume.clamp(0.0, 1.0).powi(2) * 0.9;
    let mut samples: Vec<i16> = Vec::new();
    for &(freq, ms) in tones {
        let n = (RATE as f32 * ms / 1000.0) as usize;
        let fade = ((RATE as f32 * 0.008) as usize).max(1);
        for i in 0..n {
            let t = i as f32 / RATE as f32;
            let mut env = 1.0f32;
            if i < fade {
                env = i as f32 / fade as f32;
            } else if i + fade > n {
                env = (n - i) as f32 / fade as f32;
            }
            let w = std::f32::consts::TAU * freq * t;
            let v = (w.sin() * 0.8 + (2.0 * w).sin() * 0.2) * env * gain;
            samples.push((v * i16::MAX as f32) as i16);
        }
    }
    let data_len = (samples.len() * 2) as u32;
    let mut wav: Vec<u8> = Vec::with_capacity(44 + data_len as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&RATE.to_le_bytes());
    wav.extend_from_slice(&(RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

pub fn play_toggle_sound(now_enabled: bool) {
    let vol = SOUND_VOLUME.load(Ordering::SeqCst);
    if vol == 0 {
        return;
    }
    thread::spawn(move || {
        let tones: &[(f32, f32)] = if now_enabled {
            &[(784.0, 70.0), (1175.0, 110.0), (0.0, 40.0)]
        } else {
            &[(698.0, 70.0), (440.0, 130.0), (0.0, 40.0)]
        };
        let wav = build_tone_wav(tones, vol as f32 / 100.0);
        unsafe {
            let _ = PlaySoundW(
                wav.as_ptr() as *const u16,
                std::ptr::null_mut(),
                SND_MEMORY | SND_NODEFAULT,
            );
        }
    });
}
