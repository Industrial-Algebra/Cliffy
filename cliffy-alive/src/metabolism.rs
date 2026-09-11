//! Energy and lifecycle management for living UI cells
//!
//! This module handles the biological aspects of UI cells including energy
//! metabolism, lifecycle stages, aging, and death. It ensures that UI elements
//! behave like living organisms with realistic energy dynamics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    ui_cell::{UICell, UICellType},
    UIEnergy, UITime,
};

/// Metabolic processes that can affect cell energy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetabolicProcess {
    /// Base cellular respiration
    Respiration,
    /// Energy for visual updates
    Rendering,
    /// Energy for user interaction responses
    Interaction,
    /// Energy for maintaining connections
    ConnectionMaintenance,
    /// Energy for reproduction
    Reproduction,
    /// Energy for learning and adaptation
    Learning,
    /// Energy for movement and physics
    Movement,
    /// Energy for memory storage
    Memory,
}

impl MetabolicProcess {
    /// Get the base energy cost multiplier for this process
    pub fn energy_multiplier(&self) -> f64 {
        match self {
            MetabolicProcess::Respiration => 1.0,
            MetabolicProcess::Rendering => 0.5,
            MetabolicProcess::Interaction => 2.0,
            MetabolicProcess::ConnectionMaintenance => 0.3,
            MetabolicProcess::Reproduction => 5.0,
            MetabolicProcess::Learning => 1.5,
            MetabolicProcess::Movement => 0.8,
            MetabolicProcess::Memory => 0.4,
        }
    }
}

/// Lifecycle stages of a UI cell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleStage {
    /// Just born, high growth rate
    Juvenile,
    /// Fully mature, stable energy consumption
    Adult,
    /// Aging, declining efficiency
    Elder,
    /// Near death, very low efficiency
    Senescent,
}

impl LifecycleStage {
    /// Get the energy efficiency multiplier for this stage
    pub fn efficiency_multiplier(&self) -> f64 {
        match self {
            LifecycleStage::Juvenile => 1.2,  // More efficient when young
            LifecycleStage::Adult => 1.0,     // Normal efficiency
            LifecycleStage::Elder => 0.8,     // Declining efficiency
            LifecycleStage::Senescent => 0.5, // Poor efficiency
        }
    }

    /// Get the growth rate multiplier for this stage
    pub fn growth_multiplier(&self) -> f64 {
        match self {
            LifecycleStage::Juvenile => 1.5,  // Fast growth
            LifecycleStage::Adult => 1.0,     // Normal growth
            LifecycleStage::Elder => 0.6,     // Slow growth
            LifecycleStage::Senescent => 0.2, // Very slow growth
        }
    }

    /// Determine lifecycle stage based on age
    pub fn from_age(age: UITime) -> Self {
        match age {
            a if a < 10.0 => LifecycleStage::Juvenile,
            a if a < 50.0 => LifecycleStage::Adult,
            a if a < 100.0 => LifecycleStage::Elder,
            _ => LifecycleStage::Senescent,
        }
    }
}

/// Manages the metabolism and lifecycle of UI cells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolismManager {
    /// Energy consumption rates for different processes
    process_rates: HashMap<MetabolicProcess, f64>,

    /// Age thresholds for lifecycle transitions
    age_thresholds: [UITime; 4], // Juvenile, Adult, Elder, Senescent

    /// Base metabolism configuration
    config: MetabolismConfig,
}

/// Configuration for metabolism behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolismConfig {
    /// Base energy consumption per time unit
    pub base_consumption: UIEnergy,

    /// Energy threshold below which cells start dying
    pub starvation_threshold: UIEnergy,

    /// Energy threshold above which cells can reproduce
    pub reproduction_threshold: UIEnergy,

    /// Rate at which cells age
    pub aging_rate: f64,

    /// Maximum age before forced death
    pub max_lifespan: UITime,

    /// Energy efficiency of different cell types
    pub type_efficiency: HashMap<UICellType, f64>,

    /// Energy storage capacity per cell type
    pub energy_capacity: HashMap<UICellType, UIEnergy>,
}

impl Default for MetabolismConfig {
    fn default() -> Self {
        let mut type_efficiency = HashMap::new();
        type_efficiency.insert(UICellType::ButtonCore, 0.8);
        type_efficiency.insert(UICellType::ButtonEdge, 0.9);
        type_efficiency.insert(UICellType::InputField, 0.7);
        type_efficiency.insert(UICellType::TextDisplay, 0.95);
        type_efficiency.insert(UICellType::Container, 0.85);
        type_efficiency.insert(UICellType::Spacer, 0.98);
        type_efficiency.insert(UICellType::Connector, 0.9);
        type_efficiency.insert(UICellType::Decoration, 0.6);
        type_efficiency.insert(UICellType::Sensor, 0.75);
        type_efficiency.insert(UICellType::Memory, 0.8);

        let mut energy_capacity = HashMap::new();
        energy_capacity.insert(UICellType::ButtonCore, 200.0);
        energy_capacity.insert(UICellType::ButtonEdge, 100.0);
        energy_capacity.insert(UICellType::InputField, 250.0);
        energy_capacity.insert(UICellType::TextDisplay, 150.0);
        energy_capacity.insert(UICellType::Container, 300.0);
        energy_capacity.insert(UICellType::Spacer, 50.0);
        energy_capacity.insert(UICellType::Connector, 75.0);
        energy_capacity.insert(UICellType::Decoration, 80.0);
        energy_capacity.insert(UICellType::Sensor, 120.0);
        energy_capacity.insert(UICellType::Memory, 180.0);

        Self {
            base_consumption: 1.0,
            starvation_threshold: 5.0,
            reproduction_threshold: 150.0,
            aging_rate: 1.0,
            max_lifespan: 200.0,
            type_efficiency,
            energy_capacity,
        }
    }
}

impl MetabolismManager {
    /// Create a new metabolism manager
    pub fn new(config: MetabolismConfig) -> Self {
        let mut process_rates = HashMap::new();

        // Initialize default process rates
        process_rates.insert(MetabolicProcess::Respiration, 1.0);
        process_rates.insert(MetabolicProcess::Rendering, 0.5);
        process_rates.insert(MetabolicProcess::Interaction, 0.0); // Variable
        process_rates.insert(MetabolicProcess::ConnectionMaintenance, 0.1);
        process_rates.insert(MetabolicProcess::Reproduction, 0.0); // Variable
        process_rates.insert(MetabolicProcess::Learning, 0.2);
        process_rates.insert(MetabolicProcess::Movement, 0.0); // Variable
        process_rates.insert(MetabolicProcess::Memory, 0.1);

        Self {
            process_rates,
            age_thresholds: [10.0, 50.0, 100.0, 200.0],
            config,
        }
    }

    /// Create a default metabolism manager
    pub fn with_default_config() -> Self {
        Self::new(MetabolismConfig::default())
    }

    /// Calculate total energy consumption for a cell
    pub fn calculate_energy_consumption(
        &self,
        cell: &UICell,
        active_processes: &[MetabolicProcess],
        dt: UITime,
    ) -> UIEnergy {
        let base_cost = self.config.base_consumption * dt;
        let cell_type = cell.cell_type();
        let lifecycle_stage = LifecycleStage::from_age(cell.age());

        // Get cell type efficiency
        let type_efficiency = self
            .config
            .type_efficiency
            .get(&cell_type)
            .copied()
            .unwrap_or(1.0);

        // Get genetic efficiency
        let genetic_efficiency = cell.genome().get_gene("energy_efficiency");

        // Calculate total efficiency
        let total_efficiency =
            type_efficiency * genetic_efficiency * lifecycle_stage.efficiency_multiplier();

        // Calculate process costs
        let mut total_cost = base_cost / total_efficiency;

        for process in active_processes {
            let process_rate = self.process_rates.get(process).copied().unwrap_or(0.0);
            let process_cost = process_rate * process.energy_multiplier() * dt;
            total_cost += process_cost / total_efficiency;
        }

        total_cost
    }

    /// Apply aging effects to a cell
    pub fn apply_aging(&self, cell: &mut UICell, dt: UITime) {
        let age_increase = dt * self.config.aging_rate;

        // Aging affects all cells, but efficiency can slow it down
        let genetic_efficiency = cell.genome().get_gene("energy_efficiency");
        let actual_aging = age_increase / (1.0 + genetic_efficiency * 0.5);

        // Note: Age is already tracked by the cell itself, this is for additional aging effects

        // Apply age-related energy decay
        let lifecycle_stage = LifecycleStage::from_age(cell.age());
        match lifecycle_stage {
            LifecycleStage::Elder => {
                // Start losing energy due to aging
                let aging_cost = actual_aging * 0.1;
                cell.consume_energy(aging_cost);
            }
            LifecycleStage::Senescent => {
                // Significant energy loss due to aging
                let aging_cost = actual_aging * 0.5;
                cell.consume_energy(aging_cost);
            }
            _ => {}
        }

        // Check for natural death from old age
        if cell.age() > self.config.max_lifespan {
            // Force cell to start dying
            // Note: This would require access to cell's internal state
            // In practice, this would be handled by the cell's step function
        }
    }

    /// Check if a cell should enter starvation mode
    pub fn is_starving(&self, cell: &UICell) -> bool {
        cell.energy_level() < self.config.starvation_threshold
    }

    /// Check if a cell has enough energy to reproduce
    pub fn can_reproduce(&self, cell: &UICell) -> bool {
        cell.energy_level() > self.config.reproduction_threshold &&
        cell.age() > 5.0 && // Must be at least 5 time units old
        matches!(LifecycleStage::from_age(cell.age()),
                LifecycleStage::Juvenile | LifecycleStage::Adult)
    }

    /// Calculate energy cost for reproduction
    pub fn reproduction_cost(&self, cell: &UICell) -> UIEnergy {
        let base_cost = self.config.reproduction_threshold * 0.8;
        let cell_type_cost = cell.cell_type().base_energy_cost() * 10.0;
        let genetic_factor = 2.0 - cell.genome().get_gene("energy_efficiency");

        base_cost + cell_type_cost * genetic_factor
    }

    /// Get the maximum energy capacity for a cell type
    pub fn energy_capacity(&self, cell_type: UICellType) -> UIEnergy {
        self.config
            .energy_capacity
            .get(&cell_type)
            .copied()
            .unwrap_or(100.0)
    }

    /// Apply metabolic process to a cell
    pub fn apply_process(
        &self,
        cell: &mut UICell,
        process: MetabolicProcess,
        intensity: f64,
        dt: UITime,
    ) -> bool {
        let base_cost = self.process_rates.get(&process).copied().unwrap_or(0.0);
        let total_cost = base_cost * process.energy_multiplier() * intensity * dt;

        cell.consume_energy(total_cost)
    }

    /// Get the lifecycle stage of a cell
    pub fn lifecycle_stage(&self, cell: &UICell) -> LifecycleStage {
        LifecycleStage::from_age(cell.age())
    }

    /// Calculate metabolic efficiency for a cell
    pub fn metabolic_efficiency(&self, cell: &UICell) -> f64 {
        let cell_type = cell.cell_type();
        let lifecycle_stage = self.lifecycle_stage(cell);

        let type_efficiency = self
            .config
            .type_efficiency
            .get(&cell_type)
            .copied()
            .unwrap_or(1.0);

        let genetic_efficiency = cell.genome().get_gene("energy_efficiency");
        let age_efficiency = lifecycle_stage.efficiency_multiplier();

        type_efficiency * genetic_efficiency * age_efficiency
    }

    /// Calculate energy regeneration rate
    pub fn energy_regeneration_rate(&self, cell: &UICell) -> UIEnergy {
        let base_regen = 0.1; // Base regeneration per time unit
        let efficiency = self.metabolic_efficiency(cell);
        let health_factor = (cell.energy_level() / 100.0).clamp(0.1, 1.0);

        base_regen * efficiency * health_factor
    }

    /// Update metabolism configuration
    pub fn update_config(&mut self, new_config: MetabolismConfig) {
        self.config = new_config;
    }

    /// Get current configuration
    pub fn config(&self) -> &MetabolismConfig {
        &self.config
    }
}

/// Helper struct for tracking cell vital signs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellVitals {
    pub energy_level: UIEnergy,
    pub energy_capacity: UIEnergy,
    pub age: UITime,
    pub lifecycle_stage: LifecycleStage,
    pub metabolic_efficiency: f64,
    pub health_status: HealthStatus,
    pub time_to_reproduction: Option<UITime>,
    pub time_to_death: Option<UITime>,
}

/// Health status of a cell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Cell is healthy and thriving
    Healthy,
    /// Cell is functioning but stressed
    Stressed,
    /// Cell is struggling to survive
    Struggling,
    /// Cell is critically low on resources
    Critical,
    /// Cell is dying
    Dying,
}

impl CellVitals {
    /// Calculate vitals for a cell
    pub fn calculate(cell: &UICell, metabolism: &MetabolismManager) -> Self {
        let energy_level = cell.energy_level();
        let energy_capacity = metabolism.energy_capacity(cell.cell_type());
        let age = cell.age();
        let lifecycle_stage = metabolism.lifecycle_stage(cell);
        let metabolic_efficiency = metabolism.metabolic_efficiency(cell);

        // Determine health status
        let energy_ratio = energy_level / energy_capacity;
        let health_status = match energy_ratio {
            r if r > 0.8 => HealthStatus::Healthy,
            r if r > 0.5 => HealthStatus::Stressed,
            r if r > 0.2 => HealthStatus::Struggling,
            r if r > 0.05 => HealthStatus::Critical,
            _ => HealthStatus::Dying,
        };

        // Calculate time to reproduction
        let time_to_reproduction = if metabolism.can_reproduce(cell) {
            Some(0.0)
        } else if energy_level < metabolism.config.reproduction_threshold {
            let energy_needed = metabolism.config.reproduction_threshold - energy_level;
            let regen_rate = metabolism.energy_regeneration_rate(cell);
            if regen_rate > 0.0 {
                Some(energy_needed / regen_rate)
            } else {
                None
            }
        } else {
            None
        };

        // Calculate time to death (very rough estimate)
        let time_to_death = if energy_level > metabolism.config.starvation_threshold {
            let consumption_rate = metabolism.calculate_energy_consumption(
                cell,
                &[MetabolicProcess::Respiration],
                1.0,
            );
            let regen_rate = metabolism.energy_regeneration_rate(cell);
            let net_loss = consumption_rate - regen_rate;

            if net_loss > 0.0 {
                Some((energy_level - metabolism.config.starvation_threshold) / net_loss)
            } else {
                None
            }
        } else {
            Some(0.0) // Already starving
        };

        Self {
            energy_level,
            energy_capacity,
            age,
            lifecycle_stage,
            metabolic_efficiency,
            health_status,
            time_to_reproduction,
            time_to_death,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_cell::UICellType;
    use cliffy_core::GA3;

    #[test]
    fn test_lifecycle_stages() {
        assert_eq!(LifecycleStage::from_age(5.0), LifecycleStage::Juvenile);
        assert_eq!(LifecycleStage::from_age(25.0), LifecycleStage::Adult);
        assert_eq!(LifecycleStage::from_age(75.0), LifecycleStage::Elder);
        assert_eq!(LifecycleStage::from_age(150.0), LifecycleStage::Senescent);
    }

    #[test]
    fn test_metabolic_process_costs() {
        assert_eq!(MetabolicProcess::Respiration.energy_multiplier(), 1.0);
        assert_eq!(MetabolicProcess::Reproduction.energy_multiplier(), 5.0);
        assert_eq!(MetabolicProcess::Interaction.energy_multiplier(), 2.0);
    }

    #[test]
    fn test_metabolism_manager() {
        let manager = MetabolismManager::with_default_config();
        let position = GA3::scalar(1.0);
        let mut cell = UICell::new_at_position(UICellType::ButtonCore, position);

        let consumption =
            manager.calculate_energy_consumption(&cell, &[MetabolicProcess::Respiration], 1.0);

        assert!(consumption > 0.0);

        // Add energy above reproduction threshold and age the cell
        cell.add_energy(180.0); // Total: 20 + 180 = 200 > 150 threshold
        cell.step(6.0); // Age to 6.0 > 5.0 required
        assert!(manager.can_reproduce(&cell));
    }

    #[test]
    fn test_cell_vitals() {
        let manager = MetabolismManager::with_default_config();
        let position = GA3::scalar(1.0);
        let mut cell = UICell::new_at_position(UICellType::ButtonCore, position);

        // Add energy to reach Healthy status (> 0.8 * 200 capacity = > 160 energy)
        cell.add_energy(145.0); // Total: 20 + 145 = 165 > 160

        let vitals = CellVitals::calculate(&cell, &manager);

        assert_eq!(vitals.lifecycle_stage, LifecycleStage::Juvenile);
        assert!(vitals.energy_level > 0.0);
        assert!(vitals.metabolic_efficiency > 0.0);
        assert_eq!(vitals.health_status, HealthStatus::Healthy);
    }

    #[test]
    fn test_energy_capacity() {
        let manager = MetabolismManager::with_default_config();

        let button_capacity = manager.energy_capacity(UICellType::ButtonCore);
        let spacer_capacity = manager.energy_capacity(UICellType::Spacer);

        assert!(button_capacity > spacer_capacity);
        assert_eq!(button_capacity, 200.0);
        assert_eq!(spacer_capacity, 50.0);
    }

    #[test]
    fn test_reproduction_requirements() {
        let manager = MetabolismManager::with_default_config();
        let position = GA3::scalar(1.0);
        let mut cell = UICell::new_at_position(UICellType::ButtonCore, position);

        // Cell with lots of energy and sufficient age should be able to reproduce
        cell.add_energy(200.0);
        cell.step(6.0); // Age to 6.0 > 5.0 required
        assert!(manager.can_reproduce(&cell));

        // Drain energy below threshold (150.0)
        let current_energy = cell.energy_level();
        cell.consume_energy(current_energy - 100.0); // Leave only 100.0 < 150.0 threshold
        assert!(!manager.can_reproduce(&cell));
    }
}
