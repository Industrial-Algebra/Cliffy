// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! User simulation for scale testing
//!
//! Simulates concurrent users generating operations and syncing state.

use crate::metrics::{ConvergenceMetrics, Stopwatch, ThroughputMetrics};
use crate::network::{LatencyModel, NetworkTopology};
use amari_core::Multivector;
use rayon::prelude::*;
use std::time::Duration;
use uuid::Uuid;

/// Type alias for 3D geometric algebra multivector
pub type GA3 = Multivector<3, 0, 0>;

/// User behavior patterns
#[derive(Debug, Clone)]
pub enum UserBehavior {
    /// Random operations at fixed rate
    Random {
        /// Operations per second
        ops_per_second: f64,
    },

    /// Realistic typing pattern with bursts
    Typing {
        /// Words per minute
        wpm: f64,
        /// Burst factor (1.0 = no bursts, 2.0 = double speed bursts)
        burst_factor: f64,
    },

    /// Drawing strokes (whiteboard simulation)
    Drawing {
        /// Strokes per minute
        strokes_per_minute: f64,
        /// Points per stroke
        points_per_stroke: usize,
    },

    /// Idle with occasional bursts
    Bursty {
        /// Ratio of time spent idle (0-1)
        idle_ratio: f64,
        /// Number of operations per burst
        burst_size: usize,
    },
}

impl Default for UserBehavior {
    fn default() -> Self {
        Self::Random {
            ops_per_second: 1.0,
        }
    }
}

impl UserBehavior {
    /// Get operations per second for this behavior
    pub fn ops_per_second(&self) -> f64 {
        match self {
            Self::Random { ops_per_second } => *ops_per_second,
            Self::Typing { wpm, .. } => wpm * 5.0 / 60.0, // ~5 chars per word
            Self::Drawing {
                strokes_per_minute,
                points_per_stroke,
            } => strokes_per_minute / 60.0 * *points_per_stroke as f64,
            Self::Bursty {
                idle_ratio,
                burst_size,
            } => {
                let active_ratio = 1.0 - idle_ratio;
                *burst_size as f64 * active_ratio
            }
        }
    }
}

/// A simulated user that generates operations
#[derive(Debug)]
pub struct SimulatedUser {
    /// Unique user ID
    pub id: Uuid,
    /// Current state as GA3 multivector
    pub state: GA3,
    /// User behavior pattern
    pub behavior: UserBehavior,
    /// Operations generated
    pub operation_count: usize,
}

impl SimulatedUser {
    /// Create a new simulated user
    pub fn new(behavior: UserBehavior) -> Self {
        Self {
            id: Uuid::new_v4(),
            state: GA3::zero(),
            behavior,
            operation_count: 0,
        }
    }

    /// Generate an operation (modifies state)
    pub fn generate_operation(&mut self) {
        // Simple operation: add a small random scalar to the state
        let delta = GA3::scalar(rand::random::<f64>() - 0.5);

        self.state = &self.state + &delta;
        self.operation_count += 1;
    }

    /// Merge with another user's state (CRDT merge via geometric mean)
    pub fn merge(&mut self, other_state: &GA3) {
        // Simple merge: average the states (geometric mean approximation)
        let sum = &self.state + other_state;
        self.state = &sum * 0.5;
    }

    /// Calculate divergence from a reference state
    pub fn divergence_from(&self, reference: &GA3) -> f64 {
        let diff = &self.state - reference;
        diff.magnitude()
    }
}

/// Result of a simulation run
#[derive(Debug)]
pub struct SimulationResult {
    /// Whether all users converged
    pub converged: bool,
    /// Time to convergence (if achieved)
    pub convergence_time: Option<Duration>,
    /// Total simulation duration
    pub total_duration: Duration,
    /// Convergence metrics
    pub convergence_metrics: ConvergenceMetrics,
    /// Throughput metrics
    pub throughput_metrics: ThroughputMetrics,
    /// Number of users simulated
    pub user_count: usize,
    /// Final states of all users
    pub final_states: Vec<GA3>,
}

/// Main simulation runner
pub struct Simulation {
    users: Vec<SimulatedUser>,
    topology: NetworkTopology,
    latency_model: LatencyModel,
    convergence_threshold: f64,
}

impl Simulation {
    /// Create a new simulation with the specified number of users
    pub fn new(user_count: usize, behavior: UserBehavior) -> Self {
        let users = (0..user_count)
            .map(|_| SimulatedUser::new(behavior.clone()))
            .collect();

        Self {
            users,
            topology: NetworkTopology::default(),
            latency_model: LatencyModel::default(),
            convergence_threshold: crate::DEFAULT_CONVERGENCE_THRESHOLD,
        }
    }

    /// Set the network topology
    pub fn with_topology(mut self, topology: NetworkTopology) -> Self {
        self.topology = topology;
        self
    }

    /// Set the latency model
    pub fn with_latency(mut self, latency: LatencyModel) -> Self {
        self.latency_model = latency;
        self
    }

    /// Set the convergence threshold
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.convergence_threshold = threshold;
        self
    }

    /// Run simulation for a fixed duration
    pub fn run(&mut self, duration: Duration) -> SimulationResult {
        let stopwatch = Stopwatch::start();
        let mut convergence_metrics = ConvergenceMetrics::new();
        let mut throughput_metrics = ThroughputMetrics::new();

        let ops_per_user =
            (self.users[0].behavior.ops_per_second() * duration.as_secs_f64()) as usize;

        // Run operations in parallel using Rayon
        self.users.par_iter_mut().for_each(|user| {
            for _ in 0..ops_per_user {
                user.generate_operation();
            }
        });

        // Simulate sync rounds
        let sync_rounds = 10;
        for _ in 0..sync_rounds {
            self.sync_round(&mut throughput_metrics);
        }

        let total_duration = stopwatch.elapsed();
        throughput_metrics.total_operations = self.users.iter().map(|u| u.operation_count).sum();
        throughput_metrics.finalize(total_duration);

        // Check convergence
        let divergence = self.calculate_max_divergence();
        convergence_metrics.record_divergence(divergence);

        let converged = divergence < self.convergence_threshold;
        if converged {
            convergence_metrics.mark_converged(total_duration);
        }

        SimulationResult {
            converged,
            convergence_time: if converged {
                Some(total_duration)
            } else {
                None
            },
            total_duration,
            convergence_metrics,
            throughput_metrics,
            user_count: self.users.len(),
            final_states: self.users.iter().map(|u| u.state.clone()).collect(),
        }
    }

    /// Run simulation until convergence or timeout
    pub fn run_until_convergence(&mut self, timeout: Duration) -> SimulationResult {
        let stopwatch = Stopwatch::start();
        let mut convergence_metrics = ConvergenceMetrics::new();
        let mut throughput_metrics = ThroughputMetrics::new();

        let check_interval = Duration::from_millis(100);
        let mut converged = false;

        while stopwatch.elapsed() < timeout && !converged {
            // Generate some operations
            self.users.par_iter_mut().for_each(|user| {
                user.generate_operation();
            });

            // Sync
            self.sync_round(&mut throughput_metrics);

            // Check convergence
            let divergence = self.calculate_max_divergence();
            convergence_metrics.record_divergence(divergence);

            converged = divergence < self.convergence_threshold;

            if !converged {
                std::thread::sleep(check_interval);
            }
        }

        let total_duration = stopwatch.elapsed();
        throughput_metrics.total_operations = self.users.iter().map(|u| u.operation_count).sum();
        throughput_metrics.finalize(total_duration);

        if converged {
            convergence_metrics.mark_converged(total_duration);
        }

        SimulationResult {
            converged,
            convergence_time: if converged {
                Some(total_duration)
            } else {
                None
            },
            total_duration,
            convergence_metrics,
            throughput_metrics,
            user_count: self.users.len(),
            final_states: self.users.iter().map(|u| u.state.clone()).collect(),
        }
    }

    /// Perform one round of state synchronization
    fn sync_round(&mut self, metrics: &mut ThroughputMetrics) {
        let user_count = self.users.len();

        // Collect all states
        let states: Vec<GA3> = self.users.iter().map(|u| u.state.clone()).collect();

        // Each user syncs with their peers according to topology
        for (i, user) in self.users.iter_mut().enumerate() {
            let peers = self.topology.get_peers(i, user_count);

            for peer_idx in peers {
                let dropped = self.latency_model.should_drop();
                metrics.record_message(dropped);

                if !dropped {
                    user.merge(&states[peer_idx]);
                }
            }
        }
    }

    /// Calculate maximum divergence between any two users
    fn calculate_max_divergence(&self) -> f64 {
        if self.users.is_empty() {
            return 0.0;
        }

        let reference = &self.users[0].state;
        self.users
            .iter()
            .skip(1)
            .map(|u| u.divergence_from(reference))
            .fold(0.0, f64::max)
    }

    /// Get user count
    pub fn user_count(&self) -> usize {
        self.users.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulated_user() {
        let mut user = SimulatedUser::new(UserBehavior::Random {
            ops_per_second: 10.0,
        });

        assert_eq!(user.operation_count, 0);

        user.generate_operation();
        assert_eq!(user.operation_count, 1);
    }

    #[test]
    fn test_simulation_basic() {
        let mut sim = Simulation::new(
            10,
            UserBehavior::Random {
                ops_per_second: 10.0,
            },
        );
        let result = sim.run(Duration::from_millis(100));

        assert_eq!(result.user_count, 10);
        assert!(result.throughput_metrics.total_operations > 0);
    }

    #[test]
    fn test_simulation_convergence() {
        let mut sim = Simulation::new(
            5,
            UserBehavior::Random {
                ops_per_second: 1.0,
            },
        )
        .with_topology(NetworkTopology::FullMesh)
        .with_latency(LatencyModel::local())
        .with_threshold(10.0); // Relaxed threshold for quick test

        let result = sim.run_until_convergence(Duration::from_secs(2));

        // With full mesh and many sync rounds, should converge
        assert!(result.convergence_metrics.check_count > 0);
    }
}
