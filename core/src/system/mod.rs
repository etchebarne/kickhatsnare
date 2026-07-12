#[derive(Debug, Default)]
pub struct System;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingReply {
    Pong,
}

impl System {
    #[must_use]
    pub fn ping(&mut self) -> PingReply {
        PingReply::Pong
    }
}

#[cfg(test)]
mod tests {
    use super::{PingReply, System};

    #[test]
    fn responds_to_ping() {
        assert_eq!(System.ping(), PingReply::Pong);
    }
}
