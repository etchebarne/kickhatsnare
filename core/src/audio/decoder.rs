use std::{fs::File, path::Path};

use rodio::{Decoder, Source};

use crate::CoreError;

#[derive(Debug, Clone)]
pub struct DecodedAudio {
    pub channels: u16,
    pub sample_rate: u32,
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
    let mut decoder = Decoder::try_from(file).map_err(|error| {
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
    let duration = decoder.total_duration().ok_or_else(|| {
        CoreError::new(format!(
            "audio file duration is unavailable: {}",
            path.display()
        ))
    })?;
    if duration.is_zero() || sample_rate == 0 {
        return Err(CoreError::new(format!(
            "audio file is empty: {}",
            path.display()
        )));
    }
    let waveform = sample_waveform(&mut decoder, duration, channels);
    Ok(DecodedAudio {
        channels,
        sample_rate,
        duration_seconds: duration.as_secs_f64(),
        waveform,
    })
}

fn sample_waveform(
    decoder: &mut Decoder<std::io::BufReader<File>>,
    duration: std::time::Duration,
    channels: u16,
) -> Vec<f32> {
    const BUCKET_COUNT: u32 = 128;
    const FRAMES_PER_BUCKET: u32 = 64;

    let mut waveform = Vec::with_capacity(
        usize::try_from(BUCKET_COUNT).expect("waveform bucket count fits in usize"),
    );
    for bucket in 0..BUCKET_COUNT {
        if bucket > 0 {
            let position = duration.mul_f64(f64::from(bucket) / f64::from(BUCKET_COUNT));
            if decoder.try_seek(position).is_err() {
                break;
            }
        }
        let sample_count = FRAMES_PER_BUCKET.saturating_mul(u32::from(channels));
        let peak = decoder
            .by_ref()
            .take(usize::try_from(sample_count).expect("waveform sample count fits in usize"))
            .fold(0.0_f32, |peak, sample| peak.max(sample.abs()))
            .min(1.0);
        waveform.push(peak);
    }
    waveform
}
