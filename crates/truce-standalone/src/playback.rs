//! `.wav` file → plugin input bus, gated on the `playback` feature.
//!
//! Decodes a WAV at startup, adapts it to the device sample rate +
//! channel count once, then sums the result into the audio
//! callback's per-channel buffers each block. One-shot — the
//! cursor saturates at the end of the file and subsequent calls
//! contribute nothing.
//!
//! Mic input (when enabled) and file playback both sum into the
//! same input bus, matching the CLI matrix in `cli.rs` /
//! `--help`.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Pre-decoded WAV at the device's sample rate and channel count.
/// `Send + Sync` (just owns a `Vec<f32>` and an atomic) so it can
/// be cloned-by-`Arc` into the audio worker.
pub struct PlaybackSource {
    /// Interleaved samples, `channels` per frame.
    samples: Vec<f32>,
    channels: usize,
    total_frames: usize,
    /// Number of frames consumed so far. Saturates at
    /// `total_frames`. Atomic so the audio callback can advance
    /// it without holding any lock.
    cursor: AtomicUsize,
}

impl PlaybackSource {
    /// Decode `path`, adapt to `target_sr` / `target_channels`.
    /// Errors out only on unreadable / unparseable files; channel
    /// and SR mismatches are handled with an `eprintln!` warning
    /// and a documented resolution.
    pub fn from_wav(path: &Path, target_sr: f64, target_channels: usize) -> Result<Self, String> {
        let mut reader = hound::WavReader::open(path)
            .map_err(|e| format!("could not open '{}': {e}", path.display()))?;
        let spec = reader.spec();

        // Decode all samples to f32 in [-1.0, 1.0].
        let raw: Vec<f32> = match (spec.sample_format, spec.bits_per_sample) {
            (hound::SampleFormat::Int, 16) => reader
                .samples::<i16>()
                .map(|s| s.map(|v| v as f32 / (i16::MAX as f32 + 1.0)))
                .collect::<Result<_, _>>()
                .map_err(|e| format!("WAV decode error: {e}"))?,
            (hound::SampleFormat::Int, 24) | (hound::SampleFormat::Int, 32) => {
                let bits = spec.bits_per_sample;
                // Hound returns 24-bit samples sign-extended in i32.
                let scale = (1u64 << (bits - 1)) as f32;
                reader
                    .samples::<i32>()
                    .map(|s| s.map(|v| v as f32 / scale))
                    .collect::<Result<_, _>>()
                    .map_err(|e| format!("WAV decode error: {e}"))?
            }
            (hound::SampleFormat::Float, 32) => reader
                .samples::<f32>()
                .collect::<Result<_, _>>()
                .map_err(|e| format!("WAV decode error: {e}"))?,
            (fmt, bits) => {
                return Err(format!(
                    "unsupported WAV format: {fmt:?} {bits}-bit \
                     (truce standalone supports int 16/24/32 and float 32)"
                ));
            }
        };

        let src_channels = spec.channels as usize;
        let src_sr = spec.sample_rate as f64;
        if src_channels == 0 {
            return Err("WAV has zero channels".into());
        }
        let src_frames = raw.len() / src_channels;
        if src_frames == 0 {
            return Err("WAV is empty".into());
        }

        // Sample-rate adapt first (cheaper to rechannel a smaller
        // buffer when downsampling, and a no-op when SR matches).
        let resampled: Vec<f32> = if (src_sr - target_sr).abs() < f64::EPSILON {
            raw
        } else {
            linear_resample(&raw, src_channels, src_frames, src_sr, target_sr)
        };
        let resampled_frames = resampled.len() / src_channels;

        // Channel adapt. See the table in `formats/standalone.md`:
        //   1 → N : broadcast mono to every channel
        //   N → N : passthrough
        //   N → M, N > M : take first M channels, warn
        //   N → M, N < M : copy file to dst[0..N], zero-fill rest
        let samples: Vec<f32> = if src_channels == target_channels {
            resampled
        } else if src_channels == 1 {
            let mut out = Vec::with_capacity(resampled_frames * target_channels);
            for &s in &resampled {
                for _ in 0..target_channels {
                    out.push(s);
                }
            }
            out
        } else {
            if src_channels > target_channels {
                eprintln!(
                    "[truce-standalone] file is {src_channels}ch, device is \
                     {target_channels}ch — discarding channels [{target_channels}..{src_channels}]"
                );
            } else {
                eprintln!(
                    "[truce-standalone] file is {src_channels}ch, device is \
                     {target_channels}ch — zero-filling channels [{src_channels}..{target_channels}]"
                );
            }
            let mut out = vec![0.0_f32; resampled_frames * target_channels];
            let copy = src_channels.min(target_channels);
            for f in 0..resampled_frames {
                for ch in 0..copy {
                    out[f * target_channels + ch] = resampled[f * src_channels + ch];
                }
            }
            out
        };

        let total_frames = samples.len() / target_channels;
        Ok(Self {
            samples,
            channels: target_channels,
            total_frames,
            cursor: AtomicUsize::new(0),
        })
    }

    /// Sum `frames` frames of playback samples into `channel_bufs`
    /// (one `Vec<f32>` per device channel, all sized `>= frames`).
    /// Saturates at EOF — calls beyond `total_frames` are no-ops.
    pub fn mix_into(&self, channel_bufs: &mut [Vec<f32>], frames: usize) {
        let start = self.cursor.load(Ordering::Relaxed);
        if start >= self.total_frames {
            return;
        }
        let take = frames.min(self.total_frames - start);
        let chans = self.channels.min(channel_bufs.len());
        let stride = self.channels;
        for (ch, buf) in channel_bufs.iter_mut().take(chans).enumerate() {
            for (f, dst) in buf.iter_mut().take(take).enumerate() {
                *dst += self.samples[(start + f) * stride + ch];
            }
        }
        self.cursor.store(start + take, Ordering::Relaxed);
    }
}

/// Linear-interp resample interleaved `src` from `src_sr` to
/// `target_sr`. Quality limitation called out in `--help`. No
/// anti-alias filter — fine for pre-rendered test signals at the
/// device's native SR (the dominant case is no resample at all),
/// audible aliasing on broadband content.
fn linear_resample(
    src: &[f32],
    channels: usize,
    src_frames: usize,
    src_sr: f64,
    target_sr: f64,
) -> Vec<f32> {
    let ratio = target_sr / src_sr;
    let target_frames = ((src_frames as f64) * ratio).round() as usize;
    let mut out = vec![0.0_f32; target_frames * channels];
    let inv_ratio = src_sr / target_sr;
    for f in 0..target_frames {
        let src_pos = f as f64 * inv_ratio;
        let lo = src_pos.floor() as usize;
        let hi = (lo + 1).min(src_frames - 1);
        let t = (src_pos - lo as f64) as f32;
        for ch in 0..channels {
            let a = src[lo * channels + ch];
            let b = src[hi * channels + ch];
            out[f * channels + ch] = a + (b - a) * t;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_wav(path: &Path, sr: u32, channels: u16, samples: &[i16]) {
        let spec = hound::WavSpec {
            channels,
            sample_rate: sr,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(path, spec).unwrap();
        for &s in samples {
            w.write_sample(s).unwrap();
        }
        w.finalize().unwrap();
    }

    #[test]
    fn one_shot_saturates_at_eof() {
        let dir = tempdir_path();
        let path = dir.join("tone.wav");
        // 4 frames stereo, simple ramp.
        write_wav(
            &path,
            48_000,
            2,
            &[1000, -1000, 2000, -2000, 3000, -3000, 4000, -4000],
        );

        let src = PlaybackSource::from_wav(&path, 48_000.0, 2).unwrap();
        assert_eq!(src.total_frames, 4);

        let mut bufs = vec![vec![0.0_f32; 8]; 2];
        src.mix_into(&mut bufs, 8);
        // First 4 frames have content (additive into zero-init bufs)
        assert!(bufs[0][0] != 0.0);
        assert!(bufs[0][3] != 0.0);
        // Frames 4..8 stay zero — saturated.
        assert_eq!(bufs[0][4], 0.0);
        assert_eq!(bufs[0][7], 0.0);

        // Subsequent calls are no-ops.
        let mut bufs2 = vec![vec![0.0_f32; 4]; 2];
        src.mix_into(&mut bufs2, 4);
        for ch in &bufs2 {
            for &s in ch {
                assert_eq!(s, 0.0);
            }
        }
    }

    #[test]
    fn mono_broadcasts_to_stereo() {
        let dir = tempdir_path();
        let path = dir.join("mono.wav");
        write_wav(&path, 48_000, 1, &[16384, -16384]);
        let src = PlaybackSource::from_wav(&path, 48_000.0, 2).unwrap();
        let mut bufs = vec![vec![0.0_f32; 2]; 2];
        src.mix_into(&mut bufs, 2);
        // L and R should be equal (mono broadcast).
        assert_eq!(bufs[0][0], bufs[1][0]);
        assert_eq!(bufs[0][1], bufs[1][1]);
        assert!(bufs[0][0] > 0.4);
        assert!(bufs[0][1] < -0.4);
    }

    #[test]
    fn mix_is_additive() {
        let dir = tempdir_path();
        let path = dir.join("ones.wav");
        write_wav(&path, 48_000, 2, &[16384, 16384, 16384, 16384]);
        let src = PlaybackSource::from_wav(&path, 48_000.0, 2).unwrap();
        let mut bufs = vec![vec![0.5_f32; 2]; 2];
        src.mix_into(&mut bufs, 2);
        // 0.5 + ~0.5 = ~1.0 (mic-style pre-existing signal + file).
        assert!((bufs[0][0] - 1.0).abs() < 0.01);
    }

    fn tempdir_path() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "truce-standalone-playback-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
