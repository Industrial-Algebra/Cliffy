//! Cliffy-Alive: Living, Self-Assembling UIs
//!
//! This module creates UI organisms that literally live and evolve using Amari's
//! cellular automata foundation. UIs are not static DOM structures but living
//! geometric fields that grow, adapt, and respond to user interaction through
//! biological-inspired mechanisms.

pub mod automata_bridge;
pub mod evolution;
pub mod frp_bridge;
pub mod metabolism;
pub mod nervous_system;
pub mod physics;
pub mod renderer;
pub mod ui_cell;
pub mod ui_organism;
pub mod wasm_bindings;

use cliffy_core::GeometricState;
// use amari_automata::{AutomatonField, AutomatonCell, CellularRule};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export main types
pub use evolution::*;
pub use metabolism::*;
pub use nervous_system::*;
pub use physics::*;
pub use renderer::*;
pub use ui_cell::*;
pub use ui_organism::*;

/// The fundamental energy unit in living UIs
pub type UIEnergy = f64;

/// Time units for biological processes
pub type UITime = f64;

/// Spatial coordinates in the UI organism field
pub type UICoordinates = (usize, usize);

/// Core trait for all living UI components
pub trait LivingComponent: Send + Sync {
    /// Get the current geometric state of this component
    fn geometric_state(&self) -> &GeometricState;

    /// Get the current energy level
    fn energy_level(&self) -> UIEnergy;

    /// Update the component with a time step
    fn step(&mut self, dt: UITime);

    /// Check if this component is alive
    fn is_alive(&self) -> bool;

    /// Get unique identifier
    fn id(&self) -> Uuid;

    /// Get the component's age
    fn age(&self) -> UITime;
}

/// Configuration for living UI behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliveConfig {
    /// Base energy consumption rate
    pub base_metabolism: UIEnergy,

    /// Energy threshold below which cells die
    pub death_threshold: UIEnergy,

    /// Energy threshold above which cells can reproduce
    pub reproduction_threshold: UIEnergy,

    /// Rate of energy transfer between neighboring cells
    pub energy_diffusion_rate: f64,

    /// Strength of geometric attraction forces
    pub attraction_strength: f64,

    /// Maximum velocity for geometric movement
    pub max_velocity: f64,

    /// Learning rate for adaptation
    pub learning_rate: f64,

    /// Mutation rate for evolution
    pub mutation_rate: f64,

    /// Size of the organism field
    pub field_dimensions: (usize, usize),
}

impl Default for AliveConfig {
    fn default() -> Self {
        Self {
            base_metabolism: 0.1,
            death_threshold: 0.0,
            reproduction_threshold: 100.0,
            energy_diffusion_rate: 0.05,
            attraction_strength: 1.0,
            max_velocity: 10.0,
            learning_rate: 0.01,
            mutation_rate: 0.001,
            field_dimensions: (50, 50),
        }
    }
}

/// Main entry point for creating living UIs
pub struct AliveUI {
    organism: UIOrganismField,
    #[allow(dead_code)]
    config: AliveConfig,
    time: UITime,
    renderer: Box<dyn UIRenderer>,
}

impl Default for AliveUI {
    fn default() -> Self {
        Self::new()
    }
}

impl AliveUI {
    /// Create a new living UI with default configuration
    pub fn new() -> Self {
        let config = AliveConfig::default();
        let organism = UIOrganismField::new(config.field_dimensions, config.clone());
        let renderer = Box::new(DOMRenderer::new());

        Self {
            organism,
            config,
            time: 0.0,
            renderer,
        }
    }

    /// Create a living UI with custom configuration
    pub fn with_config(config: AliveConfig) -> Self {
        let organism = UIOrganismField::new(config.field_dimensions, config.clone());
        let renderer = Box::new(DOMRenderer::new());

        Self {
            organism,
            config,
            time: 0.0,
            renderer,
        }
    }

    /// Set a custom renderer (builder pattern)
    pub fn with_renderer(mut self, renderer: Box<dyn UIRenderer>) -> Self {
        self.renderer = renderer;
        self
    }

    /// Set the renderer on an existing AliveUI
    pub fn set_renderer(&mut self, renderer: Box<dyn UIRenderer>) {
        self.renderer = renderer;
    }

    /// Plant a seed UI cell at specific coordinates
    pub fn plant_seed(
        &mut self,
        x: usize,
        y: usize,
        cell_type: UICellType,
    ) -> Result<Uuid, AliveError> {
        self.organism.plant_seed(x, y, cell_type)
    }

    /// Feed energy to a specific region to encourage growth
    pub fn feed_region(&mut self, x: usize, y: usize, radius: usize, energy: UIEnergy) {
        self.organism.feed_region(x, y, radius, energy);
    }

    /// Apply selection pressure to favor certain traits
    pub fn apply_selection_pressure(&mut self, pressure: SelectionPressure) {
        self.organism.apply_selection_pressure(pressure);
    }

    /// Step the living UI forward in time
    pub fn step(&mut self, dt: UITime) {
        self.time += dt;
        self.organism.step(dt);
    }

    /// Render the current state to the DOM
    pub fn render(&self) -> Result<(), RenderError> {
        // Clear the rendering surface first
        self.renderer.clear()?;

        // Render each cell in the organism
        for y in 0..self.organism.dimensions().1 {
            for x in 0..self.organism.dimensions().0 {
                if let Some(cell) = self.organism.get_cell(x, y) {
                    self.renderer.render_cell(cell, (x, y))?;
                }
            }
        }

        Ok(())
    }

    /// Get statistics about the living UI
    pub fn statistics(&self) -> AliveStatistics {
        AliveStatistics {
            total_cells: self.organism.cell_count(),
            living_cells: self.organism.living_cell_count(),
            total_energy: self.organism.total_energy(),
            average_age: self.organism.average_age(),
            generation: self.organism.generation(),
            time: self.time,
        }
    }

    /// Export the current organism state
    pub fn export_organism(&self) -> OrganismSnapshot {
        self.organism.snapshot()
    }

    /// Import an organism state
    pub fn import_organism(&mut self, snapshot: OrganismSnapshot) -> Result<(), AliveError> {
        self.organism.load_snapshot(snapshot)
    }
}

/// Statistics about the living UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliveStatistics {
    pub total_cells: usize,
    pub living_cells: usize,
    pub total_energy: UIEnergy,
    pub average_age: UITime,
    pub generation: u64,
    pub time: UITime,
}

/// Errors that can occur in the living UI system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AliveError {
    OutOfBounds { x: usize, y: usize },
    CellAlreadyExists { x: usize, y: usize },
    NoEnergySource,
    EvolutionFailed(String),
    RenderFailed(String),
    InvalidSnapshot(String),
}

impl std::fmt::Display for AliveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AliveError::OutOfBounds { x, y } => {
                write!(f, "Coordinates ({}, {}) are out of bounds", x, y)
            }
            AliveError::CellAlreadyExists { x, y } => {
                write!(f, "Cell already exists at ({}, {})", x, y)
            }
            AliveError::NoEnergySource => write!(f, "No energy source available"),
            AliveError::EvolutionFailed(msg) => write!(f, "Evolution failed: {}", msg),
            AliveError::RenderFailed(msg) => write!(f, "Rendering failed: {}", msg),
            AliveError::InvalidSnapshot(msg) => write!(f, "Invalid snapshot: {}", msg),
        }
    }
}

impl std::error::Error for AliveError {}

/// Convenience function to create a basic living UI
pub fn create_living_ui() -> AliveUI {
    AliveUI::new()
}

/// Convenience function to create a living button that grows and adapts
pub fn create_living_button(_text: &str) -> AliveUI {
    let mut ui = AliveUI::new();

    // Plant seeds in a button-like pattern
    let _ = ui.plant_seed(20, 20, UICellType::ButtonCore);
    let _ = ui.plant_seed(21, 20, UICellType::ButtonEdge);
    let _ = ui.plant_seed(19, 20, UICellType::ButtonEdge);
    let _ = ui.plant_seed(20, 21, UICellType::ButtonEdge);
    let _ = ui.plant_seed(20, 19, UICellType::ButtonEdge);

    // Feed the button region to encourage growth
    ui.feed_region(20, 20, 5, 50.0);

    ui
}

/// Convenience function to create a living form that adapts to user input
pub fn create_living_form() -> AliveUI {
    let mut ui = AliveUI::new();

    // Plant seeds for form components
    let _ = ui.plant_seed(10, 10, UICellType::InputField);
    let _ = ui.plant_seed(10, 20, UICellType::InputField);
    let _ = ui.plant_seed(10, 30, UICellType::ButtonCore);

    // Feed regions to encourage growth and connection
    ui.feed_region(10, 10, 3, 30.0);
    ui.feed_region(10, 20, 3, 30.0);
    ui.feed_region(10, 30, 3, 40.0);

    ui
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alive_ui_creation() {
        let ui = AliveUI::new();
        let stats = ui.statistics();

        assert_eq!(stats.total_cells, 0);
        assert_eq!(stats.living_cells, 0);
        assert_eq!(stats.time, 0.0);
    }

    #[test]
    fn test_seed_planting() {
        let mut ui = AliveUI::new();
        let result = ui.plant_seed(10, 10, UICellType::ButtonCore);

        assert!(result.is_ok());

        let stats = ui.statistics();
        assert_eq!(stats.total_cells, 1);
        assert_eq!(stats.living_cells, 1);
    }

    #[test]
    fn test_ui_step() {
        let mut ui = AliveUI::new();
        let _ = ui.plant_seed(10, 10, UICellType::ButtonCore);

        ui.step(1.0);

        let stats = ui.statistics();
        assert_eq!(stats.time, 1.0);
    }

    #[test]
    fn test_convenience_functions() {
        let button = create_living_button("Click me");
        let button_stats = button.statistics();

        assert!(button_stats.total_cells > 0);

        let form = create_living_form();
        let form_stats = form.statistics();

        assert!(form_stats.total_cells > 0);
    }
}
