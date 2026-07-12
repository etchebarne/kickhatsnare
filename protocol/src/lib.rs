pub mod audio;
mod contract;
mod envelope;
pub mod system;
pub mod workspace;

pub use contract::{Contract, ContractMethod, IpcMethod, contract};
pub use envelope::{ErrorCode, Request, Response, ResponseError};

pub const PROTOCOL_VERSION: u32 = 1;
