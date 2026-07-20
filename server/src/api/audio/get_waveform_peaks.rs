use kickhatsnare_core::Core;
use kickhatsnare_protocol::{
    ErrorCode,
    audio::{GetWaveformPeaksParams, WaveformPeaks},
};
use serde_json::Value;

use super::ApiError;

pub(super) fn handle(params: &Value, core: &mut Core) -> Result<Value, ApiError> {
    let params = serde_json::from_value::<GetWaveformPeaksParams>(params.clone())
        .map_err(|error| ApiError::new(ErrorCode::InvalidParams, error.to_string()))?;
    let peaks = core
        .waveform_peaks(
            &params.source_path,
            params.frames_per_peak,
            params.start_peak,
            params.peak_count,
        )
        .map_err(|error| ApiError::new(ErrorCode::InternalError, error.to_string()))?;
    let peaks = match peaks {
        kickhatsnare_core::audio::WaveformPeaks::Loading => WaveformPeaks::Loading,
        kickhatsnare_core::audio::WaveformPeaks::Ready {
            source_version,
            frames_per_peak,
            start_peak,
            total_peaks,
            minimums,
            maximums,
        } => WaveformPeaks::Ready {
            source_version,
            frames_per_peak,
            start_peak,
            total_peaks,
            minimums,
            maximums,
        },
    };
    serde_json::to_value(peaks)
        .map_err(|error| ApiError::new(ErrorCode::InternalError, error.to_string()))
}
