// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! # cliffy-loadtest
//!
//! Scale testing framework for Cliffy that simulates 100-10,000+ concurrent users
//! with configurable network topology, latency, and operation patterns.
//!
//! ## Overview
//!
//! This crate provides tools for validating Cliffy's distributed state convergence
//! guarantees under realistic load conditions. It uses Rayon for parallel user
//! simulation and integrates with cliffy-test for invariant verification.
//!
//! ## Example
//!
//! ```rust,no_run
//! use cliffy_loadtest::{Simulation, UserBehavior, NetworkTopology, LatencyModel};
//!
//! // Create a simulation with 100 users
//! let behavior = UserBehavior::Random { ops_per_second: 10.0 };
//! let mut sim = Simulation::new(100, &behavior);
//!
//! // Run until all users converge
//! let result = sim.run_until_convergence(std::time::Duration::from_secs(30));
//!
//! assert!(result.converged);
//! println!("Convergence time: {:?}", result.convergence_time);
//! ```

pub mod metrics;
pub mod network;
pub mod report;
pub mod scenarios;
pub mod simulator;

pub use metrics::{ConvergenceMetrics, ThroughputMetrics};
pub use network::{LatencyModel, NetworkTopology};
pub use report::{ReportFormat, ScaleTestReport};
pub use scenarios::{ScaleTestScenario, ScenarioConfig};
pub use simulator::{SimulatedUser, Simulation, SimulationResult, UserBehavior};

/// Default convergence threshold (maximum acceptable state divergence)
pub const DEFAULT_CONVERGENCE_THRESHOLD: f64 = 1e-10;

/// Default timeout for convergence tests
pub const DEFAULT_CONVERGENCE_TIMEOUT_SECS: u64 = 60;
