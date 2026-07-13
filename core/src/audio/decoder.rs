use std::{fs::File, path::Path, time::Duration};

use rodio::{Decoder, Source};

use crate::CoreError;

#[derive(Debug, Clone)]
pub struct DecodedAudio {
    pub samples: Vec<f32>,
    pub channels: u16,
    pub sample_rate: u32,
    pub frame_count: usize,
    pub duration_seconds: f64,
    pub waveform: Vec<f32>,
}

pub fn decode(path: &Path) -> Result<DecodedAudio, CoreError> {
    let file = File::open(path).map_err(|error| {
        CoreError::new(format!(
            "failed to open audio file {}: {error}",
            path.display()
        ))
    })?;
    let decoder = Decoder::try_from(file).map_err(|error| {
        CoreError::new(format!(
            "failed to decode audio file {}: {error}",
            path.display()
        ))
    })?;
    let channels = decoder.channels();
    let sample_rate = decoder.sample_rate();
    if !(1..=2).contains(&channels) {
        return Err(CoreError::new(format!(
            "audio file must be mono or stereo: {}",
            path.display()
        )));
    }
    let samples = decoder.collect::<Vec<_>>();
    let frame_count = samples.len() / usize::from(channels);
    if frame_count == 0 || sample_rate == 0 {
        return Err(CoreError::new(format!(
            "audio file is empty: {}",
            path.display()
        )));
    }
    let frames = u64::try_from(frame_count)
        .map_err(|_| CoreError::new("audio file is too large to decode"))?;
    let sample_rate_u64 = u64::from(sample_rate);
    let duration_seconds = Duration::from_secs(frames / sample_rate_u64).as_secs_f64()
        + f64::from(
            u32::try_from(frames % sample_rate_u64)
                .expect("frame remainder is smaller than the sample rate"),
        ) / f64::from(sample_rate);
    let waveform = waveform(&samples, usize::from(channels), 256);
    Ok(DecodedAudio {
        samples,
        channels,
        sample_rate,
        frame_count,
        duration_seconds,
        waveform,
    })
}

fn waveform(samples: &[f32], channels: usize, bucket_count: usize) -> Vec<f32> {
    let frames = samples.len() / channels;
    let bucket_count = bucket_count.min(frames).max(1);
    (0..bucket_count)
        .map(|bucket| {
            let start = bucket * frames / bucket_count;
            let end = ((bucket + 1) * frames / bucket_count).max(start + 1);
            samples[start * channels..end * channels]
                .iter()
                .fold(0.0_f32, |peak, sample| peak.max(sample.abs()))
                .min(1.0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::waveform;

    #[test]
    fn builds_bounded_peak_buckets() {
        let peaks = waveform(&[0.1, -0.5, 0.25, 0.8, -1.2, 0.2, 0.4, 0.3], 2, 2);

        assert_eq!(peaks, [0.8, 1.0]);
    }
}
