use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::IpcMethod;

pub struct GetWaveformPeaks;

impl IpcMethod for GetWaveformPeaks {
    const NAME: &'static str = "audio.getWaveformPeaks";
    type Params = GetWaveformPeaksParams;
    type Result = WaveformPeaks;
}

#[derive(Debug, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(rename_all = "camelCase")]
pub struct GetWaveformPeaksParams {
    pub source_path: String,
    pub frames_per_peak: u32,
    pub start_peak: u32,
    pub peak_count: u32,
}

#[derive(Debug, Serialize, JsonSchema, TS)]
#[serde(tag = "status", rename_all = "camelCase")]
#[ts(tag = "status", rename_all = "camelCase")]
pub enum WaveformPeaks {
    Loading,
    Ready {
        #[serde(rename = "sourceVersion")]
        #[ts(rename = "sourceVersion")]
        source_version: String,
        #[serde(rename = "framesPerPeak")]
        #[ts(rename = "framesPerPeak")]
        frames_per_peak: u32,
        #[serde(rename = "startPeak")]
        #[ts(rename = "startPeak")]
        start_peak: u32,
        #[serde(rename = "totalPeaks")]
        #[ts(rename = "totalPeaks")]
        total_peaks: u32,
        minimums: Vec<f32>,
        maximums: Vec<f32>,
    },
}
