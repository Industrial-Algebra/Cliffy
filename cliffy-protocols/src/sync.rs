// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Synchronization protocol for P2P state coordination
//!
//! This module defines the protocol messages and types for peer-to-peer
//! state synchronization using geometric deltas and vector clocks.
//!
//! # Protocol Overview
//!
//! The sync protocol operates in three phases:
//! 1. **Discovery**: Peers announce themselves and exchange clock information
//! 2. **Sync**: Peers exchange deltas to converge on consistent state
//! 3. **Maintenance**: Periodic heartbeats and partition recovery
//!
//! # Example
//!
//! ```rust
//! use cliffy_protocols::sync::{SyncMessage, SyncState, PeerInfo};
//! use cliffy_protocols::VectorClock;
//! use uuid::Uuid;
//!
//! // Create sync state for a node
//! let node_id = Uuid::new_v4();
//! let mut sync_state = SyncState::new(node_id);
//!
//! // Register a peer
//! let peer_id = Uuid::new_v4();
//! sync_state.register_peer(peer_id, VectorClock::new());
//! ```

use crate::delta::DeltaBatch;
use crate::serde_ga3;
use crate::VectorClock;
use cliffy_core::GA3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// A message in the sync protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMessage {
    /// Unique message ID
    pub id: u64,
    /// Sender's node ID
    pub sender: Uuid,
    /// Message type and payload
    pub payload: SyncPayload,
    /// Sender's current vector clock
    pub clock: VectorClock,
    /// Timestamp (milliseconds since epoch, for debugging)
    pub timestamp: u64,
}

/// The payload of a sync message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncPayload {
    /// Announce presence to peers
    Hello(PeerInfo),

    /// Request clock comparison
    ClockRequest,

    /// Response with current clock
    ClockResponse(VectorClock),

    /// Request deltas since a given clock
    DeltaRequest {
        /// The clock the requester has
        since_clock: VectorClock,
    },

    /// Response with deltas
    DeltaResponse {
        /// Deltas to apply
        deltas: DeltaBatch,
        /// Whether more deltas are available
        has_more: bool,
    },

    /// Full state sync (for new peers or recovery)
    FullState {
        /// The complete state
        #[serde(with = "serde_ga3")]
        state: GA3,
        /// Clock at this state
        clock: VectorClock,
    },

    /// Heartbeat to maintain connection
    Heartbeat,

    /// Acknowledge receipt of deltas
    Ack {
        /// Message ID being acknowledged
        message_id: u64,
        /// Applied clock
        applied_clock: VectorClock,
    },

    /// Goodbye message when leaving
    Goodbye,
}

/// Information about a peer node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// The peer's node ID
    pub node_id: Uuid,
    /// Human-readable name (optional)
    pub name: Option<String>,
    /// Capabilities this peer supports
    pub capabilities: PeerCapabilities,
    /// Protocol version
    pub protocol_version: u32,
}

/// Capabilities a peer may support.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PeerCapabilities {
    /// Supports compressed deltas
    pub compressed_deltas: bool,
    /// Supports batch operations
    pub batch_operations: bool,
    /// Maximum batch size supported
    pub max_batch_size: usize,
    /// Supports full state snapshots
    pub full_state_sync: bool,
}

impl PeerCapabilities {
    /// Create default capabilities.
    pub fn default_capabilities() -> Self {
        Self {
            compressed_deltas: true,
            batch_operations: true,
            max_batch_size: 100,
            full_state_sync: true,
        }
    }
}

/// Connection state with a peer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerConnectionState {
    /// Just discovered, not yet synced
    Discovered,
    /// Currently synchronizing
    Syncing,
    /// Fully synchronized
    Synced,
    /// Connection lost, attempting recovery
    Disconnected,
    /// Peer has left the network
    Gone,
}

/// Tracked state for a connected peer.
#[derive(Debug, Clone)]
pub struct PeerState {
    /// Peer information
    pub info: PeerInfo,
    /// Last known clock from this peer
    pub last_clock: VectorClock,
    /// Connection state
    pub connection_state: PeerConnectionState,
    /// Last message received timestamp
    pub last_seen: Option<Instant>,
    /// Pending acknowledgments (message_id -> sent_time)
    pub pending_acks: HashMap<u64, Instant>,
    /// Round-trip time estimate (in milliseconds)
    pub rtt_estimate: Option<Duration>,
}

impl PeerState {
    /// Create a new peer state.
    pub fn new(info: PeerInfo, clock: VectorClock) -> Self {
        Self {
            info,
            last_clock: clock,
            connection_state: PeerConnectionState::Discovered,
            last_seen: None,
            pending_acks: HashMap::new(),
            rtt_estimate: None,
        }
    }

    /// Update the last seen time.
    pub fn touch(&mut self) {
        self.last_seen = Some(Instant::now());
    }

    /// Check if the peer is considered stale (no recent activity).
    pub fn is_stale(&self, timeout: Duration) -> bool {
        match self.last_seen {
            Some(last) => last.elapsed() > timeout,
            None => true,
        }
    }

    /// Record that we sent a message requiring acknowledgment.
    pub fn expect_ack(&mut self, message_id: u64) {
        self.pending_acks.insert(message_id, Instant::now());
    }

    /// Record that we received an acknowledgment.
    pub fn receive_ack(&mut self, message_id: u64) {
        if let Some(sent_time) = self.pending_acks.remove(&message_id) {
            let rtt = sent_time.elapsed();
            self.rtt_estimate = Some(match self.rtt_estimate {
                Some(prev) => Duration::from_millis(
                    (prev.as_millis() as f64 * 0.8 + rtt.as_millis() as f64 * 0.2) as u64,
                ),
                None => rtt,
            });
        }
    }
}

/// The synchronization state for a node.
#[derive(Debug)]
pub struct SyncState {
    /// This node's ID
    pub node_id: Uuid,
    /// Current vector clock
    pub clock: VectorClock,
    /// Known peers
    pub peers: HashMap<Uuid, PeerState>,
    /// Next message ID
    next_message_id: u64,
    /// Configuration
    pub config: SyncConfig,
}

/// Configuration for sync behavior.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// How often to send heartbeats
    pub heartbeat_interval: Duration,
    /// How long before a peer is considered stale
    pub peer_timeout: Duration,
    /// Maximum deltas per batch
    pub max_batch_size: usize,
    /// Whether to prefer compressed deltas
    pub prefer_compressed: bool,
    /// Protocol version
    pub protocol_version: u32,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(5),
            peer_timeout: Duration::from_secs(30),
            max_batch_size: 100,
            prefer_compressed: true,
            protocol_version: 1,
        }
    }
}

impl SyncState {
    /// Create a new sync state for a node.
    pub fn new(node_id: Uuid) -> Self {
        Self {
            node_id,
            clock: VectorClock::new(),
            peers: HashMap::new(),
            next_message_id: 0,
            config: SyncConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(node_id: Uuid, config: SyncConfig) -> Self {
        Self {
            node_id,
            clock: VectorClock::new(),
            peers: HashMap::new(),
            next_message_id: 0,
            config,
        }
    }

    /// Register a new peer.
    pub fn register_peer(&mut self, peer_id: Uuid, clock: VectorClock) {
        let info = PeerInfo {
            node_id: peer_id,
            name: None,
            capabilities: PeerCapabilities::default_capabilities(),
            protocol_version: self.config.protocol_version,
        };
        self.peers.insert(peer_id, PeerState::new(info, clock));
    }

    /// Register a peer with full info.
    pub fn register_peer_with_info(&mut self, info: PeerInfo, clock: VectorClock) {
        let peer_id = info.node_id;
        self.peers.insert(peer_id, PeerState::new(info, clock));
    }

    /// Remove a peer.
    pub fn remove_peer(&mut self, peer_id: &Uuid) {
        self.peers.remove(peer_id);
    }

    /// Get a peer's state.
    pub fn get_peer(&self, peer_id: &Uuid) -> Option<&PeerState> {
        self.peers.get(peer_id)
    }

    /// Get a mutable reference to a peer's state.
    pub fn get_peer_mut(&mut self, peer_id: &Uuid) -> Option<&mut PeerState> {
        self.peers.get_mut(peer_id)
    }

    /// Update our clock and get next message ID.
    pub fn tick(&mut self) -> u64 {
        self.clock.tick(self.node_id);
        let id = self.next_message_id;
        self.next_message_id += 1;
        id
    }

    /// Create a hello message.
    pub fn create_hello(&mut self, name: Option<String>) -> SyncMessage {
        let id = self.tick();
        SyncMessage {
            id,
            sender: self.node_id,
            payload: SyncPayload::Hello(PeerInfo {
                node_id: self.node_id,
                name,
                capabilities: PeerCapabilities::default_capabilities(),
                protocol_version: self.config.protocol_version,
            }),
            clock: self.clock.clone(),
            timestamp: current_timestamp_ms(),
        }
    }

    /// Create a delta request message.
    pub fn create_delta_request(&mut self, since_clock: VectorClock) -> SyncMessage {
        let id = self.tick();
        SyncMessage {
            id,
            sender: self.node_id,
            payload: SyncPayload::DeltaRequest { since_clock },
            clock: self.clock.clone(),
            timestamp: current_timestamp_ms(),
        }
    }

    /// Create a delta response message.
    pub fn create_delta_response(&mut self, deltas: DeltaBatch, has_more: bool) -> SyncMessage {
        let id = self.tick();
        SyncMessage {
            id,
            sender: self.node_id,
            payload: SyncPayload::DeltaResponse { deltas, has_more },
            clock: self.clock.clone(),
            timestamp: current_timestamp_ms(),
        }
    }

    /// Create a full state message.
    pub fn create_full_state(&mut self, state: GA3) -> SyncMessage {
        let id = self.tick();
        SyncMessage {
            id,
            sender: self.node_id,
            payload: SyncPayload::FullState {
                state,
                clock: self.clock.clone(),
            },
            clock: self.clock.clone(),
            timestamp: current_timestamp_ms(),
        }
    }

    /// Create a heartbeat message.
    pub fn create_heartbeat(&mut self) -> SyncMessage {
        let id = self.tick();
        SyncMessage {
            id,
            sender: self.node_id,
            payload: SyncPayload::Heartbeat,
            clock: self.clock.clone(),
            timestamp: current_timestamp_ms(),
        }
    }

    /// Create an acknowledgment message.
    pub fn create_ack(&mut self, message_id: u64) -> SyncMessage {
        let id = self.tick();
        SyncMessage {
            id,
            sender: self.node_id,
            payload: SyncPayload::Ack {
                message_id,
                applied_clock: self.clock.clone(),
            },
            clock: self.clock.clone(),
            timestamp: current_timestamp_ms(),
        }
    }

    /// Create a goodbye message.
    pub fn create_goodbye(&mut self) -> SyncMessage {
        let id = self.tick();
        SyncMessage {
            id,
            sender: self.node_id,
            payload: SyncPayload::Goodbye,
            clock: self.clock.clone(),
            timestamp: current_timestamp_ms(),
        }
    }

    /// Handle an incoming message from a peer.
    pub fn handle_message(&mut self, message: &SyncMessage) -> Option<SyncMessage> {
        // Update peer state
        if let Some(peer) = self.peers.get_mut(&message.sender) {
            peer.touch();
            peer.last_clock = message.clock.clone();
        }

        // Update our clock
        self.clock.update(&message.clock);

        match &message.payload {
            SyncPayload::Hello(info) => {
                self.register_peer_with_info(info.clone(), message.clock.clone());
                Some(self.create_hello(None))
            }
            SyncPayload::ClockRequest => Some(SyncMessage {
                id: self.tick(),
                sender: self.node_id,
                payload: SyncPayload::ClockResponse(self.clock.clone()),
                clock: self.clock.clone(),
                timestamp: current_timestamp_ms(),
            }),
            SyncPayload::Heartbeat => {
                // No response needed, just updated peer state above
                None
            }
            SyncPayload::Ack {
                message_id,
                applied_clock: _,
            } => {
                if let Some(peer) = self.peers.get_mut(&message.sender) {
                    peer.receive_ack(*message_id);
                }
                None
            }
            SyncPayload::Goodbye => {
                if let Some(peer) = self.peers.get_mut(&message.sender) {
                    peer.connection_state = PeerConnectionState::Gone;
                }
                None
            }
            // Other message types need application-level handling
            _ => None,
        }
    }

    /// Get list of stale peers that should be checked.
    pub fn stale_peers(&self) -> Vec<Uuid> {
        self.peers
            .iter()
            .filter(|(_, state)| state.is_stale(self.config.peer_timeout))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get peers that need heartbeats.
    pub fn peers_needing_heartbeat(&self) -> Vec<Uuid> {
        self.peers
            .iter()
            .filter(|(_, state)| {
                matches!(
                    state.connection_state,
                    PeerConnectionState::Synced | PeerConnectionState::Syncing
                ) && state
                    .last_seen
                    .map(|t| t.elapsed() > self.config.heartbeat_interval / 2)
                    .unwrap_or(true)
            })
            .map(|(id, _)| *id)
            .collect()
    }
}

/// Get current timestamp in milliseconds since epoch.
fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_state_creation() {
        let node_id = Uuid::new_v4();
        let state = SyncState::new(node_id);

        assert_eq!(state.node_id, node_id);
        assert!(state.peers.is_empty());
    }

    #[test]
    fn test_peer_registration() {
        let node_id = Uuid::new_v4();
        let mut state = SyncState::new(node_id);

        let peer_id = Uuid::new_v4();
        state.register_peer(peer_id, VectorClock::new());

        assert!(state.get_peer(&peer_id).is_some());
        assert_eq!(state.peers.len(), 1);
    }

    #[test]
    fn test_create_hello_message() {
        let node_id = Uuid::new_v4();
        let mut state = SyncState::new(node_id);

        let msg = state.create_hello(Some("Test Node".to_string()));

        assert_eq!(msg.sender, node_id);
        assert!(matches!(msg.payload, SyncPayload::Hello(_)));

        if let SyncPayload::Hello(info) = msg.payload {
            assert_eq!(info.name, Some("Test Node".to_string()));
        }
    }

    #[test]
    fn test_handle_hello_message() {
        let node1_id = Uuid::new_v4();
        let node2_id = Uuid::new_v4();

        let mut state1 = SyncState::new(node1_id);
        let mut state2 = SyncState::new(node2_id);

        // Node 2 sends hello to Node 1
        let hello = state2.create_hello(Some("Node 2".to_string()));
        let response = state1.handle_message(&hello);

        // Node 1 should have registered Node 2
        assert!(state1.get_peer(&node2_id).is_some());

        // Node 1 should respond with its own hello
        assert!(response.is_some());
        if let Some(msg) = response {
            assert!(matches!(msg.payload, SyncPayload::Hello(_)));
        }
    }

    #[test]
    fn test_clock_updates_on_message() {
        let node1_id = Uuid::new_v4();
        let node2_id = Uuid::new_v4();

        let mut state1 = SyncState::new(node1_id);
        let mut state2 = SyncState::new(node2_id);

        // Node 2 ticks multiple times
        state2.tick();
        state2.tick();
        state2.tick();

        let msg = state2.create_heartbeat();
        state1.handle_message(&msg);

        // Node 1's clock should have been updated
        assert!(!state1.clock.happens_before(&state2.clock));
    }

    #[test]
    fn test_peer_capabilities() {
        let caps = PeerCapabilities::default_capabilities();

        assert!(caps.compressed_deltas);
        assert!(caps.batch_operations);
        assert_eq!(caps.max_batch_size, 100);
    }

    #[test]
    fn test_goodbye_handling() {
        let node1_id = Uuid::new_v4();
        let node2_id = Uuid::new_v4();

        let mut state1 = SyncState::new(node1_id);
        let mut state2 = SyncState::new(node2_id);

        // First register peer
        state1.register_peer(node2_id, VectorClock::new());

        // Node 2 sends goodbye
        let goodbye = state2.create_goodbye();
        state1.handle_message(&goodbye);

        // Peer should be marked as gone
        let peer = state1.get_peer(&node2_id).unwrap();
        assert_eq!(peer.connection_state, PeerConnectionState::Gone);
    }

    #[test]
    fn test_delta_request_response() {
        let node_id = Uuid::new_v4();
        let mut state = SyncState::new(node_id);

        let delta_req = state.create_delta_request(VectorClock::new());

        assert!(matches!(
            delta_req.payload,
            SyncPayload::DeltaRequest { .. }
        ));

        let batch = DeltaBatch::new();
        let delta_resp = state.create_delta_response(batch, false);

        assert!(matches!(
            delta_resp.payload,
            SyncPayload::DeltaResponse {
                has_more: false,
                ..
            }
        ));
    }

    #[test]
    fn test_ack_rtt_tracking() {
        let node_id = Uuid::new_v4();
        let mut state = SyncState::new(node_id);

        let peer_id = Uuid::new_v4();
        state.register_peer(peer_id, VectorClock::new());

        // Simulate sending a message
        let peer = state.get_peer_mut(&peer_id).unwrap();
        peer.expect_ack(42);

        // Simulate receiving ack after some time
        std::thread::sleep(Duration::from_millis(10));
        peer.receive_ack(42);

        assert!(peer.rtt_estimate.is_some());
        assert!(peer.pending_acks.is_empty());
    }
}
