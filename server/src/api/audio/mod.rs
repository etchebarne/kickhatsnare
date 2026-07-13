mod get_transport;
mod pause;
mod play;
mod seek;
mod stop;

use kickhatsnare_core::{Core, audio::TransportSnapshot};
use kickhatsnare_protocol::{
    IpcMethod,
    audio::{GetTransport, PauseAudio, PlayAudio, SeekAudio, StopAudio},
};
use serde_json::Value;

use super::ApiError;

pub(super) fn dispatch(
    method: &str,
    action: &str,
    params: &Value,
    core: &mut Core,
) -> Result<Value, ApiError> {
    match method {
        GetTransport::NAME => get_transport::handle(params, core),
        PauseAudio::NAME => pause::handle(params, core),
        PlayAudio::NAME => play::handle(params, core),
        SeekAudio::NAME => seek::handle(params, core),
        StopAudio::NAME => stop::handle(params, core),
        _ => Err(ApiError::method_not_found("audio", action)),
    }
}

fn serialize_transport(snapshot: TransportSnapshot) -> Result<Value, ApiError> {
    use kickhatsnare_core::audio::TransportState as CoreState;
    let snapshot = kickhatsnare_protocol::audio::TransportSnapshot {
        state: match snapshot.state {
            CoreState::Stopped => kickhatsnare_protocol::audio::TransportState::Stopped,
            CoreState::Playing => kickhatsnare_protocol::audio::TransportState::Playing,
            CoreState::Paused => kickhatsnare_protocol::audio::TransportState::Paused,
        },
        position_tick: snapshot.position_tick,
        duration_ticks: snapshot.duration_ticks,
        last_error: snapshot.last_error,
    };
    serde_json::to_value(snapshot).map_err(|error| {
        ApiError::new(
            kickhatsnare_protocol::ErrorCode::InternalError,
            error.to_string(),
        )
    })
}
