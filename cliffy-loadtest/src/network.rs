//! Network topology and latency simulation
//!
//! Provides configurable network characteristics for realistic testing.

use rand::RngExt;
use std::time::Duration;

/// Network topology for simulation
#[derive(Debug, Clone, Default)]
pub enum NetworkTopology {
    /// Every peer connected to every other peer
    #[default]
    FullMesh,

    /// Star topology with central coordinator
    Star {
        /// ID of the coordinator node
        coordinator_index: usize,
    },

    /// Hierarchical with fanout
    Hierarchical {
        /// Number of children per node
        fanout: usize,
        /// Number of levels in the hierarchy
        levels: usize,
    },

    /// Random sparse graph
    Random {
        /// Probability of edge between any two nodes
        edge_probability: f64,
    },

    /// Ring topology (each node connects to next)
    Ring,
}

impl NetworkTopology {
    /// Get the list of peers that a given node can communicate with
    pub fn get_peers(&self, node_index: usize, total_nodes: usize) -> Vec<usize> {
        match self {
            Self::FullMesh => (0..total_nodes).filter(|&i| i != node_index).collect(),

            Self::Star { coordinator_index } => {
                if node_index == *coordinator_index {
                    // Coordinator connects to all
                    (0..total_nodes).filter(|&i| i != node_index).collect()
                } else {
                    // Others only connect to coordinator
                    vec![*coordinator_index]
                }
            }

            Self::Hierarchical { fanout, .. } => {
                let mut peers = Vec::new();

                // Parent (if not root)
                if node_index > 0 {
                    peers.push((node_index - 1) / fanout);
                }

                // Children
                let first_child = node_index * fanout + 1;
                for i in 0..*fanout {
                    let child = first_child + i;
                    if child < total_nodes {
                        peers.push(child);
                    }
                }

                peers
            }

            Self::Random { edge_probability } => {
                let mut rng = rand::rng();
                (0..total_nodes)
                    .filter(|&i| i != node_index && rng.random::<f64>() < *edge_probability)
                    .collect()
            }

            Self::Ring => {
                let mut peers = Vec::new();
                if total_nodes > 1 {
                    // Previous node
                    peers.push((node_index + total_nodes - 1) % total_nodes);
                    // Next node
                    if total_nodes > 2 {
                        peers.push((node_index + 1) % total_nodes);
                    }
                }
                peers
            }
        }
    }

    /// Calculate the diameter of the network (maximum hops between any two nodes)
    pub fn diameter(&self, total_nodes: usize) -> usize {
        match self {
            Self::FullMesh => 1,
            Self::Star { .. } => 2,
            Self::Hierarchical { fanout: _, levels } => levels * 2,
            Self::Random { edge_probability } => {
                // Approximate: log_fanout(n) where fanout = edge_probability * n
                let avg_fanout = (*edge_probability * total_nodes as f64).max(1.0);
                ((total_nodes as f64).ln() / avg_fanout.ln()).ceil() as usize
            }
            Self::Ring => total_nodes / 2,
        }
    }
}

/// Latency model for network simulation
#[derive(Debug, Clone)]
pub struct LatencyModel {
    /// Base latency in milliseconds
    pub base_ms: f64,
    /// Variance in milliseconds
    pub variance_ms: f64,
    /// Packet loss probability (0-1)
    pub loss_rate: f64,
    /// Jitter factor (0-1)
    pub jitter: f64,
}

impl Default for LatencyModel {
    fn default() -> Self {
        Self::wan()
    }
}

impl LatencyModel {
    /// Create a local network model (very low latency)
    pub fn local() -> Self {
        Self {
            base_ms: 1.0,
            variance_ms: 0.5,
            loss_rate: 0.0,
            jitter: 0.1,
        }
    }

    /// Create a LAN model
    pub fn lan() -> Self {
        Self {
            base_ms: 5.0,
            variance_ms: 2.0,
            loss_rate: 0.001,
            jitter: 0.2,
        }
    }

    /// Create a WAN model (typical internet)
    pub fn wan() -> Self {
        Self {
            base_ms: 50.0,
            variance_ms: 20.0,
            loss_rate: 0.01,
            jitter: 0.3,
        }
    }

    /// Create a mobile network model (high latency, variable)
    pub fn mobile() -> Self {
        Self {
            base_ms: 100.0,
            variance_ms: 50.0,
            loss_rate: 0.05,
            jitter: 0.5,
        }
    }

    /// Create a custom latency model
    pub fn custom(base_ms: f64, variance_ms: f64, loss_rate: f64, jitter: f64) -> Self {
        Self {
            base_ms,
            variance_ms,
            loss_rate,
            jitter,
        }
    }

    /// Get a random latency value based on the model
    pub fn get_latency(&self) -> Duration {
        let mut rng = rand::rng();
        let variance = (rng.random::<f64>() - 0.5) * 2.0 * self.variance_ms;
        let jitter_factor = 1.0 + (rng.random::<f64>() - 0.5) * 2.0 * self.jitter;
        let ms = ((self.base_ms + variance) * jitter_factor).max(0.0);
        Duration::from_secs_f64(ms / 1000.0)
    }

    /// Check if a packet should be dropped
    pub fn should_drop(&self) -> bool {
        rand::rng().random::<f64>() < self.loss_rate
    }

    /// Get the expected average latency
    pub fn average_latency(&self) -> Duration {
        Duration::from_secs_f64(self.base_ms / 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_mesh_topology() {
        let topology = NetworkTopology::FullMesh;
        let peers = topology.get_peers(0, 5);
        assert_eq!(peers, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_star_topology() {
        let topology = NetworkTopology::Star {
            coordinator_index: 0,
        };

        // Coordinator sees all
        let coordinator_peers = topology.get_peers(0, 5);
        assert_eq!(coordinator_peers, vec![1, 2, 3, 4]);

        // Others only see coordinator
        let leaf_peers = topology.get_peers(2, 5);
        assert_eq!(leaf_peers, vec![0]);
    }

    #[test]
    fn test_ring_topology() {
        let topology = NetworkTopology::Ring;
        let peers = topology.get_peers(2, 5);
        assert_eq!(peers, vec![1, 3]);
    }

    #[test]
    fn test_latency_model() {
        let model = LatencyModel::wan();

        // Get multiple samples and verify they're reasonable
        for _ in 0..100 {
            let latency = model.get_latency();
            assert!(latency.as_secs_f64() >= 0.0);
            assert!(latency.as_secs_f64() < 1.0); // Should be less than 1 second
        }
    }

    #[test]
    fn test_loss_rate() {
        let model = LatencyModel::custom(50.0, 10.0, 0.5, 0.1);

        let mut drops = 0;
        let samples = 1000;

        for _ in 0..samples {
            if model.should_drop() {
                drops += 1;
            }
        }

        // Should be roughly 50% (with some variance)
        let drop_rate = drops as f64 / samples as f64;
        assert!(drop_rate > 0.3 && drop_rate < 0.7);
    }
}
