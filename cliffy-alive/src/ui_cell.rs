//! Living UI cells with geometric state
//!
//! UI cells are the fundamental units of living interfaces. Each cell maintains
//! a geometric state using Amari's multivectors and participates in the cellular
//! automaton that drives UI evolution and behavior.

use cliffy_core::{Component, Element, GeometricState, GA3};
// use amari_automata::{AutomatonCell, CellularRule};
use rand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{LivingComponent, UIEnergy, UITime};

/// Different types of UI cells that can exist in the organism
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UICellType {
    /// Core of interactive elements like buttons
    ButtonCore,
    /// Edge/border cells of interactive elements  
    ButtonEdge,
    /// Input field cells for text entry
    InputField,
    /// Text display cells
    TextDisplay,
    /// Container cells that group other cells
    Container,
    /// Spacer cells that create layout spacing
    Spacer,
    /// Connection cells that link components
    Connector,
    /// Decorative cells for visual appeal
    Decoration,
    /// Sensor cells that detect user interaction
    Sensor,
    /// Memory cells that store state
    Memory,
    /// Generic cell type for testing
    Generic,
    /// Header cells for page headers
    Header,
    /// Content cells for main content
    Content,
    /// Navigation cells for menu items
    Navigation,
    /// Data display cells for showing information
    DataDisplay,
    /// Data visualization cells for charts/graphs
    DataVisualization,
}

impl UICellType {
    /// Get the base energy cost for this cell type
    pub fn base_energy_cost(&self) -> UIEnergy {
        match self {
            UICellType::ButtonCore => 2.0,
            UICellType::ButtonEdge => 1.0,
            UICellType::InputField => 3.0,
            UICellType::TextDisplay => 1.5,
            UICellType::Container => 0.8,
            UICellType::Spacer => 0.2,
            UICellType::Connector => 0.5,
            UICellType::Decoration => 0.3,
            UICellType::Sensor => 1.2,
            UICellType::Memory => 1.8,
            UICellType::Generic => 1.0,
            UICellType::Header => 2.5,
            UICellType::Content => 1.8,
            UICellType::Navigation => 2.0,
            UICellType::DataDisplay => 2.2,
            UICellType::DataVisualization => 3.5,
        }
    }

    /// Get the preferred neighbors for this cell type
    pub fn preferred_neighbors(&self) -> Vec<UICellType> {
        match self {
            UICellType::ButtonCore => vec![UICellType::ButtonEdge, UICellType::Sensor],
            UICellType::ButtonEdge => vec![UICellType::ButtonCore, UICellType::Decoration],
            UICellType::InputField => vec![UICellType::Sensor, UICellType::Memory],
            UICellType::TextDisplay => vec![UICellType::Container, UICellType::Decoration],
            UICellType::Container => vec![UICellType::Spacer, UICellType::Connector],
            UICellType::Spacer => vec![UICellType::Container],
            UICellType::Connector => vec![UICellType::Container, UICellType::Memory],
            UICellType::Decoration => vec![UICellType::ButtonEdge, UICellType::TextDisplay],
            UICellType::Sensor => vec![UICellType::ButtonCore, UICellType::InputField],
            UICellType::Memory => vec![UICellType::InputField, UICellType::Connector],
            UICellType::Generic => vec![],
            UICellType::Header => vec![UICellType::Navigation, UICellType::Content],
            UICellType::Content => vec![UICellType::Header, UICellType::DataDisplay],
            UICellType::Navigation => vec![UICellType::Header],
            UICellType::DataDisplay => vec![UICellType::Content, UICellType::DataVisualization],
            UICellType::DataVisualization => vec![UICellType::DataDisplay],
        }
    }

    /// Get the growth probability for this cell type
    pub fn growth_probability(&self) -> f64 {
        match self {
            UICellType::ButtonCore => 0.1,
            UICellType::ButtonEdge => 0.3,
            UICellType::InputField => 0.2,
            UICellType::TextDisplay => 0.4,
            UICellType::Container => 0.6,
            UICellType::Spacer => 0.8,
            UICellType::Connector => 0.7,
            UICellType::Decoration => 0.9,
            UICellType::Sensor => 0.5,
            UICellType::Memory => 0.3,
            UICellType::Generic => 0.5,
            UICellType::Header => 0.2,
            UICellType::Content => 0.4,
            UICellType::Navigation => 0.3,
            UICellType::DataDisplay => 0.6,
            UICellType::DataVisualization => 0.4,
        }
    }
}

/// State that a UI cell can be in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellState {
    /// Cell is alive and functioning
    Alive,
    /// Cell is dormant but can be reactivated
    Dormant,
    /// Cell is dying and will be removed
    Dying,
    /// Cell is dead and should be removed
    Dead,
    /// Cell is reproducing
    Reproducing,
    /// Cell is mutating to a new type
    Mutating,
    /// Cell is focused
    Focused,
}

/// A living UI cell that participates in the cellular automaton
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UICell {
    /// Unique identifier for this cell
    id: Uuid,

    /// Type of UI element this cell represents
    cell_type: UICellType,

    /// Current state of the cell
    state: CellState,

    /// Geometric state using Amari multivectors (8D: x,y,w,h,z,opacity,rotation,scale)
    #[serde(skip, default = "default_geometric_state")]
    geometric_state: GeometricState,

    /// Current energy level
    energy: UIEnergy,

    /// Age of the cell in time units
    age: UITime,

    /// Genetic information for evolution
    pub dna: CellGenome,

    /// Fitness value for evolution
    fitness: f64,

    /// Cell interaction state tracking
    interaction_state: CellInteractionState,

    /// Interaction history for fitness calculation
    interaction_history: HashMap<String, f64>,

    /// Connections to neighboring cells
    connections: HashMap<Uuid, ConnectionStrength>,

    /// Visual properties
    visual_properties: VisualProperties,

    /// Behavioral properties
    behavioral_properties: BehaviorProperties,
}

/// Connection strength between cells
pub type ConnectionStrength = f64;

/// 2D position structure
#[derive(Debug, Clone, Copy)]
pub struct Position2D {
    pub x: f64,
    pub y: f64,
}

impl Position2D {
    pub fn distance_to(&self, other: &Position2D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn distance_to_center(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// 2D size structure
#[derive(Debug, Clone, Copy)]
pub struct Size2D {
    pub width: f64,
    pub height: f64,
}

impl Size2D {
    pub fn area(&self) -> f64 {
        self.width * self.height
    }
}

/// 2D force vector
#[derive(Debug, Clone, Copy)]
pub struct Force2D {
    pub x: f64,
    pub y: f64,
}

impl Force2D {
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

/// Cell interaction state for stress and interaction tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellInteractionState {
    pub stress_level: f64,
    pub interaction_count: u32,
}

impl Default for CellInteractionState {
    fn default() -> Self {
        Self::new()
    }
}

impl CellInteractionState {
    pub fn new() -> Self {
        Self {
            stress_level: 0.0,
            interaction_count: 0,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.stress_level < 0.7
    }
}

/// Genetic information for UI cells
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellGenome {
    /// Genes that determine cell behavior
    pub genes: HashMap<String, f64>,
    /// Traits that affect behavior
    pub traits: HashMap<String, f64>,
    /// Affinity genes that determine relationships with other cell types
    pub affinities: HashMap<UICellType, f64>,
}

impl Default for CellGenome {
    fn default() -> Self {
        Self::new()
    }
}

impl CellGenome {
    pub fn new() -> Self {
        let mut genes = HashMap::new();
        let mut traits = HashMap::new();

        // Initialize default genes
        genes.insert("growth_rate".to_string(), 0.5);
        genes.insert("energy_efficiency".to_string(), 0.7);
        genes.insert("cooperation".to_string(), 0.6);
        genes.insert("adaptability".to_string(), 0.4);
        genes.insert("visual_appeal".to_string(), 0.5);
        genes.insert("responsiveness".to_string(), 0.8);

        // Initialize default traits
        traits.insert("display_brightness".to_string(), 0.5);
        traits.insert("update_frequency".to_string(), 0.5);
        traits.insert("user_engagement".to_string(), 0.3);

        Self {
            genes,
            traits,
            affinities: HashMap::new(),
        }
    }

    pub fn get_gene(&self, name: &str) -> f64 {
        self.genes.get(name).copied().unwrap_or(0.5)
    }

    pub fn set_gene(&mut self, name: &str, value: f64) {
        self.genes.insert(name.to_string(), value.clamp(0.0, 1.0));
    }

    pub fn mutate(&mut self, rate: f64) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for value in self.genes.values_mut() {
            if rng.gen::<f64>() < rate {
                let delta = rng.gen_range(-0.1..=0.1);
                *value = (*value + delta).clamp(0.0, 1.0);
            }
        }
    }

    pub fn crossover(&self, other: &CellGenome, _crossover_rate: f64) -> CellGenome {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut new_genes = HashMap::new();
        let mut new_traits = HashMap::new();
        let mut new_affinities = HashMap::new();

        // Combine genes from both parents
        for key in self.genes.keys() {
            let value = if rng.gen::<f64>() < 0.5 {
                self.genes.get(key).copied().unwrap_or(0.5)
            } else {
                other.genes.get(key).copied().unwrap_or(0.5)
            };
            new_genes.insert(key.clone(), value);
        }

        // Combine traits
        for key in self.traits.keys().chain(other.traits.keys()) {
            let value = if rng.gen::<f64>() < 0.5 {
                self.traits.get(key).copied().unwrap_or(0.5)
            } else {
                other.traits.get(key).copied().unwrap_or(0.5)
            };
            new_traits.insert(key.clone(), value);
        }

        // Combine affinities
        for key in self.affinities.keys().chain(other.affinities.keys()) {
            let value = if rng.gen::<f64>() < 0.5 {
                self.affinities.get(key).copied().unwrap_or(0.0)
            } else {
                other.affinities.get(key).copied().unwrap_or(0.0)
            };
            new_affinities.insert(*key, value);
        }

        CellGenome {
            genes: new_genes,
            traits: new_traits,
            affinities: new_affinities,
        }
    }

    /// Add affinity gene for a cell type
    pub fn add_affinity_gene(&mut self, cell_type: UICellType, affinity: f64) {
        self.affinities.insert(cell_type, affinity);
    }

    /// Calculate affinity to another genome
    pub fn calculate_affinity(&self, _other: &CellGenome, other_type: UICellType) -> f64 {
        self.affinities.get(&other_type).copied().unwrap_or(0.0)
    }

    /// Check if has genes from another genome
    pub fn has_genes_from(&self, other: &CellGenome) -> bool {
        for key in other.genes.keys() {
            if self.genes.contains_key(key) {
                return true;
            }
        }
        false
    }

    /// Calculate similarity to another genome
    pub fn similarity_to(&self, other: &CellGenome) -> f64 {
        let mut total_diff = 0.0;
        let mut count = 0;

        for (key, value) in &self.genes {
            if let Some(other_value) = other.genes.get(key) {
                total_diff += (value - other_value).abs();
                count += 1;
            }
        }

        if count == 0 {
            return 0.0;
        }

        1.0 - (total_diff / count as f64)
    }
}

/// Visual properties of a UI cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualProperties {
    pub color: [f64; 4],        // RGBA
    pub size: f64,              // Relative size
    pub opacity: f64,           // Transparency
    pub border_width: f64,      // Border thickness
    pub border_color: [f64; 4], // Border RGBA
    pub glow_intensity: f64,    // Glow effect strength
}

impl Default for VisualProperties {
    fn default() -> Self {
        Self {
            color: [0.2, 0.5, 0.8, 1.0], // Blue
            size: 1.0,
            opacity: 1.0,
            border_width: 1.0,
            border_color: [0.0, 0.0, 0.0, 1.0], // Black border
            glow_intensity: 0.0,
        }
    }
}

/// Behavioral properties of a UI cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorProperties {
    pub interaction_radius: f64, // How far the cell can sense interactions
    pub response_speed: f64,     // How quickly it responds to stimuli
    pub memory_capacity: usize,  // How many past events it remembers
    pub learning_rate: f64,      // How quickly it adapts
    pub social_tendency: f64,    // Tendency to connect with neighbors
}

impl Default for BehaviorProperties {
    fn default() -> Self {
        Self {
            interaction_radius: 5.0,
            response_speed: 1.0,
            memory_capacity: 10,
            learning_rate: 0.1,
            social_tendency: 0.5,
        }
    }
}

/// Default geometric state for deserialization
fn default_geometric_state() -> GeometricState {
    GeometricState::from_scalar(0.0)
}

impl UICell {
    /// Create a new UI cell of the specified type  
    pub fn new(cell_type: UICellType) -> Self {
        Self::new_at_position(cell_type, GA3::scalar(0.0))
    }

    /// Create a new UI cell at a specific position
    pub fn new_at_position(cell_type: UICellType, position: GA3) -> Self {
        let id = Uuid::new_v4();
        let geometric_state = GeometricState::new(position);
        let energy = cell_type.base_energy_cost() * 10.0; // Start with 10x base cost

        let mut visual_props = VisualProperties::default();

        // Customize appearance based on cell type
        match cell_type {
            UICellType::ButtonCore => {
                visual_props.color = [0.2, 0.7, 0.3, 1.0]; // Green
                visual_props.size = 1.5;
                visual_props.glow_intensity = 0.3;
            }
            UICellType::ButtonEdge => {
                visual_props.color = [0.1, 0.6, 0.2, 1.0]; // Darker green
                visual_props.size = 1.0;
            }
            UICellType::InputField => {
                visual_props.color = [0.9, 0.9, 0.9, 1.0]; // Light gray
                visual_props.border_width = 2.0;
                visual_props.border_color = [0.5, 0.5, 0.5, 1.0];
            }
            UICellType::TextDisplay => {
                visual_props.color = [0.1, 0.1, 0.1, 1.0]; // Dark text
                visual_props.size = 0.8;
            }
            UICellType::Decoration => {
                visual_props.color = [0.8, 0.6, 0.9, 0.7]; // Light purple, transparent
                visual_props.glow_intensity = 0.8;
            }
            _ => {} // Use defaults
        }

        Self {
            id,
            cell_type,
            state: CellState::Alive,
            geometric_state,
            energy,
            age: 0.0,
            dna: CellGenome::new(),
            fitness: 0.0,
            interaction_state: CellInteractionState::new(),
            interaction_history: HashMap::new(),
            connections: HashMap::new(),
            visual_properties: visual_props,
            behavioral_properties: BehaviorProperties::default(),
        }
    }

    /// Get the cell type
    pub fn cell_type(&self) -> UICellType {
        self.cell_type
    }

    /// Get the current state
    pub fn state(&self) -> CellState {
        self.state
    }

    /// Get visual properties
    pub fn visual_properties(&self) -> &VisualProperties {
        &self.visual_properties
    }

    /// Get behavioral properties
    pub fn behavioral_properties(&self) -> &BehaviorProperties {
        &self.behavioral_properties
    }

    /// Get the genome
    pub fn genome(&self) -> &CellGenome {
        &self.dna
    }

    /// Get mutable genome for mutations
    pub fn genome_mut(&mut self) -> &mut CellGenome {
        &mut self.dna
    }

    /// Connect to another cell
    pub fn connect_to(&mut self, other_id: Uuid, strength: ConnectionStrength) {
        self.connections.insert(other_id, strength);
    }

    /// Disconnect from another cell
    pub fn disconnect_from(&mut self, other_id: &Uuid) {
        self.connections.remove(other_id);
    }

    /// Get connection strength to another cell
    pub fn connection_strength(&self, other_id: &Uuid) -> ConnectionStrength {
        self.connections.get(other_id).copied().unwrap_or(0.0)
    }

    /// Get all connections
    pub fn connections(&self) -> &HashMap<Uuid, ConnectionStrength> {
        &self.connections
    }

    /// Add energy to the cell
    pub fn add_energy(&mut self, amount: UIEnergy) {
        self.energy += amount;
    }

    /// Consume energy from the cell
    pub fn consume_energy(&mut self, amount: UIEnergy) -> bool {
        if self.energy >= amount {
            self.energy -= amount;
            true
        } else {
            false
        }
    }

    /// Check if the cell can reproduce
    pub fn can_reproduce(&self, threshold: UIEnergy) -> bool {
        self.energy >= threshold && self.state == CellState::Alive
    }

    /// Start reproduction process
    pub fn start_reproduction(&mut self) {
        if self.state == CellState::Alive {
            self.state = CellState::Reproducing;
        }
    }

    /// Create offspring cell
    pub fn reproduce(&mut self, position: GA3) -> Option<UICell> {
        if self.state != CellState::Reproducing {
            return None;
        }

        // Cost energy for reproduction
        let reproduction_cost = self.cell_type.base_energy_cost() * 5.0;
        if !self.consume_energy(reproduction_cost) {
            return None;
        }

        // Create offspring with mutated genome
        let mut offspring = UICell::new_at_position(self.cell_type, position);
        offspring.dna = self.dna.clone();
        offspring.dna.mutate(0.05); // 5% mutation rate

        // Return to normal state
        self.state = CellState::Alive;

        Some(offspring)
    }

    /// Update visual properties based on genome
    pub fn update_appearance(&mut self) {
        let appeal_gene = self.dna.get_gene("visual_appeal");
        let energy_ratio = (self.energy / 100.0).clamp(0.0, 1.0);

        // Adjust visual properties based on genes and energy
        self.visual_properties.glow_intensity = appeal_gene * energy_ratio;
        self.visual_properties.opacity = 0.3 + 0.7 * energy_ratio;
        self.visual_properties.size = 0.5 + 1.5 * energy_ratio;
    }

    /// React to user interaction
    pub fn on_interaction(&mut self, interaction_type: InteractionType, intensity: f64) {
        match interaction_type {
            InteractionType::Click => {
                self.add_energy(intensity * 10.0);
                self.visual_properties.glow_intensity =
                    (self.visual_properties.glow_intensity + 0.5).min(1.0);
            }
            InteractionType::Hover => {
                self.visual_properties.size = (self.visual_properties.size * 1.1).min(2.0);
            }
            InteractionType::Focus => {
                self.visual_properties.border_width =
                    (self.visual_properties.border_width * 1.5).min(5.0);
            }
        }
    }

    // === Methods for test support ===

    /// Get the cell's 8D geometric nucleus state
    pub fn nucleus(&self) -> &GeometricState {
        &self.geometric_state
    }

    /// Get 2D position (x, y)
    pub fn position(&self) -> Position2D {
        let state = self.geometric_state.multivector();
        Position2D {
            x: state.scalar_part(),       // [0] scalar
            y: state.vector_component(0), // [1] e1
        }
    }

    /// Get size (width, height)
    pub fn size(&self) -> Size2D {
        let state = self.geometric_state.multivector();
        Size2D {
            width: state.vector_component(1).abs().max(10.0), // [2] e2, minimum size
            height: state.vector_component(2).abs().max(10.0), // [4] e3, minimum size
        }
    }

    /// Get opacity (0.0 to 1.0)
    pub fn opacity(&self) -> f64 {
        let state = self.geometric_state.multivector();
        ((state.get(3) + 1.0) / 2.0).clamp(0.0, 1.0) // [3] e12 bivector, normalized to 0-1
    }

    /// Get z-index from geometric state
    pub fn z_index(&self) -> i32 {
        let state = self.geometric_state.multivector();
        state.get(5) as i32 // [5] e13 bivector
    }

    /// Get rotation angle in degrees
    pub fn rotation(&self) -> f64 {
        let state = self.geometric_state.multivector();
        state.get(7) * 180.0 // [7] e123 pseudoscalar, radians to degrees
    }

    /// Get scale factor
    pub fn scale(&self) -> f64 {
        let state = self.geometric_state.multivector();
        state.get(6).abs().max(0.1) // [6] e23 bivector, minimum scale
    }

    /// Project 8D geometric state to a CSS style string
    ///
    /// Maps GA3 coefficients to CSS properties:
    /// - `[0]` scalar → x position (px)
    /// - `[1]` e1 → y position (px)
    /// - `[2]` e2 → width (px)
    /// - `[4]` e3 → height (px)
    /// - `[3]` e12 → opacity (normalized 0-1)
    /// - `[5]` e13 → z-index
    /// - `[6]` e23 → scale
    /// - `[7]` e123 → rotation (degrees)
    pub fn to_css_string(&self, state: &GA3) -> String {
        let x = state.scalar_part();
        let y = state.vector_component(0);
        let width = state.vector_component(1).abs().max(10.0);
        let height = state.vector_component(2).abs().max(10.0);
        let opacity = ((state.get(3) + 1.0) / 2.0).clamp(0.0, 1.0);
        let z_index = state.get(5) as i32;
        let scale = state.get(6).abs().max(0.1);
        let rotation = state.get(7) * 180.0;

        let color = &self.visual_properties.color;
        let r = (color[0] * 255.0) as u8;
        let g = (color[1] * 255.0) as u8;
        let b = (color[2] * 255.0) as u8;

        format!(
            "position:absolute;\
             left:{x:.1}px;\
             top:{y:.1}px;\
             width:{width:.1}px;\
             height:{height:.1}px;\
             opacity:{opacity:.3};\
             z-index:{z_index};\
             transform:rotate({rotation:.1}deg) scale({scale:.3});\
             background:rgb({r},{g},{b});\
             border:{bw:.1}px solid rgb({br},{bg},{bb})",
            x = x,
            y = y,
            width = width,
            height = height,
            opacity = opacity,
            z_index = z_index,
            rotation = rotation,
            scale = scale,
            r = r,
            g = g,
            b = b,
            bw = self.visual_properties.border_width,
            br = (self.visual_properties.border_color[0] * 255.0) as u8,
            bg = (self.visual_properties.border_color[1] * 255.0) as u8,
            bb = (self.visual_properties.border_color[2] * 255.0) as u8,
        )
    }

    /// Set position
    pub fn set_position(&mut self, x: f64, y: f64) {
        let mut current = self.geometric_state.multivector();
        current.set_scalar(x);
        current.set_vector_component(0, y); // e1
        self.geometric_state.set(current);
    }

    /// Calculate affinity to another cell based on DNA
    pub fn affinity_to(&self, other: &UICell) -> f64 {
        self.dna.calculate_affinity(&other.dna, other.cell_type)
    }

    /// Calculate interaction force with another cell
    pub fn interaction_force(&self, other: &UICell) -> Force2D {
        let my_pos = self.position();
        let other_pos = other.position();

        let dx = other_pos.x - my_pos.x;
        let dy = other_pos.y - my_pos.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < 1.0 {
            return Force2D { x: 0.0, y: 0.0 };
        }

        let affinity = self.affinity_to(other);
        let force_magnitude = affinity / (distance * distance);

        Force2D {
            x: force_magnitude * dx / distance,
            y: force_magnitude * dy / distance,
        }
    }

    /// Metabolize energy over time
    pub fn metabolize(&mut self, dt: UITime) -> Result<(), crate::AliveError> {
        let base_cost = self.cell_type.base_energy_cost() * dt;
        let efficiency = self.dna.get_gene("energy_efficiency");
        let actual_cost = base_cost * (2.0 - efficiency);

        if !self.consume_energy(actual_cost) {
            self.state = CellState::Dying;
        }

        Ok(())
    }

    /// Receive click interaction
    pub fn receive_click(&mut self) {
        self.energy += 15.0;
        self.interaction_state.interaction_count += 1;
        self.interaction_history.insert("click".to_string(), 1.0);
    }

    /// Receive hover interaction
    pub fn receive_hover(&mut self) {
        self.energy += 5.0;
        self.interaction_state.interaction_count += 1;
        self.interaction_history.insert("hover".to_string(), 0.5);
    }

    /// Receive focus interaction
    pub fn receive_focus(&mut self) {
        self.energy += 8.0;
        self.state = CellState::Focused;
        self.interaction_history.insert("focus".to_string(), 0.3);
    }

    /// Check if cell should die
    pub fn should_die(&self) -> bool {
        self.energy < 1.0 || self.interaction_state.stress_level > 0.9
    }

    /// Check if cell is focused
    pub fn is_focused(&self) -> bool {
        matches!(self.state, CellState::Focused)
    }

    /// Check if cell is in juvenile stage
    pub fn is_juvenile(&self) -> bool {
        self.age < 5.0
    }

    /// Apply geometric mutation
    pub fn apply_geometric_mutation(&mut self, strength: f64) {
        let mut current = self.geometric_state.multivector();
        let random_dx = (rand::random::<f64>() - 0.5) * strength * 20.0;
        let random_dy = (rand::random::<f64>() - 0.5) * strength * 20.0;

        current.set_scalar(current.scalar_part() + random_dx);
        current.set_vector_component(0, current.vector_component(0) + random_dy); // e1

        self.geometric_state.set(current);
    }

    /// Apply stress to the cell
    pub fn apply_stress(&mut self, stress_amount: f64) {
        self.interaction_state.stress_level =
            (self.interaction_state.stress_level + stress_amount).min(1.0);
    }

    /// Get cell interaction state
    pub fn get_interaction_state(&self) -> &CellInteractionState {
        &self.interaction_state
    }

    /// Get cell genome
    pub fn get_genome(&self) -> &CellGenome {
        &self.dna
    }

    /// Record interaction for fitness calculation
    pub fn record_interaction(&mut self, interaction_type: &str, value: f64) {
        self.interaction_history
            .insert(interaction_type.to_string(), value);
    }

    /// Set fitness value
    pub fn set_fitness(&mut self, fitness: f64) {
        self.fitness = fitness;
    }

    /// Get fitness value
    pub fn fitness(&self) -> f64 {
        self.fitness
    }

    /// Check if marked for death
    pub fn marked_for_death(&self) -> bool {
        matches!(self.state, CellState::Dead)
    }

    /// Mark cell for death
    pub fn mark_for_death(&mut self) {
        self.state = CellState::Dead;
    }

    /// Set age
    pub fn set_age(&mut self, new_age: UITime) {
        self.age = new_age;
    }

    /// Get update frequency based on genetics
    pub fn get_update_frequency(&self) -> f64 {
        self.dna
            .traits
            .get("update_frequency")
            .copied()
            .unwrap_or(0.5)
    }

    /// Express genes to update visual properties
    pub fn express_genes(&mut self) {
        // Update opacity based on brightness gene
        if let Some(brightness) = self.dna.traits.get("display_brightness") {
            let mut current = self.geometric_state.multivector();
            current.set(3, brightness * 2.0 - 1.0); // e12 bivector component, convert 0-1 to -1-1
            self.geometric_state.set(current);
        }
    }

    /// Set energy level
    pub fn set_energy(&mut self, energy: UIEnergy) {
        self.energy = energy;
    }

    /// Get energy level
    pub fn energy(&self) -> UIEnergy {
        self.energy
    }

    /// Get the cell's unique ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the cell's age in time units
    pub fn age(&self) -> UITime {
        self.age
    }

    /// Check if the cell is alive
    pub fn is_alive(&self) -> bool {
        matches!(
            self.state,
            CellState::Alive | CellState::Reproducing | CellState::Focused
        )
    }

    /// Publicly accessible energy_level (delegates to trait method)
    pub fn energy_level(&self) -> UIEnergy {
        self.energy
    }

    /// Publicly accessible step method
    pub fn step(&mut self, dt: UITime) {
        self.age += dt;

        // Consume energy over time
        let energy_cost = self.cell_type.base_energy_cost() * dt;
        self.energy -= energy_cost;

        // Update vitals (health is derived from energy level and stress)
        // Note: Health can be calculated as a function of energy and stress_level when needed

        // Check if cell should die
        if self.energy <= 0.0 {
            self.state = CellState::Dead;
        }
    }
}

/// Types of user interactions
#[derive(Debug, Clone, Copy)]
pub enum InteractionType {
    Click,
    Hover,
    Focus,
}

impl LivingComponent for UICell {
    fn geometric_state(&self) -> &GeometricState {
        &self.geometric_state
    }

    fn energy_level(&self) -> UIEnergy {
        self.energy
    }

    fn step(&mut self, dt: UITime) {
        self.age += dt;

        // Base metabolism - consume energy over time
        let base_cost = self.cell_type.base_energy_cost() * dt;
        let efficiency = self.dna.get_gene("energy_efficiency");
        let actual_cost = base_cost * (2.0 - efficiency); // More efficient = less cost

        if !self.consume_energy(actual_cost) {
            // Not enough energy - start dying
            if self.state == CellState::Alive {
                self.state = CellState::Dying;
            }
        }

        // Update state based on energy
        match self.state {
            CellState::Dying => {
                if self.energy <= 0.0 {
                    self.state = CellState::Dead;
                }
            }
            CellState::Reproducing if self.age % 10.0 < dt => {
                // Reproduction takes time and energy
                self.state = CellState::Alive;
            }
            _ => {}
        }

        // Update appearance based on current state
        self.update_appearance();
    }

    fn is_alive(&self) -> bool {
        matches!(
            self.state,
            CellState::Alive | CellState::Reproducing | CellState::Mutating
        )
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn age(&self) -> UITime {
        self.age
    }
}

impl Component for UICell {
    fn render(&self, state: &GA3) -> Element {
        Element::tag("div")
            .attr("class", format!("alive-cell alive-{:?}", self.cell_type))
            .with_key(self.id.to_string())
            .attr("style", self.to_css_string(state))
    }

    fn initial_state(&self) -> GA3 {
        self.geometric_state.multivector()
    }

    fn type_name(&self) -> &'static str {
        "UICell"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cliffy_core::GA3;

    #[test]
    fn test_cell_creation() {
        let position = GA3::scalar(1.0);
        let cell = UICell::new_at_position(UICellType::ButtonCore, position);

        assert_eq!(cell.cell_type(), UICellType::ButtonCore);
        assert_eq!(cell.state(), CellState::Alive);
        assert!(cell.is_alive());
        assert!(cell.energy_level() > 0.0);
    }

    #[test]
    fn test_cell_energy_management() {
        let position = GA3::scalar(1.0);
        let mut cell = UICell::new_at_position(UICellType::ButtonCore, position);

        let initial_energy = cell.energy_level();
        cell.add_energy(10.0);
        assert_eq!(cell.energy_level(), initial_energy + 10.0);

        assert!(cell.consume_energy(5.0));
        assert_eq!(cell.energy_level(), initial_energy + 5.0);

        assert!(!cell.consume_energy(1000.0));
    }

    #[test]
    fn test_cell_reproduction() {
        let position = GA3::scalar(1.0);
        let mut cell = UICell::new_at_position(UICellType::ButtonCore, position);

        // Add enough energy for reproduction
        cell.add_energy(100.0);

        assert!(cell.can_reproduce(50.0));
        cell.start_reproduction();
        assert_eq!(cell.state(), CellState::Reproducing);

        let offspring_pos = GA3::scalar(2.0);
        let offspring = cell.reproduce(offspring_pos);
        assert!(offspring.is_some());

        let offspring = offspring.unwrap();
        assert_eq!(offspring.cell_type(), UICellType::ButtonCore);
        assert!(offspring.is_alive());
    }

    #[test]
    fn test_genome_operations() {
        let mut genome1 = CellGenome::new();
        let mut genome2 = CellGenome::new();

        genome1.set_gene("test_gene", 0.8);
        genome2.set_gene("test_gene", 0.2);

        let offspring_genome = genome1.crossover(&genome2, 0.5);
        let offspring_value = offspring_genome.get_gene("test_gene");

        // Should be one of the parent values
        assert!(offspring_value == 0.8 || offspring_value == 0.2);

        // Test mutation
        let _original_value = genome1.get_gene("growth_rate");
        genome1.mutate(1.0); // 100% mutation rate
        let mutated_value = genome1.get_gene("growth_rate");

        // Value might have changed due to mutation
        assert!((0.0..=1.0).contains(&mutated_value));
    }

    #[test]
    fn test_component_render() {
        use cliffy_core::{Component, ElementKind};

        let cell = UICell::new_at_position(UICellType::ButtonCore, GA3::scalar(5.0));
        let state = cell.initial_state();
        let element = cell.render(&state);

        // Should produce a div with class and style
        assert!(matches!(&element.kind, ElementKind::Tag(t) if t == "div"));
        assert!(element.props.has("class"));
        assert!(element.props.has("style"));
        assert!(element.key.is_some());

        // Class should include cell type
        let class = element.props.get("class").unwrap().as_str().unwrap();
        assert!(class.contains("alive-cell"));
        assert!(class.contains("ButtonCore"));
    }

    #[test]
    fn test_css_projection() {
        let cell = UICell::new_at_position(UICellType::ButtonCore, GA3::scalar(0.0));

        // Build a state with known coefficients
        let coeffs = vec![
            100.0, // [0] x
            200.0, // [1] y
            50.0,  // [2] width
            0.5,   // [3] opacity (raw, maps to (0.5+1)/2 = 0.75)
            30.0,  // [4] height
            5.0,   // [5] z-index
            1.0,   // [6] scale
            0.0,   // [7] rotation
        ];
        let state = GA3::from_coefficients(coeffs);
        let css = cell.to_css_string(&state);

        assert!(css.contains("left:100.0px"));
        assert!(css.contains("top:200.0px"));
        assert!(css.contains("width:50.0px"));
        assert!(css.contains("height:30.0px"));
        assert!(css.contains("opacity:0.750"));
        assert!(css.contains("z-index:5"));
        assert!(css.contains("scale(1.000)"));
    }

    #[test]
    fn test_cell_step() {
        let position = GA3::scalar(1.0);
        let mut cell = UICell::new_at_position(UICellType::ButtonCore, position);

        let initial_energy = cell.energy_level();
        let initial_age = cell.age();

        cell.step(1.0);

        assert_eq!(cell.age(), initial_age + 1.0);
        assert!(cell.energy_level() < initial_energy); // Should consume energy
    }
}
