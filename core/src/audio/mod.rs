/// Owns audio-engine state and operations.
///
/// Audio graph, device, transport, and rendering modules should be composed
/// here rather than exposed directly to a transport crate.
#[derive(Debug, Default)]
pub struct Audio;
