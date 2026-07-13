mod get_transport;
mod pause;
mod play;
mod seek;
mod stop;

pub use get_transport::{GetTransport, GetTransportParams};
pub use pause::{PauseAudio, PauseAudioParams};
pub use play::{PlayAudio, PlayAudioParams};
pub use seek::{SeekAudio, SeekAudioParams};
pub use stop::{StopAudio, StopAudioParams};

use schemars::JsonSchema;
use serde::Serialize;
use ts_rs::TS;

use crate::{ContractMethod, contract::describe};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum TransportState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct TransportSnapshot {
    #[ts(inline)]
    pub state: TransportState,
    pub position_tick: u32,
    pub duration_ticks: u32,
    pub last_error: Option<String>,
}

pub(crate) fn methods() -> Vec<ContractMethod> {
    vec![
        describe::<GetTransport>(),
        describe::<PauseAudio>(),
        describe::<PlayAudio>(),
        describe::<SeekAudio>(),
        describe::<StopAudio>(),
    ]
}
