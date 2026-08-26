// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Predefined test scenarios for scale testing
//!
//! Provides ready-to-use scenarios for common testing patterns.

use crate::network::{LatencyModel, NetworkTopology};
use crate::report::ScaleTestReport;
use crate::simulator::{Simulation, SimulationResult, UserBehavior};
use std::time::Duration;

/// Configuration for a test scenario
#[derive(Debug, Clone)]
pub struct ScenarioConfig {
    /// Number of users to simulate
    pub user_count: usize,
    /// User behavior pattern
    pub behavior: UserBehavior,
    /// Network topology
    pub topology: NetworkTopology,
    /// Latency model
    pub latency: LatencyModel,
    /// Maximum test duration
    pub timeout: Duration,
    /// Convergence threshold
    pub convergence_threshold: f64,
}

impl Default for ScenarioConfig {
    fn default() -> Self {
        Self {
            user_count: 100,
            behavior: UserBehavior::Random {
                ops_per_second: 1.0,
            },
            topology: NetworkTopology::FullMesh,
            latency: LatencyModel::wan(),
            timeout: Duration::from_secs(30),
            convergence_threshold: crate::DEFAULT_CONVERGENCE_THRESHOLD,
        }
    }
}

/// Predefined test scenarios
#[derive(Debug, Clone)]
pub enum ScaleTestScenario {
    /// Simple counter increment test
    Counter { users: usize, ops_per_user: usize },

    /// Collaborative whiteboard simulation
    Whiteboard {
        users: usize,
        strokes_per_user: usize,
        points_per_stroke: usize,
    },

    /// Document editing simulation
    DocumentEditor { users: usize, edits_per_user: usize },

    /// High-frequency game state updates
    MultiplayerGame {
        users: usize,
        updates_per_second: f64,
        duration_seconds: f64,
    },

    /// Custom scenario with full configuration
    Custom(ScenarioConfig),
}

impl ScaleTestScenario {
    /// Create a 100-user test scenario
    pub fn scale_100() -> Self {
        Self::Counter {
            users: 100,
            ops_per_user: 10,
        }
    }

    /// Create a 1000-user test scenario
    pub fn scale_1000() -> Self {
        Self::Counter {
            users: 1000,
            ops_per_user: 10,
        }
    }

    /// Create a 10000-user test scenario
    pub fn scale_10000() -> Self {
        Self::Counter {
            users: 10000,
            ops_per_user: 5,
        }
    }

    /// Get the configuration for this scenario
    pub fn config(&self) -> ScenarioConfig {
        match self {
            Self::Counter {
                users,
                ops_per_user,
            } => ScenarioConfig {
                user_count: *users,
                behavior: UserBehavior::Random {
                    ops_per_second: *ops_per_user as f64,
                },
                topology: NetworkTopology::FullMesh,
                latency: LatencyModel::lan(),
                timeout: Duration::from_secs(60),
                ..Default::default()
            },

            Self::Whiteboard {
                users,
                strokes_per_user,
                points_per_stroke,
            } => ScenarioConfig {
                user_count: *users,
                behavior: UserBehavior::Drawing {
                    strokes_per_minute: *strokes_per_user as f64,
                    points_per_stroke: *points_per_stroke,
                },
                topology: NetworkTopology::FullMesh,
                latency: LatencyModel::wan(),
                timeout: Duration::from_secs(120),
                ..Default::default()
            },

            Self::DocumentEditor {
                users,
                edits_per_user,
            } => ScenarioConfig {
                user_count: *users,
                behavior: UserBehavior::Typing {
                    wpm: *edits_per_user as f64,
                    burst_factor: 1.5,
                },
                topology: NetworkTopology::FullMesh,
                latency: LatencyModel::wan(),
                timeout: Duration::from_secs(120),
                ..Default::default()
            },

            Self::MultiplayerGame {
                users,
                updates_per_second,
                duration_seconds,
            } => ScenarioConfig {
                user_count: *users,
                behavior: UserBehavior::Random {
                    ops_per_second: *updates_per_second,
                },
                topology: NetworkTopology::Star {
                    coordinator_index: 0,
                },
                latency: LatencyModel::lan(),
                timeout: Duration::from_secs_f64(*duration_seconds + 10.0),
                ..Default::default()
            },

            Self::Custom(config) => config.clone(),
        }
    }

    /// Run the scenario and return results
    pub fn run(&self) -> SimulationResult {
        let config = self.config();

        let mut sim = Simulation::new(config.user_count, config.behavior)
            .with_topology(config.topology)
            .with_latency(config.latency)
            .with_threshold(config.convergence_threshold);

        sim.run_until_convergence(config.timeout)
    }

    /// Run the scenario and generate a report
    pub fn run_with_report(&self) -> ScaleTestReport {
        let result = self.run();
        ScaleTestReport::from_result(self.name(), result)
    }

    /// Get the scenario name
    pub fn name(&self) -> String {
        match self {
            Self::Counter { users, .. } => format!("Counter ({})", users),
            Self::Whiteboard { users, .. } => format!("Whiteboard ({})", users),
            Self::DocumentEditor { users, .. } => format!("Document Editor ({})", users),
            Self::MultiplayerGame { users, .. } => format!("Multiplayer Game ({})", users),
            Self::Custom(config) => format!("Custom ({})", config.user_count),
        }
    }
}

/// Run multiple scenarios and collect results
pub fn run_scenarios(scenarios: &[ScaleTestScenario]) -> Vec<ScaleTestReport> {
    scenarios.iter().map(|s| s.run_with_report()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_config() {
        let scenario = ScaleTestScenario::scale_100();
        let config = scenario.config();
        assert_eq!(config.user_count, 100);
    }

    #[test]
    fn test_scenario_run() {
        let scenario = ScaleTestScenario::Counter {
            users: 5,
            ops_per_user: 2,
        };
        let result = scenario.run();
        assert_eq!(result.user_count, 5);
    }
}
