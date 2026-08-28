//! The organism field that contains living UI cells
//!
//! The UI organism is a 2D field of cellular automata where UI cells live, grow,
//! reproduce, and evolve. It uses Amari's cellular automata foundation to create
//! complex emergent behaviors from simple local rules.

// use amari_automata::{AutomatonField, AutomatonCell, CellularRule}; // Not yet published
use amari_core::Vector;
use cliffy_core::{Component, Element, GA3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    ui_cell::{InteractionType, UICell, UICellType},
    AliveConfig, AliveError, UIEnergy, UITime,
};

/// Helper to create a GA3 from 3D vector components
fn vector3(x: f64, y: f64, z: f64) -> GA3 {
    let v = Vector::<3, 0, 0>::from_components(x, y, z);
    GA3::from_vector(&v)
}

/// Selection pressure that can be applied to influence evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionPressure {
    /// Favor cells that respond quickly to interactions
    Responsiveness,
    /// Favor cells that use energy efficiently
    EnergyEfficiency,
    /// Favor cells that form strong connections
    Cooperation,
    /// Favor cells with appealing visual properties
    VisualAppeal,
    /// Favor cells that adapt quickly
    Adaptability,
    /// Custom pressure with specific gene targets
    Custom { target_genes: HashMap<String, f64> },
}

/// A snapshot of the organism state that can be saved/loaded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganismSnapshot {
    pub cells: HashMap<(usize, usize), UICell>,
    pub dimensions: (usize, usize),
    pub generation: u64,
    pub total_time: UITime,
    pub config: AliveConfig,
}

/// The 2D field containing living UI cells
pub struct UIOrganismField {
    /// Grid of cells - Some(cell) if occupied, None if empty
    grid: Vec<Vec<Option<UICell>>>,

    /// Dimensions of the field
    dimensions: (usize, usize),

    /// Configuration for the organism
    config: AliveConfig,

    /// Current generation number
    generation: u64,

    /// Total time the organism has been alive
    total_time: UITime,

    /// Energy sources feeding the organism
    energy_sources: Vec<EnergySource>,

    /// Active selection pressures
    selection_pressures: Vec<SelectionPressure>,

    /// History of organism statistics
    history: Vec<OrganismStatistics>,
}

/// An energy source that feeds the organism
#[derive(Debug, Clone)]
pub struct EnergySource {
    pub position: (usize, usize),
    pub radius: usize,
    pub energy_per_second: UIEnergy,
    pub is_active: bool,
}

/// Statistics about the organism at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganismStatistics {
    pub time: UITime,
    pub total_cells: usize,
    pub living_cells: usize,
    pub cell_type_counts: HashMap<UICellType, usize>,
    pub total_energy: UIEnergy,
    pub average_age: UITime,
    pub genetic_diversity: f64,
}

impl UIOrganismField {
    /// Create a new organism field with the specified dimensions
    pub fn new(dimensions: (usize, usize), config: AliveConfig) -> Self {
        let (width, height) = dimensions;
        let grid = vec![vec![None; width]; height];

        Self {
            grid,
            dimensions,
            config,
            generation: 0,
            total_time: 0.0,
            energy_sources: Vec::new(),
            selection_pressures: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Get the total time the organism has been alive
    pub fn total_time(&self) -> UITime {
        self.total_time
    }

    /// Plant a seed cell at the specified coordinates
    pub fn plant_seed(
        &mut self,
        x: usize,
        y: usize,
        cell_type: UICellType,
    ) -> Result<Uuid, AliveError> {
        if x >= self.dimensions.0 || y >= self.dimensions.1 {
            return Err(AliveError::OutOfBounds { x, y });
        }

        if self.grid[y][x].is_some() {
            return Err(AliveError::CellAlreadyExists { x, y });
        }

        let position = vector3(x as f64, y as f64, 0.0);
        let cell = UICell::new_at_position(cell_type, position);
        let id = cell.id();

        self.grid[y][x] = Some(cell);
        Ok(id)
    }

    /// Remove a cell at the specified coordinates
    pub fn remove_cell(&mut self, x: usize, y: usize) -> Option<UICell> {
        if x < self.dimensions.0 && y < self.dimensions.1 {
            self.grid[y][x].take()
        } else {
            None
        }
    }

    /// Get a reference to the cell at the specified coordinates
    pub fn get_cell(&self, x: usize, y: usize) -> Option<&UICell> {
        if x < self.dimensions.0 && y < self.dimensions.1 {
            self.grid[y][x].as_ref()
        } else {
            None
        }
    }

    /// Get a mutable reference to the cell at the specified coordinates
    pub fn get_cell_mut(&mut self, x: usize, y: usize) -> Option<&mut UICell> {
        if x < self.dimensions.0 && y < self.dimensions.1 {
            self.grid[y][x].as_mut()
        } else {
            None
        }
    }

    /// Add an energy source to the organism
    pub fn add_energy_source(&mut self, source: EnergySource) {
        self.energy_sources.push(source);
    }

    /// Feed energy to a specific region
    pub fn feed_region(&mut self, x: usize, y: usize, radius: usize, energy: UIEnergy) {
        let source = EnergySource {
            position: (x, y),
            radius,
            energy_per_second: energy,
            is_active: true,
        };

        // Immediately apply some energy
        self.apply_energy_source(&source, 1.0);

        // Add as ongoing source
        self.add_energy_source(source);
    }

    /// Apply selection pressure to influence evolution
    pub fn apply_selection_pressure(&mut self, pressure: SelectionPressure) {
        self.selection_pressures.push(pressure);
    }

    /// Step the organism forward in time
    pub fn step(&mut self, dt: UITime) {
        self.total_time += dt;

        // Apply energy sources (clone to avoid borrow issues)
        let active_sources: Vec<_> = self
            .energy_sources
            .iter()
            .filter(|s| s.is_active)
            .cloned()
            .collect();
        for source in active_sources {
            self.apply_energy_source(&source, dt);
        }

        // Step all cells
        let mut cells_to_remove = Vec::new();
        let mut cells_to_reproduce = Vec::new();

        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                if let Some(cell) = &mut self.grid[y][x] {
                    cell.step(dt);

                    // Check if cell should be removed
                    if !cell.is_alive() {
                        cells_to_remove.push((x, y));
                    }

                    // Check for reproduction (will process in second pass)
                    if cell.can_reproduce(self.config.reproduction_threshold) {
                        cells_to_reproduce.push((x, y));
                    }
                }
            }
        }

        // Second pass: handle reproduction
        let mut cells_to_add = Vec::new();
        for (x, y) in cells_to_reproduce {
            if let Some(offspring_pos) = self.find_empty_neighbor(x, y) {
                if let Some(cell) = &mut self.grid[y][x] {
                    cell.start_reproduction();
                    let position = vector3(offspring_pos.0 as f64, offspring_pos.1 as f64, 0.0);
                    if let Some(offspring) = cell.reproduce(position) {
                        cells_to_add.push((offspring_pos.0, offspring_pos.1, offspring));
                    }
                }
            }
        }

        // Remove dead cells
        for (x, y) in cells_to_remove {
            self.grid[y][x] = None;
        }

        // Add new cells
        for (x, y, cell) in cells_to_add {
            if self.grid[y][x].is_none() {
                self.grid[y][x] = Some(cell);
            }
        }

        // Apply selection pressures
        self.apply_selection_pressures(dt);

        // Update connections between cells
        self.update_connections();

        // Record statistics
        if self.total_time.fract() < dt {
            // Record stats roughly once per time unit
            let stats = self.calculate_statistics();
            self.history.push(stats);
        }
    }

    /// Apply energy from a source to nearby cells
    fn apply_energy_source(&mut self, source: &EnergySource, dt: UITime) {
        let (sx, sy) = source.position;
        let energy_to_distribute = source.energy_per_second * dt;

        // Find all cells within radius
        let mut cells_in_range = Vec::new();

        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                let distance =
                    ((x as f64 - sx as f64).powi(2) + (y as f64 - sy as f64).powi(2)).sqrt();
                if distance <= source.radius as f64 && self.grid[y][x].is_some() {
                    cells_in_range.push((x, y, distance));
                }
            }
        }

        // Distribute energy based on distance (closer cells get more)
        let num_cells = cells_in_range.len();
        for (x, y, distance) in cells_in_range {
            if let Some(cell) = &mut self.grid[y][x] {
                let distance_factor = 1.0 - (distance / source.radius as f64);
                let energy_for_cell = energy_to_distribute * distance_factor / num_cells as f64;
                cell.add_energy(energy_for_cell);
            }
        }
    }

    /// Find an empty neighboring position for reproduction
    fn find_empty_neighbor(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        let neighbors = [
            (x.saturating_sub(1), y),
            (x + 1, y),
            (x, y.saturating_sub(1)),
            (x, y + 1),
            (x.saturating_sub(1), y.saturating_sub(1)),
            (x + 1, y.saturating_sub(1)),
            (x.saturating_sub(1), y + 1),
            (x + 1, y + 1),
        ];

        for (nx, ny) in neighbors {
            if nx < self.dimensions.0 && ny < self.dimensions.1 && self.grid[ny][nx].is_none() {
                return Some((nx, ny));
            }
        }

        None
    }

    /// Apply selection pressures to influence evolution
    fn apply_selection_pressures(&mut self, dt: UITime) {
        // Clone selection pressures to avoid borrow checker issues
        let pressures = self.selection_pressures.clone();

        for pressure in &pressures {
            match pressure {
                SelectionPressure::Responsiveness => {
                    self.reward_gene("responsiveness", dt);
                }
                SelectionPressure::EnergyEfficiency => {
                    self.reward_gene("energy_efficiency", dt);
                }
                SelectionPressure::Cooperation => {
                    self.reward_gene("cooperation", dt);
                }
                SelectionPressure::VisualAppeal => {
                    self.reward_gene("visual_appeal", dt);
                }
                SelectionPressure::Adaptability => {
                    self.reward_gene("adaptability", dt);
                }
                SelectionPressure::Custom { target_genes } => {
                    for (gene, target_value) in target_genes {
                        self.reward_gene_toward_target(gene, *target_value, dt);
                    }
                }
            }
        }
    }

    /// Reward cells with high values for a specific gene
    fn reward_gene(&mut self, gene_name: &str, dt: UITime) {
        for row in &mut self.grid {
            for cell in row.iter_mut().flatten() {
                let gene_value = cell.genome().get_gene(gene_name);
                let reward = gene_value * self.config.learning_rate * dt * 10.0;
                cell.add_energy(reward);
            }
        }
    }

    /// Reward cells with gene values close to target
    fn reward_gene_toward_target(&mut self, gene_name: &str, target: f64, dt: UITime) {
        for row in &mut self.grid {
            for cell in row.iter_mut().flatten() {
                let gene_value = cell.genome().get_gene(gene_name);
                let distance = (gene_value - target).abs();
                let reward = (1.0 - distance) * self.config.learning_rate * dt * 10.0;
                cell.add_energy(reward);
            }
        }
    }

    /// Update connections between neighboring cells
    fn update_connections(&mut self) {
        let mut connections_to_update = Vec::new();

        // Find all cell pairs that should be connected
        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                if let Some(cell) = &self.grid[y][x] {
                    let cell_id = cell.id();
                    let cell_type = cell.cell_type();
                    let cooperation = cell.genome().get_gene("cooperation");

                    // Check neighbors
                    let neighbors = [
                        (x.saturating_sub(1), y),
                        (x + 1, y),
                        (x, y.saturating_sub(1)),
                        (x, y + 1),
                    ];

                    for (nx, ny) in neighbors {
                        if nx < self.dimensions.0 && ny < self.dimensions.1 {
                            if let Some(neighbor) = &self.grid[ny][nx] {
                                let neighbor_id = neighbor.id();
                                let neighbor_type = neighbor.cell_type();
                                let neighbor_cooperation =
                                    neighbor.genome().get_gene("cooperation");

                                // Calculate connection strength based on compatibility
                                let type_compatibility =
                                    if cell_type.preferred_neighbors().contains(&neighbor_type) {
                                        1.0
                                    } else {
                                        0.5
                                    };

                                let cooperation_factor = (cooperation + neighbor_cooperation) / 2.0;
                                let connection_strength = type_compatibility * cooperation_factor;

                                if connection_strength > 0.1 {
                                    connections_to_update.push((
                                        x,
                                        y,
                                        cell_id,
                                        neighbor_id,
                                        connection_strength,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Apply connections
        for (x, y, _cell_id, neighbor_id, strength) in connections_to_update {
            if let Some(cell) = &mut self.grid[y][x] {
                cell.connect_to(neighbor_id, strength);
            }
        }
    }

    /// Calculate current organism statistics
    fn calculate_statistics(&self) -> OrganismStatistics {
        let mut total_cells = 0;
        let mut living_cells = 0;
        let mut cell_type_counts = HashMap::new();
        let mut total_energy = 0.0;
        let mut total_age = 0.0;
        let mut gene_values: HashMap<String, Vec<f64>> = HashMap::new();

        for row in &self.grid {
            for cell in row.iter().flatten() {
                total_cells += 1;

                if cell.is_alive() {
                    living_cells += 1;
                    total_energy += cell.energy_level();
                    total_age += cell.age();

                    *cell_type_counts.entry(cell.cell_type()).or_insert(0) += 1;

                    // Collect gene values for diversity calculation
                    for gene_name in [
                        "growth_rate",
                        "energy_efficiency",
                        "cooperation",
                        "adaptability",
                        "visual_appeal",
                        "responsiveness",
                    ] {
                        gene_values
                            .entry(gene_name.to_string())
                            .or_default()
                            .push(cell.genome().get_gene(gene_name));
                    }
                }
            }
        }

        let average_age = if living_cells > 0 {
            total_age / living_cells as f64
        } else {
            0.0
        };

        // Calculate genetic diversity as average variance across genes
        let genetic_diversity = if living_cells > 1 {
            gene_values
                .values()
                .map(|values| calculate_variance(values))
                .sum::<f64>()
                / gene_values.len() as f64
        } else {
            0.0
        };

        OrganismStatistics {
            time: self.total_time,
            total_cells,
            living_cells,
            cell_type_counts,
            total_energy,
            average_age,
            genetic_diversity,
        }
    }

    /// Handle user interaction at specific coordinates
    pub fn handle_interaction(
        &mut self,
        x: usize,
        y: usize,
        interaction_type: InteractionType,
        intensity: f64,
    ) {
        if let Some(cell) = self.get_cell_mut(x, y) {
            cell.on_interaction(interaction_type, intensity);
        }
    }

    /// Get the total number of cells
    pub fn cell_count(&self) -> usize {
        self.grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.is_some())
            .count()
    }

    /// Get the number of living cells
    pub fn living_cell_count(&self) -> usize {
        self.grid
            .iter()
            .flat_map(|row| row.iter())
            .filter_map(|cell| cell.as_ref())
            .filter(|cell| cell.is_alive())
            .count()
    }

    /// Get the total energy in the organism
    pub fn total_energy(&self) -> UIEnergy {
        self.grid
            .iter()
            .flat_map(|row| row.iter())
            .filter_map(|cell| cell.as_ref())
            .map(|cell| cell.energy_level())
            .sum()
    }

    /// Get the average age of living cells
    pub fn average_age(&self) -> UITime {
        let living_cells: Vec<_> = self
            .grid
            .iter()
            .flat_map(|row| row.iter())
            .filter_map(|cell| cell.as_ref())
            .filter(|cell| cell.is_alive())
            .collect();

        if living_cells.is_empty() {
            0.0
        } else {
            living_cells.iter().map(|cell| cell.age()).sum::<UITime>() / living_cells.len() as f64
        }
    }

    /// Get the current generation
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Get the dimensions of the field
    pub fn dimensions(&self) -> (usize, usize) {
        self.dimensions
    }

    /// Get a snapshot of the current organism state
    pub fn snapshot(&self) -> OrganismSnapshot {
        let mut cells = HashMap::new();

        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                if let Some(cell) = &self.grid[y][x] {
                    cells.insert((x, y), cell.clone());
                }
            }
        }

        OrganismSnapshot {
            cells,
            dimensions: self.dimensions,
            generation: self.generation,
            total_time: self.total_time,
            config: self.config.clone(),
        }
    }

    /// Load a snapshot into the organism
    pub fn load_snapshot(&mut self, snapshot: OrganismSnapshot) -> Result<(), AliveError> {
        if snapshot.dimensions != self.dimensions {
            return Err(AliveError::InvalidSnapshot(format!(
                "Dimension mismatch: expected {:?}, got {:?}",
                self.dimensions, snapshot.dimensions
            )));
        }

        // Clear current grid
        for row in &mut self.grid {
            for cell in row {
                *cell = None;
            }
        }

        // Load cells from snapshot
        for ((x, y), cell) in snapshot.cells {
            if x < self.dimensions.0 && y < self.dimensions.1 {
                self.grid[y][x] = Some(cell);
            }
        }

        self.generation = snapshot.generation;
        self.total_time = snapshot.total_time;
        self.config = snapshot.config;

        Ok(())
    }

    /// Get the organism history
    pub fn history(&self) -> &[OrganismStatistics] {
        &self.history
    }

    /// Iterate over all cells in the field
    pub fn iter_cells(&self) -> impl Iterator<Item = ((usize, usize), &UICell)> {
        self.grid.iter().enumerate().flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .filter_map(move |(x, cell)| cell.as_ref().map(|c| ((x, y), c)))
        })
    }
}

impl Component for UIOrganismField {
    fn render(&self, _state: &GA3) -> Element {
        let mut children = Vec::new();

        for y in 0..self.dimensions.1 {
            for x in 0..self.dimensions.0 {
                if let Some(cell) = &self.grid[y][x] {
                    let cell_state = cell.initial_state();
                    children.push(cell.render(&cell_state));
                }
            }
        }

        Element::tag("div")
            .attr("class", "alive-organism")
            .attr(
                "style",
                format!(
                    "position:relative;width:{}px;height:{}px",
                    self.dimensions.0 * 20,
                    self.dimensions.1 * 20
                ),
            )
            .children(children)
    }

    fn initial_state(&self) -> GA3 {
        // Organism state: cell count as scalar, generation in e1, total energy in e2
        let mut coeffs = vec![0.0; 8];
        coeffs[0] = self.cell_count() as f64;
        coeffs[1] = self.generation as f64;
        coeffs[2] = self.total_energy();
        GA3::from_coefficients(coeffs)
    }

    fn type_name(&self) -> &'static str {
        "UIOrganismField"
    }
}

/// Calculate variance of a set of values
fn calculate_variance(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    variance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organism_creation() {
        let config = AliveConfig::default();
        let organism = UIOrganismField::new((10, 10), config);

        assert_eq!(organism.dimensions(), (10, 10));
        assert_eq!(organism.cell_count(), 0);
        assert_eq!(organism.living_cell_count(), 0);
    }

    #[test]
    fn test_seed_planting() {
        let config = AliveConfig::default();
        let mut organism = UIOrganismField::new((10, 10), config);

        let result = organism.plant_seed(5, 5, UICellType::ButtonCore);
        assert!(result.is_ok());

        assert_eq!(organism.cell_count(), 1);
        assert_eq!(organism.living_cell_count(), 1);

        let cell = organism.get_cell(5, 5);
        assert!(cell.is_some());
        assert_eq!(cell.unwrap().cell_type(), UICellType::ButtonCore);
    }

    #[test]
    fn test_energy_feeding() {
        let config = AliveConfig::default();
        let mut organism = UIOrganismField::new((10, 10), config);

        let _ = organism.plant_seed(5, 5, UICellType::ButtonCore);
        let initial_energy = organism.get_cell(5, 5).unwrap().energy_level();

        organism.feed_region(5, 5, 2, 50.0);

        let new_energy = organism.get_cell(5, 5).unwrap().energy_level();
        assert!(new_energy > initial_energy);
    }

    #[test]
    fn test_organism_step() {
        let config = AliveConfig::default();
        let mut organism = UIOrganismField::new((10, 10), config);

        let _ = organism.plant_seed(5, 5, UICellType::ButtonCore);

        organism.step(1.0);

        // Cell should still be alive but may have consumed energy
        assert_eq!(organism.living_cell_count(), 1);
    }

    #[test]
    fn test_selection_pressure() {
        let config = AliveConfig::default();
        let mut organism = UIOrganismField::new((10, 10), config);

        let _ = organism.plant_seed(5, 5, UICellType::ButtonCore);

        organism.apply_selection_pressure(SelectionPressure::EnergyEfficiency);
        organism.step(1.0);

        // Should have applied selection pressure
        assert_eq!(organism.living_cell_count(), 1);
    }

    #[test]
    fn test_organism_component_render() {
        use cliffy_core::{Component, ElementKind};

        let config = AliveConfig::default();
        let mut organism = UIOrganismField::new((10, 10), config);

        let _ = organism.plant_seed(5, 5, UICellType::ButtonCore);
        let _ = organism.plant_seed(3, 3, UICellType::InputField);

        let state = organism.initial_state();
        let element = organism.render(&state);

        // Should be a container div
        assert!(matches!(&element.kind, ElementKind::Tag(t) if t == "div"));
        assert!(element.props.has("class"));

        // Should have 2 children (one per cell)
        assert_eq!(element.children.len(), 2);

        // Initial state should encode cell count
        assert!((state.get(0) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_snapshot() {
        let config = AliveConfig::default();
        let mut organism = UIOrganismField::new((5, 5), config.clone());

        let _ = organism.plant_seed(2, 2, UICellType::ButtonCore);
        organism.step(1.0);

        let snapshot = organism.snapshot();
        assert_eq!(snapshot.cells.len(), 1);
        assert_eq!(snapshot.dimensions, (5, 5));

        let mut new_organism = UIOrganismField::new((5, 5), config);
        let result = new_organism.load_snapshot(snapshot);
        assert!(result.is_ok());

        assert_eq!(new_organism.cell_count(), 1);
        assert_eq!(new_organism.living_cell_count(), 1);
    }
}
