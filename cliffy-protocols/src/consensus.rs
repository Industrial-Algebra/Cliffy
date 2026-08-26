// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Geometric consensus protocol implementations

use crate::{geometric_mean, serde_ga3, GeometricCRDT, OperationType};
use cliffy_core::GA3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// A message in the consensus protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMessage {
    pub sender_id: Uuid,
    #[serde(with = "serde_ga3")]
    pub proposal: GA3,
    pub round: u64,
    pub message_type: MessageType,
}

/// Types of consensus messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Propose(#[serde(with = "serde_ga3")] GA3),
    Vote(bool, #[serde(with = "serde_ga3")] GA3),
    Commit(#[serde(with = "serde_ga3")] GA3),
    Sync(GeometricCRDT),
}

/// A geometric consensus protocol implementation
pub struct GeometricConsensus {
    node_id: Uuid,
    current_round: u64,
    /// Proposals collected per round (for future Byzantine fault tolerance)
    #[allow(dead_code)]
    proposals: Arc<RwLock<HashMap<u64, Vec<GA3>>>>,
    /// Votes collected per round (for future Byzantine fault tolerance)
    #[allow(dead_code)]
    votes: Arc<RwLock<HashMap<u64, HashMap<Uuid, bool>>>>,
    /// Committed states per round (for state recovery)
    #[allow(dead_code)]
    committed_states: Arc<RwLock<HashMap<u64, GA3>>>,
    message_sender: broadcast::Sender<ConsensusMessage>,
    message_receiver: broadcast::Receiver<ConsensusMessage>,
    crdt_state: Arc<RwLock<GeometricCRDT>>,
}

impl GeometricConsensus {
    /// Create a new consensus protocol instance
    pub fn new(node_id: Uuid, initial_state: GA3) -> Self {
        let (sender, receiver) = broadcast::channel(1000);
        let crdt = GeometricCRDT::new(node_id, initial_state);

        Self {
            node_id,
            current_round: 0,
            proposals: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            committed_states: Arc::new(RwLock::new(HashMap::new())),
            message_sender: sender,
            message_receiver: receiver,
            crdt_state: Arc::new(RwLock::new(crdt)),
        }
    }

    /// Propose a value for consensus
    pub async fn propose(&mut self, value: GA3) -> Result<(), Box<dyn std::error::Error>> {
        let round = self.current_round;
        self.current_round += 1;

        let message = ConsensusMessage {
            sender_id: self.node_id,
            proposal: value.clone(),
            round,
            message_type: MessageType::Propose(value),
        };

        self.message_sender.send(message)?;
        Ok(())
    }

    /// Compute consensus from a set of proposals using geometric algebra
    pub async fn geometric_consensus(
        &self,
        proposals: &[GA3],
        threshold: f64,
    ) -> Result<GA3, Box<dyn std::error::Error>> {
        if proposals.is_empty() {
            return Ok(GA3::zero());
        }

        // Compute geometric mean of all proposals
        let consensus_value = geometric_mean(proposals);

        // Check if consensus meets threshold (based on geometric distance)
        let max_distance = proposals
            .iter()
            .map(|proposal| {
                let diff = &consensus_value - proposal;
                diff.magnitude()
            })
            .fold(0.0_f64, |acc, dist| acc.max(dist));

        if max_distance <= threshold {
            Ok(consensus_value)
        } else {
            // No consensus reached, return weighted geometric mean
            self.weighted_geometric_consensus(proposals).await
        }
    }

    /// Compute weighted geometric consensus based on magnitudes
    async fn weighted_geometric_consensus(
        &self,
        proposals: &[GA3],
    ) -> Result<GA3, Box<dyn std::error::Error>> {
        // Weight proposals by their geometric magnitude
        let weights: Vec<f64> = proposals.iter().map(|p| p.magnitude()).collect();

        let total_weight: f64 = weights.iter().sum();

        if total_weight == 0.0 {
            return Ok(GA3::zero());
        }

        // Simple weighted average for now
        let mut result = GA3::zero();
        for (proposal, weight) in proposals.iter().zip(weights.iter()) {
            let scaled: Vec<f64> = proposal
                .as_slice()
                .iter()
                .map(|&c| c * weight / total_weight)
                .collect();
            let scaled_mv = GA3::from_slice(&scaled);
            result = &result + &scaled_mv;
        }

        Ok(result)
    }

    /// Run a full consensus round
    pub async fn run_consensus_round(
        &mut self,
        proposal: GA3,
        participants: &[Uuid],
    ) -> Result<Option<GA3>, Box<dyn std::error::Error>> {
        let round = self.current_round;

        // Phase 1: Propose
        self.propose(proposal.clone()).await?;

        // Collect proposals from all participants
        let mut received_proposals = vec![proposal];
        let mut proposal_count = 1;

        while proposal_count < participants.len() {
            if let Ok(message) = self.message_receiver.recv().await {
                if message.round == round {
                    if let MessageType::Propose(prop) = message.message_type {
                        received_proposals.push(prop);
                        proposal_count += 1;
                    }
                }
            }
        }

        // Phase 2: Vote
        let consensus_candidate = self.geometric_consensus(&received_proposals, 0.1).await?;
        let vote_message = ConsensusMessage {
            sender_id: self.node_id,
            proposal: consensus_candidate.clone(),
            round,
            message_type: MessageType::Vote(true, consensus_candidate.clone()),
        };

        self.message_sender.send(vote_message)?;

        // Collect votes
        let mut votes = HashMap::new();
        votes.insert(self.node_id, true);

        while votes.len() < participants.len() {
            if let Ok(message) = self.message_receiver.recv().await {
                if message.round == round {
                    if let MessageType::Vote(vote, _) = message.message_type {
                        votes.insert(message.sender_id, vote);
                    }
                }
            }
        }

        // Phase 3: Commit if majority votes yes
        let yes_votes = votes.values().filter(|&&v| v).count();
        if yes_votes > participants.len() / 2 {
            let commit_message = ConsensusMessage {
                sender_id: self.node_id,
                proposal: consensus_candidate.clone(),
                round,
                message_type: MessageType::Commit(consensus_candidate.clone()),
            };

            self.message_sender.send(commit_message)?;

            // Update CRDT state
            let mut crdt_guard = self.crdt_state.write().await;
            let op =
                crdt_guard.create_operation(consensus_candidate.clone(), OperationType::Addition);
            crdt_guard.apply_operation(op);

            Ok(Some(consensus_candidate))
        } else {
            Ok(None)
        }
    }

    /// Sync CRDT state with another node
    pub async fn sync_crdt_state(
        &self,
        _other_node: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let crdt_guard = self.crdt_state.read().await;
        let sync_message = ConsensusMessage {
            sender_id: self.node_id,
            proposal: crdt_guard.state.clone(),
            round: self.current_round,
            message_type: MessageType::Sync(crdt_guard.clone()),
        };

        self.message_sender.send(sync_message)?;
        Ok(())
    }

    /// Handle an incoming sync message
    pub async fn handle_sync_message(&self, crdt_state: GeometricCRDT) {
        let mut local_crdt = self.crdt_state.write().await;
        *local_crdt = local_crdt.merge(&crdt_state);
    }

    /// Get the current consensus state
    pub async fn get_current_state(&self) -> GA3 {
        let crdt_guard = self.crdt_state.read().await;
        crdt_guard.state.clone()
    }
}

/// Compute the lattice join (least upper bound) of two multivectors
pub fn lattice_join(a: &GA3, b: &GA3) -> GA3 {
    let a_coeffs = a.as_slice();
    let b_coeffs = b.as_slice();

    let result_coeffs: Vec<f64> = a_coeffs
        .iter()
        .zip(b_coeffs.iter())
        .map(|(&ac, &bc)| ac.max(bc))
        .collect();

    GA3::from_slice(&result_coeffs)
}

/// Compute the lattice meet (greatest lower bound) of two multivectors
pub fn lattice_meet(a: &GA3, b: &GA3) -> GA3 {
    let a_coeffs = a.as_slice();
    let b_coeffs = b.as_slice();

    let result_coeffs: Vec<f64> = a_coeffs
        .iter()
        .zip(b_coeffs.iter())
        .map(|(&ac, &bc)| ac.min(bc))
        .collect();

    GA3::from_slice(&result_coeffs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_geometric_consensus_simple() {
        let proposals = vec![GA3::scalar(1.0), GA3::scalar(2.0), GA3::scalar(4.0)];

        let node_id = Uuid::new_v4();
        let consensus = GeometricConsensus::new(node_id, GA3::zero());

        let result = consensus
            .geometric_consensus(&proposals, 5.0) // Use larger threshold for test
            .await
            .unwrap();

        // Result should be some weighted average
        assert!(result.scalar_part() > 0.0);
    }

    #[test]
    fn test_lattice_operations() {
        let a = GA3::from_slice(&[1.0, 2.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let b = GA3::from_slice(&[2.0, 1.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        let join = lattice_join(&a, &b);
        let meet = lattice_meet(&a, &b);

        assert!((join.scalar_part() - 2.0).abs() < 1e-10);
        assert!((join.get(1) - 2.0).abs() < 1e-10);
        assert!((join.get(2) - 4.0).abs() < 1e-10);

        assert!((meet.scalar_part() - 1.0).abs() < 1e-10);
        assert!((meet.get(1) - 1.0).abs() < 1e-10);
        assert!((meet.get(2) - 3.0).abs() < 1e-10);
    }
}
