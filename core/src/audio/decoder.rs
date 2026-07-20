use std::{fs::File, io::BufReader, path::Path, sync::Arc, time::Duration};

use rodio::{Decoder, Source};

use super::waveform::{DecodedAudio, WaveformPyramid};
use crate::CoreError;

pub(super) fn analyze(path: &Path) -> Result<DecodedAudio, CoreError> {
    let decoder = open(path)?;
    let (channels, sample_rate, duration) = metadata(&decoder, path)?;
    Ok(DecodedAudio {
        channels,
        sample_rate,
        duration_seconds: duration.as_secs_f64(),
        waveform: Vec::new(),
    })
}

pub(super) fn decode(path: &Path) -> Result<Arc<WaveformPyramid>, CoreError> {
    let decoder = open(path)?;
    let (channels, sample_rate, duration) = metadata(&decoder, path)?;
    WaveformPyramid::from_samples(channels, sample_rate, duration.as_secs_f64(), decoder)
        .map(Arc::new)
        .ok_or_else(|| CoreError::new(format!("audio file is empty: {}", path.display())))
}

fn open(path: &Path) -> Result<Decoder<BufReader<File>>, CoreError> {
    let file = File::open(path).map_err(|error| {
        CoreError::new(format!(
            "failed to open audio file {}: {error}",
            path.display()
        ))
    })?;
    Decoder::try_from(file).map_err(|error| {
        CoreError::new(format!(
            "failed to decode audio file {}: {error}",
            path.display()
        ))
    })
}

fn metadata(
    decoder: &Decoder<BufReader<File>>,
    path: &Path,
) -> Result<(u16, u32, Duration), CoreError> {
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

    Ok((channels, sample_rate, duration))
}
