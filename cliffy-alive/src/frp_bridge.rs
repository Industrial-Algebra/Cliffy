//! FRP Bridge — Connects the step-based simulation to cliffy-core's reactive primitives
//!
//! This module provides reactive wrappers around the imperative AliveUI,
//! exposing `Behavior<T>` for continuous organism state and `Event<T>` for
//! discrete lifecycle events (cell birth, death, generation completion).
//!
//! # Example
//!
//! ```rust,no_run
//! use cliffy_alive::frp_bridge::ReactiveAliveUI;
//! use cliffy_alive::UICellType;
//!
//! let mut ui = ReactiveAliveUI::new();
//! ui.plant_seed(10, 10, UICellType::ButtonCore).unwrap();
//!
//! // Subscribe to cell count changes
//! let _sub = ui.behaviors().cell_count.subscribe(|count| {
//!     println!("Cell count: {}", count);
//! });
//!
//! // Subscribe to cell birth events
//! let _sub = ui.events().cell_born.subscribe(|evt| {
//!     println!("Cell born at ({}, {})", evt.x, evt.y);
//! });
//!
//! ui.step(1.0);
//! ```

use cliffy_core::{behavior, event, Behavior, Event};
use uuid::Uuid;

use crate::{
    AliveConfig, AliveError, AliveStatistics, AliveUI, OrganismSnapshot, SelectionPressure,
    UICellType, UIEnergy, UITime,
};

/// Event emitted when a new cell is born
#[derive(Debug, Clone)]
pub struct CellBornEvent {
    pub id: Uuid,
    pub x: usize,
    pub y: usize,
    pub cell_type: UICellType,
}

/// Event emitted when a cell dies
#[derive(Debug, Clone)]
pub struct CellDiedEvent {
    pub x: usize,
    pub y: usize,
    pub cell_type: UICellType,
}

/// Event emitted when a generation completes
#[derive(Debug, Clone)]
pub struct GenerationEvent {
    pub generation: u64,
    pub living_cells: usize,
    pub total_energy: f64,
}

/// Organism-level reactive state (continuously varying values)
pub struct OrganismBehavior {
    pub cell_count: Behavior<usize>,
    pub total_energy: Behavior<f64>,
    pub generation: Behavior<usize>,
    pub average_age: Behavior<f64>,
    pub time: Behavior<f64>,
}

/// Organism-level discrete events
pub struct AliveEvents {
    pub cell_born: Event<CellBornEvent>,
    pub cell_died: Event<CellDiedEvent>,
    pub generation_complete: Event<GenerationEvent>,
}

/// Reactive wrapper around AliveUI
///
/// Wraps the imperative step-based simulation, updating Behaviors and
/// emitting Events by diffing pre/post state after each step.
pub struct ReactiveAliveUI {
    inner: AliveUI,
    behaviors: OrganismBehavior,
    events: AliveEvents,
    /// Track cell positions for birth/death detection
    cell_snapshot: Vec<(usize, usize, Uuid, UICellType)>,
    last_generation: u64,
}

impl Default for ReactiveAliveUI {
    fn default() -> Self {
        Self::new()
    }
}

impl ReactiveAliveUI {
    /// Create a new reactive living UI with default configuration
    pub fn new() -> Self {
        Self::from_inner(AliveUI::new())
    }

    /// Create a reactive living UI with custom configuration
    pub fn with_config(config: AliveConfig) -> Self {
        Self::from_inner(AliveUI::with_config(config))
    }

    fn from_inner(inner: AliveUI) -> Self {
        let behaviors = OrganismBehavior {
            cell_count: behavior(0usize),
            total_energy: behavior(0.0f64),
            generation: behavior(0usize),
            average_age: behavior(0.0f64),
            time: behavior(0.0f64),
        };

        let events = AliveEvents {
            cell_born: event::<CellBornEvent>(),
            cell_died: event::<CellDiedEvent>(),
            generation_complete: event::<GenerationEvent>(),
        };

        Self {
            inner,
            behaviors,
            events,
            cell_snapshot: Vec::new(),
            last_generation: 0,
        }
    }

    /// Get the reactive behaviors
    pub fn behaviors(&self) -> &OrganismBehavior {
        &self.behaviors
    }

    /// Get the event streams
    pub fn events(&self) -> &AliveEvents {
        &self.events
    }

    /// Plant a seed cell and emit a birth event
    pub fn plant_seed(
        &mut self,
        x: usize,
        y: usize,
        cell_type: UICellType,
    ) -> Result<Uuid, AliveError> {
        let id = self.inner.plant_seed(x, y, cell_type)?;

        // Emit birth event
        self.events.cell_born.emit(CellBornEvent {
            id,
            x,
            y,
            cell_type,
        });

        // Update cell snapshot
        self.cell_snapshot.push((x, y, id, cell_type));

        // Update behaviors
        self.sync_behaviors();

        Ok(id)
    }

    /// Feed energy to a region
    pub fn feed_region(&mut self, x: usize, y: usize, radius: usize, energy: UIEnergy) {
        self.inner.feed_region(x, y, radius, energy);
        self.sync_behaviors();
    }

    /// Apply selection pressure
    pub fn apply_selection_pressure(&mut self, pressure: SelectionPressure) {
        self.inner.apply_selection_pressure(pressure);
    }

    /// Step the simulation and update all reactive state
    pub fn step(&mut self, dt: UITime) {
        // Snapshot before step
        let pre_cells = self.snapshot_cells();

        self.inner.step(dt);

        // Snapshot after step
        let post_cells = self.snapshot_cells();

        // Detect births (in post but not in pre)
        for (x, y, id, cell_type) in &post_cells {
            if !pre_cells.iter().any(|(_, _, pre_id, _)| pre_id == id) {
                self.events.cell_born.emit(CellBornEvent {
                    id: *id,
                    x: *x,
                    y: *y,
                    cell_type: *cell_type,
                });
            }
        }

        // Detect deaths (in pre but not in post)
        for (x, y, id, cell_type) in &pre_cells {
            if !post_cells.iter().any(|(_, _, post_id, _)| post_id == id) {
                self.events.cell_died.emit(CellDiedEvent {
                    x: *x,
                    y: *y,
                    cell_type: *cell_type,
                });
            }
        }

        self.cell_snapshot = post_cells;

        // Check for generation change
        let stats = self.inner.statistics();
        if stats.generation != self.last_generation {
            self.events.generation_complete.emit(GenerationEvent {
                generation: stats.generation,
                living_cells: stats.living_cells,
                total_energy: stats.total_energy,
            });
            self.last_generation = stats.generation;
        }

        // Update behaviors
        self.sync_behaviors();
    }

    /// Render the current state
    pub fn render(&self) -> Result<(), crate::RenderError> {
        self.inner.render()
    }

    /// Get current statistics
    pub fn statistics(&self) -> AliveStatistics {
        self.inner.statistics()
    }

    /// Export organism snapshot
    pub fn export_organism(&self) -> OrganismSnapshot {
        self.inner.export_organism()
    }

    /// Import organism snapshot
    pub fn import_organism(&mut self, snapshot: OrganismSnapshot) -> Result<(), AliveError> {
        self.inner.import_organism(snapshot)?;
        self.cell_snapshot = self.snapshot_cells();
        self.sync_behaviors();
        Ok(())
    }

    /// Get a reference to the inner AliveUI
    pub fn inner(&self) -> &AliveUI {
        &self.inner
    }

    /// Get a mutable reference to the inner AliveUI
    pub fn inner_mut(&mut self) -> &mut AliveUI {
        &mut self.inner
    }

    /// Synchronize behaviors with current organism state
    fn sync_behaviors(&self) {
        let stats = self.inner.statistics();
        self.behaviors.cell_count.set(stats.living_cells);
        self.behaviors.total_energy.set(stats.total_energy);
        self.behaviors.generation.set(stats.generation as usize);
        self.behaviors.average_age.set(stats.average_age);
        self.behaviors.time.set(stats.time);
    }

    /// Snapshot all living cells as (x, y, id, type)
    fn snapshot_cells(&self) -> Vec<(usize, usize, Uuid, UICellType)> {
        let mut cells = Vec::new();
        let snapshot = self.inner.export_organism();
        for ((x, y), cell) in &snapshot.cells {
            if cell.is_alive() {
                cells.push((*x, *y, cell.id(), cell.cell_type()));
            }
        }
        cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_reactive_ui_creation() {
        let ui = ReactiveAliveUI::new();
        assert_eq!(ui.behaviors().cell_count.sample(), 0);
        assert_eq!(ui.behaviors().total_energy.sample(), 0.0);
        assert_eq!(ui.behaviors().time.sample(), 0.0);
    }

    #[test]
    fn test_plant_seed_updates_behaviors() {
        let mut ui = ReactiveAliveUI::new();
        ui.plant_seed(10, 10, UICellType::ButtonCore).unwrap();

        assert_eq!(ui.behaviors().cell_count.sample(), 1);
        assert!(ui.behaviors().total_energy.sample() > 0.0);
    }

    #[test]
    fn test_plant_seed_emits_birth_event() {
        let mut ui = ReactiveAliveUI::new();

        let born = Rc::new(Cell::new(false));
        let born_clone = born.clone();
        let _sub = ui.events().cell_born.subscribe(move |_evt| {
            born_clone.set(true);
        });

        ui.plant_seed(10, 10, UICellType::ButtonCore).unwrap();
        assert!(born.get());
    }

    #[test]
    fn test_step_updates_time_behavior() {
        let mut ui = ReactiveAliveUI::new();
        ui.plant_seed(10, 10, UICellType::ButtonCore).unwrap();

        ui.step(1.0);
        assert!((ui.behaviors().time.sample() - 1.0).abs() < 1e-10);

        ui.step(0.5);
        assert!((ui.behaviors().time.sample() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_subscribe_to_cell_count() {
        let mut ui = ReactiveAliveUI::new();

        let last_count = Rc::new(Cell::new(0usize));
        let last_count_clone = last_count.clone();
        let _sub = ui.behaviors().cell_count.subscribe(move |count| {
            last_count_clone.set(*count);
        });

        ui.plant_seed(10, 10, UICellType::ButtonCore).unwrap();
        assert_eq!(last_count.get(), 1);

        ui.plant_seed(11, 10, UICellType::InputField).unwrap();
        assert_eq!(last_count.get(), 2);
    }

    #[test]
    fn test_derived_behavior() {
        let mut ui = ReactiveAliveUI::new();

        // Create a derived behavior: is the colony alive?
        let is_alive = ui.behaviors().cell_count.map(|count| count > 0);

        assert!(!is_alive.sample());

        ui.plant_seed(10, 10, UICellType::ButtonCore).unwrap();
        assert!(is_alive.sample());
    }

    #[test]
    fn test_cell_death_emits_event() {
        let mut ui = ReactiveAliveUI::new();

        let died = Rc::new(Cell::new(false));
        let died_clone = died.clone();
        let _sub = ui.events().cell_died.subscribe(move |_evt| {
            died_clone.set(true);
        });

        // Plant a cell with low energy - it should die after enough steps
        ui.plant_seed(10, 10, UICellType::DataVisualization)
            .unwrap();

        // Step many times without feeding to exhaust energy
        for _ in 0..100 {
            ui.step(1.0);
        }

        assert!(died.get(), "Cell should have died and emitted death event");
    }

    #[test]
    fn test_feed_region_updates_energy() {
        let mut ui = ReactiveAliveUI::new();
        ui.plant_seed(10, 10, UICellType::ButtonCore).unwrap();

        let energy_before = ui.behaviors().total_energy.sample();
        ui.feed_region(10, 10, 3, 100.0);
        let energy_after = ui.behaviors().total_energy.sample();

        assert!(energy_after > energy_before);
    }
}
