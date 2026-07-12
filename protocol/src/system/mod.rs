mod ping;

pub use ping::{Ping, PingParams, PingResult};

use crate::{ContractMethod, contract::describe};

pub(crate) fn methods() -> Vec<ContractMethod> {
    vec![describe::<Ping>()]
}
