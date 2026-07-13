use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::CoreError;

pub const MASTER_NODE_ID: &str = "master";
pub const AUDIO_INPUT_PORT_ID: &str = "audio-in";
pub const AUDIO_OUTPUT_PORT_ID: &str = "audio-out";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixNodeKind {
    TrackChannel,
    MasterOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixPortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixSignalType {
    Audio,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixPortSnapshot {
    pub id: String,
    pub label: String,
    pub direction: MixPortDirection,
    pub signal_type: MixSignalType,
    pub allows_multiple_connections: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixNodeSnapshot {
    pub id: String,
    pub kind: MixNodeKind,
    pub track_id: Option<String>,
    pub x: f64,
    pub y: f64,
    pub ports: Vec<MixPortSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MixConnection {
    pub source_node_id: String,
    pub source_port_id: String,
    pub target_node_id: String,
    pub target_port_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixGraphSnapshot {
    pub nodes: Vec<MixNodeSnapshot>,
    pub connections: Vec<MixConnection>,
}

impl MixGraphSnapshot {
    pub(crate) fn track_routes_to_master(&self, track_id: &str) -> bool {
        self.connections.iter().any(|connection| {
            connection.source_node_id == track_id
                && connection.source_port_id == AUDIO_OUTPUT_PORT_ID
                && connection.target_node_id == MASTER_NODE_ID
                && connection.target_port_id == AUDIO_INPUT_PORT_ID
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct MixGraph {
    nodes: Vec<MixNode>,
    connections: Vec<MixConnection>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MixNode {
    id: String,
    kind: StoredMixNodeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    track_id: Option<String>,
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum StoredMixNodeKind {
    TrackChannel,
    MasterOutput,
}

#[derive(Debug, Clone)]
pub(super) struct LegacyTrackMix {
    pub id: String,
    pub is_connected: bool,
    pub x: f64,
    pub y: f64,
}

impl MixGraph {
    pub(super) fn new(track_ids: impl IntoIterator<Item = String>) -> Self {
        let mut graph = Self {
            nodes: vec![MixNode {
                id: MASTER_NODE_ID.to_owned(),
                kind: StoredMixNodeKind::MasterOutput,
                track_id: None,
                x: default_master_x(),
                y: default_master_y(),
            }],
            connections: Vec::new(),
        };
        for track_id in track_ids {
            graph
                .add_track(&track_id)
                .expect("default mix node positions should be valid");
        }
        graph
    }

    pub(super) fn from_legacy(
        tracks: impl IntoIterator<Item = LegacyTrackMix>,
        master_x: f64,
        master_y: f64,
    ) -> Self {
        let mut graph = Self {
            nodes: vec![MixNode {
                id: MASTER_NODE_ID.to_owned(),
                kind: StoredMixNodeKind::MasterOutput,
                track_id: None,
                x: master_x,
                y: master_y,
            }],
            connections: Vec::new(),
        };
        for track in tracks {
            graph.nodes.push(MixNode {
                id: track.id.clone(),
                kind: StoredMixNodeKind::TrackChannel,
                track_id: Some(track.id.clone()),
                x: track.x,
                y: track.y,
            });
            if track.is_connected {
                graph
                    .connections
                    .push(track_to_master_connection(&track.id));
            }
        }
        graph
    }

    pub(super) fn snapshot(&self) -> MixGraphSnapshot {
        MixGraphSnapshot {
            nodes: self
                .nodes
                .iter()
                .map(|node| MixNodeSnapshot {
                    id: node.id.clone(),
                    kind: node.kind.into(),
                    track_id: node.track_id.clone(),
                    x: node.x,
                    y: node.y,
                    ports: ports(node.kind),
                })
                .collect(),
            connections: self.connections.clone(),
        }
    }

    pub(super) fn add_track(&mut self, track_id: &str) -> Result<(), CoreError> {
        if self.nodes.iter().any(|node| node.id == track_id) {
            return Err(CoreError::new("mix node already exists"));
        }
        let number = track_number(track_id).unwrap_or(self.nodes.len() as u64);
        let (x, y) = default_track_position(number);
        validate_position(x, y)?;
        self.nodes.push(MixNode {
            id: track_id.to_owned(),
            kind: StoredMixNodeKind::TrackChannel,
            track_id: Some(track_id.to_owned()),
            x,
            y,
        });
        self.connections.push(track_to_master_connection(track_id));
        Ok(())
    }

    pub(super) fn remove_track(&mut self, track_id: &str) {
        self.nodes
            .retain(|node| node.track_id.as_deref() != Some(track_id));
        self.connections.retain(|connection| {
            connection.source_node_id != track_id && connection.target_node_id != track_id
        });
    }

    pub(super) fn set_node_position(
        &mut self,
        node_id: &str,
        x: f64,
        y: f64,
    ) -> Result<(), CoreError> {
        validate_position(x, y)?;
        let node = self
            .nodes
            .iter_mut()
            .find(|node| node.id == node_id)
            .ok_or_else(|| CoreError::new("mix node does not exist"))?;
        node.x = x;
        node.y = y;
        Ok(())
    }

    pub(super) fn connect(
        &mut self,
        source_node_id: &str,
        source_port_id: &str,
        target_node_id: &str,
        target_port_id: &str,
    ) -> Result<(), CoreError> {
        let connection = MixConnection {
            source_node_id: source_node_id.to_owned(),
            source_port_id: source_port_id.to_owned(),
            target_node_id: target_node_id.to_owned(),
            target_port_id: target_port_id.to_owned(),
        };
        if self.connections.contains(&connection) {
            return Ok(());
        }
        validate_connection(&self.nodes, &self.connections, &connection)?;
        self.connections.push(connection);
        if has_cycle(&self.nodes, &self.connections) {
            self.connections.pop();
            return Err(CoreError::new("mix connections cannot contain a cycle"));
        }
        Ok(())
    }

    pub(super) fn disconnect(
        &mut self,
        source_node_id: &str,
        source_port_id: &str,
        target_node_id: &str,
        target_port_id: &str,
    ) -> Result<(), CoreError> {
        let index = self
            .connections
            .iter()
            .position(|connection| {
                connection.source_node_id == source_node_id
                    && connection.source_port_id == source_port_id
                    && connection.target_node_id == target_node_id
                    && connection.target_port_id == target_port_id
            })
            .ok_or_else(|| CoreError::new("mix connection does not exist"))?;
        self.connections.remove(index);
        Ok(())
    }

    pub(super) fn track_routes_to_master(&self, track_id: &str) -> bool {
        self.connections.iter().any(|connection| {
            connection.source_node_id == track_id
                && connection.source_port_id == AUDIO_OUTPUT_PORT_ID
                && connection.target_node_id == MASTER_NODE_ID
                && connection.target_port_id == AUDIO_INPUT_PORT_ID
        })
    }

    pub(super) fn validate(&self, track_ids: &HashSet<&str>) -> Result<(), CoreError> {
        let mut node_ids = HashSet::new();
        let mut channel_track_ids = HashSet::new();
        let mut master_count = 0;
        for node in &self.nodes {
            validate_position(node.x, node.y)?;
            if node.id.is_empty() || !node_ids.insert(node.id.as_str()) {
                return Err(CoreError::new(
                    "project contains an invalid or duplicate mix node ID",
                ));
            }
            match node.kind {
                StoredMixNodeKind::TrackChannel => {
                    let track_id = node.track_id.as_deref().ok_or_else(|| {
                        CoreError::new("track channel node must reference a timeline track")
                    })?;
                    if node.id != track_id
                        || !track_ids.contains(track_id)
                        || !channel_track_ids.insert(track_id)
                    {
                        return Err(CoreError::new(
                            "mix graph contains an invalid track channel",
                        ));
                    }
                }
                StoredMixNodeKind::MasterOutput => {
                    master_count += 1;
                    if node.id != MASTER_NODE_ID || node.track_id.is_some() {
                        return Err(CoreError::new(
                            "mix graph contains an invalid master output",
                        ));
                    }
                }
            }
        }
        if master_count != 1 || channel_track_ids != *track_ids {
            return Err(CoreError::new(
                "mix graph must contain one channel per track and one master output",
            ));
        }

        let mut unique_connections = HashSet::new();
        for connection in &self.connections {
            if !unique_connections.insert((
                connection.source_node_id.as_str(),
                connection.source_port_id.as_str(),
                connection.target_node_id.as_str(),
                connection.target_port_id.as_str(),
            )) {
                return Err(CoreError::new("mix graph contains a duplicate connection"));
            }
            validate_connection(&self.nodes, &self.connections, connection)?;
        }
        if has_cycle(&self.nodes, &self.connections) {
            return Err(CoreError::new("mix connections cannot contain a cycle"));
        }
        Ok(())
    }
}

impl From<StoredMixNodeKind> for MixNodeKind {
    fn from(kind: StoredMixNodeKind) -> Self {
        match kind {
            StoredMixNodeKind::TrackChannel => Self::TrackChannel,
            StoredMixNodeKind::MasterOutput => Self::MasterOutput,
        }
    }
}

fn ports(kind: StoredMixNodeKind) -> Vec<MixPortSnapshot> {
    match kind {
        StoredMixNodeKind::TrackChannel => vec![MixPortSnapshot {
            id: AUDIO_OUTPUT_PORT_ID.to_owned(),
            label: "Audio out".to_owned(),
            direction: MixPortDirection::Output,
            signal_type: MixSignalType::Audio,
            allows_multiple_connections: true,
        }],
        StoredMixNodeKind::MasterOutput => vec![MixPortSnapshot {
            id: AUDIO_INPUT_PORT_ID.to_owned(),
            label: "Audio in".to_owned(),
            direction: MixPortDirection::Input,
            signal_type: MixSignalType::Audio,
            allows_multiple_connections: true,
        }],
    }
}

fn validate_connection(
    nodes: &[MixNode],
    existing: &[MixConnection],
    connection: &MixConnection,
) -> Result<(), CoreError> {
    if connection.source_node_id == connection.target_node_id {
        return Err(CoreError::new("mix nodes cannot connect to themselves"));
    }
    let source = nodes
        .iter()
        .find(|node| node.id == connection.source_node_id)
        .ok_or_else(|| CoreError::new("mix connection source node does not exist"))?;
    let target = nodes
        .iter()
        .find(|node| node.id == connection.target_node_id)
        .ok_or_else(|| CoreError::new("mix connection target node does not exist"))?;
    let source_port = ports(source.kind)
        .into_iter()
        .find(|port| port.id == connection.source_port_id)
        .ok_or_else(|| CoreError::new("mix connection source port does not exist"))?;
    let target_port = ports(target.kind)
        .into_iter()
        .find(|port| port.id == connection.target_port_id)
        .ok_or_else(|| CoreError::new("mix connection target port does not exist"))?;
    if source_port.direction != MixPortDirection::Output
        || target_port.direction != MixPortDirection::Input
        || source_port.signal_type != target_port.signal_type
    {
        return Err(CoreError::new("mix connection ports are incompatible"));
    }
    if !target_port.allows_multiple_connections
        && existing.iter().any(|item| {
            item.target_node_id == connection.target_node_id
                && item.target_port_id == connection.target_port_id
                && item != connection
        })
    {
        return Err(CoreError::new("mix input already has a connection"));
    }
    if !source_port.allows_multiple_connections
        && existing.iter().any(|item| {
            item.source_node_id == connection.source_node_id
                && item.source_port_id == connection.source_port_id
                && item != connection
        })
    {
        return Err(CoreError::new("mix output already has a connection"));
    }
    Ok(())
}

fn has_cycle(nodes: &[MixNode], connections: &[MixConnection]) -> bool {
    let mut incoming = nodes
        .iter()
        .map(|node| (node.id.as_str(), 0_usize))
        .collect::<HashMap<_, _>>();
    let mut outgoing = HashMap::<&str, Vec<&str>>::new();
    for connection in connections {
        *incoming
            .entry(connection.target_node_id.as_str())
            .or_default() += 1;
        outgoing
            .entry(connection.source_node_id.as_str())
            .or_default()
            .push(connection.target_node_id.as_str());
    }
    let mut queue = incoming
        .iter()
        .filter_map(|(node, count)| (*count == 0).then_some(*node))
        .collect::<VecDeque<_>>();
    let mut visited = 0;
    while let Some(node) = queue.pop_front() {
        visited += 1;
        for target in outgoing.get(node).into_iter().flatten() {
            if let Some(count) = incoming.get_mut(target) {
                *count -= 1;
                if *count == 0 {
                    queue.push_back(target);
                }
            }
        }
    }
    visited != nodes.len()
}

fn track_to_master_connection(track_id: &str) -> MixConnection {
    MixConnection {
        source_node_id: track_id.to_owned(),
        source_port_id: AUDIO_OUTPUT_PORT_ID.to_owned(),
        target_node_id: MASTER_NODE_ID.to_owned(),
        target_port_id: AUDIO_INPUT_PORT_ID.to_owned(),
    }
}

fn track_number(track_id: &str) -> Option<u64> {
    track_id.strip_prefix("track-")?.parse().ok()
}

fn default_track_position(number: u64) -> (f64, f64) {
    let index = number.saturating_sub(1);
    let column = u32::try_from(index % 2).expect("track column is always less than two");
    let row = u32::try_from(index / 2).unwrap_or(u32::MAX);
    (f64::from(column) * 260.0, f64::from(row) * 176.0)
}

pub(super) fn default_master_x() -> f64 {
    820.0
}

pub(super) fn default_master_y() -> f64 {
    352.0
}

pub(super) fn default_track_position_for(number: u64) -> (f64, f64) {
    default_track_position(number)
}

fn validate_position(x: f64, y: f64) -> Result<(), CoreError> {
    if !x.is_finite() || !y.is_finite() || x.abs() > 100_000.0 || y.abs() > 100_000.0 {
        return Err(CoreError::new("mix node position is invalid"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AUDIO_INPUT_PORT_ID, AUDIO_OUTPUT_PORT_ID, MASTER_NODE_ID, MixGraph};

    #[test]
    fn exposes_typed_ports_and_explicit_default_connections() {
        let graph = MixGraph::new(["track-1".to_owned()]);
        let snapshot = graph.snapshot();

        assert_eq!(snapshot.nodes.len(), 2);
        assert_eq!(snapshot.connections.len(), 1);
        assert_eq!(snapshot.connections[0].source_node_id, "track-1");
        assert_eq!(snapshot.connections[0].source_port_id, AUDIO_OUTPUT_PORT_ID);
        assert_eq!(snapshot.connections[0].target_node_id, MASTER_NODE_ID);
        assert_eq!(snapshot.connections[0].target_port_id, AUDIO_INPUT_PORT_ID);
        assert_eq!(snapshot.nodes[0].ports[0].label, "Audio in");
        assert_eq!(snapshot.nodes[1].ports[0].label, "Audio out");
    }

    #[test]
    fn connects_and_disconnects_ports_idempotently() {
        let mut graph = MixGraph::new(["track-1".to_owned()]);
        graph
            .disconnect(
                "track-1",
                AUDIO_OUTPUT_PORT_ID,
                MASTER_NODE_ID,
                AUDIO_INPUT_PORT_ID,
            )
            .expect("default route should disconnect");
        assert!(!graph.track_routes_to_master("track-1"));

        graph
            .connect(
                "track-1",
                AUDIO_OUTPUT_PORT_ID,
                MASTER_NODE_ID,
                AUDIO_INPUT_PORT_ID,
            )
            .expect("route should connect");
        graph
            .connect(
                "track-1",
                AUDIO_OUTPUT_PORT_ID,
                MASTER_NODE_ID,
                AUDIO_INPUT_PORT_ID,
            )
            .expect("duplicate connect should be idempotent");

        assert!(graph.track_routes_to_master("track-1"));
        assert_eq!(graph.snapshot().connections.len(), 1);
    }

    #[test]
    fn rejects_incompatible_port_directions() {
        let mut graph = MixGraph::new(["track-1".to_owned()]);
        let error = graph
            .connect(
                MASTER_NODE_ID,
                AUDIO_INPUT_PORT_ID,
                "track-1",
                AUDIO_OUTPUT_PORT_ID,
            )
            .expect_err("input to output connection should fail");

        assert_eq!(error.to_string(), "mix connection ports are incompatible");
    }
}
