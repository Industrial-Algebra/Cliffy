//! Nervous system for reacting to user input
//!
//! This module implements a neural network-like system that allows UI organisms
//! to sense, process, and respond to user interactions in sophisticated ways.

use amari_core::Vector;
use cliffy_core::GA3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    ui_cell::{UICell, UICellType},
    UIEnergy, UITime,
};

/// Helper to create a GA3 from 3D vector components
fn vector3(x: f64, y: f64, z: f64) -> GA3 {
    let v = Vector::<3, 0, 0>::from_components(x, y, z);
    GA3::from_vector(&v)
}

/// Types of sensory input the nervous system can detect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensorType {
    /// Mouse/touch position detection
    Position,
    /// Mouse/touch movement detection
    Movement,
    /// Click/tap detection
    Touch,
    /// Hover/proximity detection
    Proximity,
    /// Keyboard input detection
    Keyboard,
    /// Visual attention detection
    Attention,
    /// Energy level changes
    Energy,
    /// Neighbor activity detection
    Social,
    /// Time-based patterns
    Temporal,
    /// Environmental changes
    Environmental,
}

impl SensorType {
    /// Get the detection range for this sensor type
    pub fn detection_range(&self) -> f64 {
        match self {
            SensorType::Position => 1.0,
            SensorType::Movement => 2.0,
            SensorType::Touch => 1.0,
            SensorType::Proximity => 3.0,
            SensorType::Keyboard => 5.0,
            SensorType::Attention => 4.0,
            SensorType::Energy => 0.5,
            SensorType::Social => 2.0,
            SensorType::Temporal => f64::INFINITY,
            SensorType::Environmental => 10.0,
        }
    }

    /// Get the sensitivity of this sensor type
    pub fn sensitivity(&self) -> f64 {
        match self {
            SensorType::Position => 0.8,
            SensorType::Movement => 0.9,
            SensorType::Touch => 1.0,
            SensorType::Proximity => 0.7,
            SensorType::Keyboard => 0.6,
            SensorType::Attention => 0.5,
            SensorType::Energy => 0.9,
            SensorType::Social => 0.6,
            SensorType::Temporal => 0.3,
            SensorType::Environmental => 0.4,
        }
    }
}

/// A sensory input stimulus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stimulus {
    pub sensor_type: SensorType,
    #[serde(skip, default = "GA3::zero")]
    pub position: GA3,
    pub intensity: f64,
    pub duration: UITime,
    pub timestamp: UITime,
    pub metadata: HashMap<String, f64>,
}

impl Stimulus {
    /// Create a new stimulus
    pub fn new(sensor_type: SensorType, position: GA3, intensity: f64) -> Self {
        Self {
            sensor_type,
            position,
            intensity,
            duration: 0.0,
            timestamp: 0.0, // Would be set by the nervous system
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the stimulus
    pub fn with_metadata(mut self, key: String, value: f64) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if the stimulus affects a cell at a given position
    pub fn affects_position(&self, position: &GA3) -> bool {
        let distance = (&self.position - position).magnitude();
        distance <= self.sensor_type.detection_range()
    }

    /// Calculate the stimulus intensity at a given position
    pub fn intensity_at(&self, position: &GA3) -> f64 {
        let distance = (&self.position - position).magnitude();
        let range = self.sensor_type.detection_range();

        if distance > range {
            0.0
        } else {
            let falloff = 1.0 - (distance / range);
            self.intensity * falloff * self.sensor_type.sensitivity()
        }
    }
}

/// Types of neural responses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseType {
    /// Immediate reflexive response
    Reflex,
    /// Learned behavioral response
    Learned,
    /// Adaptive response that changes over time
    Adaptive,
    /// Social response influenced by neighbors
    Social,
    /// Memory-based response
    Memory,
    /// Creative/novel response
    Creative,
}

/// A neural response to stimuli
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub response_type: ResponseType,
    pub intensity: f64,
    pub energy_cost: UIEnergy,
    pub duration: UITime,
    pub actions: Vec<Action>,
}

/// Actions that can be taken in response to stimuli
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Change visual appearance
    VisualChange {
        color_delta: [f64; 4],
        size_delta: f64,
        opacity_delta: f64,
        glow_delta: f64,
    },
    /// Physical movement
    Movement {
        #[serde(skip, default = "GA3::zero")]
        force: GA3,
        #[serde(skip, default = "GA3::zero")]
        torque: GA3,
    },
    /// Energy manipulation
    EnergyChange { delta: UIEnergy },
    /// Communication with neighbors
    Communication { message: String, range: f64 },
    /// State change
    StateChange { new_state: String },
    /// Connection modification
    ConnectionChange {
        target_id: Option<Uuid>,
        strength_delta: f64,
    },
    /// Memory storage
    MemoryStore { key: String, value: f64 },
    /// Learning update
    LearningUpdate { gene: String, adjustment: f64 },
}

/// A single neuron in the nervous system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neuron {
    /// Unique identifier
    pub id: Uuid,

    /// Type of neuron
    pub neuron_type: NeuronType,

    /// Current activation level
    pub activation: f64,

    /// Threshold for firing
    pub threshold: f64,

    /// Current fatigue level (reduces response)
    pub fatigue: f64,

    /// Learning rate for this neuron
    pub learning_rate: f64,

    /// Input connections with weights
    pub inputs: HashMap<Uuid, f64>,

    /// Output connections with weights
    pub outputs: HashMap<Uuid, f64>,

    /// Memory traces
    pub memory: HashMap<String, f64>,
}

/// Types of neurons in the nervous system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NeuronType {
    /// Sensory input neurons
    Sensor,
    /// Processing neurons
    Processor,
    /// Motor output neurons
    Motor,
    /// Memory neurons
    Memory,
    /// Modulatory neurons (affect learning)
    Modulator,
}

impl Neuron {
    /// Create a new neuron
    pub fn new(neuron_type: NeuronType) -> Self {
        Self {
            id: Uuid::new_v4(),
            neuron_type,
            activation: 0.0,
            threshold: 0.5,
            fatigue: 0.0,
            learning_rate: 0.1,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            memory: HashMap::new(),
        }
    }

    /// Update neuron activation based on inputs
    pub fn update(&mut self, dt: UITime) {
        // Calculate total input
        let mut total_input = 0.0;
        for weight in self.inputs.values() {
            // Would need access to other neurons to get their activation
            // For now, use a simplified calculation
            total_input += weight * 0.5; // Placeholder
        }

        // Apply activation function (sigmoid)
        let net_input = total_input - self.threshold;
        self.activation = 1.0 / (1.0 + (-net_input).exp());

        // Apply fatigue
        self.activation *= 1.0 - self.fatigue;

        // Update fatigue (increases with high activation)
        if self.activation > 0.8 {
            self.fatigue += dt * 0.1;
        } else {
            self.fatigue -= dt * 0.05;
        }
        self.fatigue = self.fatigue.clamp(0.0, 0.9);
    }

    /// Check if the neuron is firing
    pub fn is_firing(&self) -> bool {
        self.activation > self.threshold
    }

    /// Add an input connection
    pub fn add_input(&mut self, neuron_id: Uuid, weight: f64) {
        self.inputs.insert(neuron_id, weight);
    }

    /// Add an output connection
    pub fn add_output(&mut self, neuron_id: Uuid, weight: f64) {
        self.outputs.insert(neuron_id, weight);
    }

    /// Update connection weight based on learning
    pub fn update_weight(&mut self, connection_id: &Uuid, delta: f64) {
        if let Some(weight) = self.inputs.get_mut(connection_id) {
            *weight += delta * self.learning_rate;
            *weight = weight.clamp(-2.0, 2.0); // Limit weight range
        }
        if let Some(weight) = self.outputs.get_mut(connection_id) {
            *weight += delta * self.learning_rate;
            *weight = weight.clamp(-2.0, 2.0);
        }
    }

    /// Store a memory trace
    pub fn store_memory(&mut self, key: String, value: f64) {
        self.memory.insert(key, value);
    }

    /// Retrieve a memory trace
    pub fn recall_memory(&self, key: &str) -> Option<f64> {
        self.memory.get(key).copied()
    }
}

/// Neural network for a UI cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellNeuralNetwork {
    /// All neurons in the network
    neurons: HashMap<Uuid, Neuron>,

    /// Sensory input neurons by type
    sensor_neurons: HashMap<SensorType, Uuid>,

    /// Motor output neurons by action type
    motor_neurons: HashMap<String, Uuid>,

    /// Processing layers
    hidden_layers: Vec<Vec<Uuid>>,

    /// Current stimulus inputs
    current_stimuli: Vec<Stimulus>,

    /// Response patterns learned over time
    learned_responses: HashMap<String, Response>,

    /// Network configuration
    config: NetworkConfig,
}

/// Configuration for neural network behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Learning rate for the entire network
    pub learning_rate: f64,

    /// Decay rate for memories
    pub memory_decay: f64,

    /// Threshold for response activation
    pub response_threshold: f64,

    /// Maximum network complexity
    pub max_neurons: usize,

    /// Adaptation speed
    pub adaptation_rate: f64,

    /// Noise level in neural processing
    pub noise_level: f64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            memory_decay: 0.01,
            response_threshold: 0.7,
            max_neurons: 50,
            adaptation_rate: 0.05,
            noise_level: 0.02,
        }
    }
}

impl CellNeuralNetwork {
    /// Create a new neural network for a cell
    pub fn new(cell_type: UICellType) -> Self {
        let config = NetworkConfig::default();
        let mut network = Self {
            neurons: HashMap::new(),
            sensor_neurons: HashMap::new(),
            motor_neurons: HashMap::new(),
            hidden_layers: Vec::new(),
            current_stimuli: Vec::new(),
            learned_responses: HashMap::new(),
            config,
        };

        network.initialize_for_cell_type(cell_type);
        network
    }

    /// Initialize network topology based on cell type
    fn initialize_for_cell_type(&mut self, cell_type: UICellType) {
        // Create sensory neurons based on cell type capabilities
        let sensor_types = match cell_type {
            UICellType::ButtonCore => vec![
                SensorType::Touch,
                SensorType::Proximity,
                SensorType::Attention,
            ],
            UICellType::InputField => vec![
                SensorType::Touch,
                SensorType::Keyboard,
                SensorType::Attention,
            ],
            UICellType::Sensor => vec![
                SensorType::Position,
                SensorType::Movement,
                SensorType::Proximity,
                SensorType::Social,
                SensorType::Environmental,
            ],
            UICellType::Memory => {
                vec![SensorType::Temporal, SensorType::Energy, SensorType::Social]
            }
            _ => vec![SensorType::Proximity, SensorType::Social],
        };

        // Create sensor neurons
        for sensor_type in sensor_types {
            let neuron = Neuron::new(NeuronType::Sensor);
            let neuron_id = neuron.id;
            self.neurons.insert(neuron_id, neuron);
            self.sensor_neurons.insert(sensor_type, neuron_id);
        }

        // Create motor neurons
        let motor_actions = vec![
            "visual_change",
            "movement",
            "energy_change",
            "communication",
            "state_change",
            "learning",
        ];

        for action in motor_actions {
            let neuron = Neuron::new(NeuronType::Motor);
            let neuron_id = neuron.id;
            self.neurons.insert(neuron_id, neuron);
            self.motor_neurons.insert(action.to_string(), neuron_id);
        }

        // Create hidden layer
        let hidden_layer_size = 4;
        let mut hidden_layer = Vec::new();
        for _ in 0..hidden_layer_size {
            let neuron = Neuron::new(NeuronType::Processor);
            let neuron_id = neuron.id;
            hidden_layer.push(neuron_id);
            self.neurons.insert(neuron_id, neuron);
        }
        self.hidden_layers.push(hidden_layer);

        // Connect sensors to hidden layer
        for sensor_id in self.sensor_neurons.values() {
            for hidden_id in &self.hidden_layers[0] {
                if let Some(hidden_neuron) = self.neurons.get_mut(hidden_id) {
                    hidden_neuron.add_input(*sensor_id, rand::random::<f64>() * 0.5);
                }
                if let Some(sensor_neuron) = self.neurons.get_mut(sensor_id) {
                    sensor_neuron.add_output(*hidden_id, rand::random::<f64>() * 0.5);
                }
            }
        }

        // Connect hidden layer to motors
        for motor_id in self.motor_neurons.values() {
            for hidden_id in &self.hidden_layers[0] {
                if let Some(motor_neuron) = self.neurons.get_mut(motor_id) {
                    motor_neuron.add_input(*hidden_id, rand::random::<f64>() * 0.5);
                }
                if let Some(hidden_neuron) = self.neurons.get_mut(hidden_id) {
                    hidden_neuron.add_output(*motor_id, rand::random::<f64>() * 0.5);
                }
            }
        }
    }

    /// Process a stimulus and generate responses
    pub fn process_stimulus(&mut self, stimulus: Stimulus, cell_position: &GA3) -> Vec<Response> {
        // Check if stimulus affects this cell
        if !stimulus.affects_position(cell_position) {
            return Vec::new();
        }

        // Add to current stimuli
        self.current_stimuli.push(stimulus.clone());

        // Activate sensor neurons
        if let Some(&sensor_id) = self.sensor_neurons.get(&stimulus.sensor_type) {
            let intensity = stimulus.intensity_at(cell_position);
            if let Some(sensor_neuron) = self.neurons.get_mut(&sensor_id) {
                sensor_neuron.activation = intensity;
            }
        }

        // Update network
        self.update_network(0.1); // Fixed timestep for processing

        // Generate responses from motor neurons
        self.generate_responses()
    }

    /// Update the entire neural network
    fn update_network(&mut self, dt: UITime) {
        // Update all neurons
        let neuron_ids: Vec<Uuid> = self.neurons.keys().cloned().collect();

        for neuron_id in neuron_ids {
            // Calculate inputs from connected neurons
            let mut total_input = 0.0;

            if let Some(neuron) = self.neurons.get(&neuron_id) {
                for (input_id, weight) in &neuron.inputs {
                    if let Some(input_neuron) = self.neurons.get(input_id) {
                        total_input += input_neuron.activation * weight;
                    }
                }
            }

            // Update neuron with calculated input
            if let Some(neuron) = self.neurons.get_mut(&neuron_id) {
                // Apply sigmoid activation function
                let net_input = total_input - neuron.threshold;
                neuron.activation = 1.0 / (1.0 + (-net_input).exp());

                // Add noise
                neuron.activation += (rand::random::<f64>() - 0.5) * self.config.noise_level;
                neuron.activation = neuron.activation.clamp(0.0, 1.0);

                neuron.update(dt);
            }
        }

        // Apply learning
        self.apply_learning(dt);

        // Decay memories
        self.decay_memories(dt);
    }

    /// Generate responses based on motor neuron activation
    fn generate_responses(&self) -> Vec<Response> {
        let mut responses = Vec::new();

        for (action_name, motor_id) in &self.motor_neurons {
            if let Some(motor_neuron) = self.neurons.get(motor_id) {
                if motor_neuron.activation > self.config.response_threshold {
                    let response =
                        self.create_response_for_action(action_name, motor_neuron.activation);
                    responses.push(response);
                }
            }
        }

        responses
    }

    /// Create a specific response based on action type and intensity
    fn create_response_for_action(&self, action_name: &str, intensity: f64) -> Response {
        let actions = match action_name {
            "visual_change" => vec![Action::VisualChange {
                color_delta: [0.0, 0.0, 0.0, 0.0],
                size_delta: intensity * 0.2,
                opacity_delta: intensity * 0.1,
                glow_delta: intensity * 0.5,
            }],
            "movement" => vec![Action::Movement {
                force: vector3(
                    (rand::random::<f64>() - 0.5) * intensity,
                    (rand::random::<f64>() - 0.5) * intensity,
                    0.0,
                ),
                torque: GA3::zero(),
            }],
            "energy_change" => vec![Action::EnergyChange {
                delta: intensity * 5.0,
            }],
            "communication" => vec![Action::Communication {
                message: format!("activation_{:.2}", intensity),
                range: intensity * 3.0,
            }],
            "state_change" => vec![Action::StateChange {
                new_state: "stimulated".to_string(),
            }],
            "learning" => vec![Action::LearningUpdate {
                gene: "adaptability".to_string(),
                adjustment: intensity * 0.01,
            }],
            _ => vec![],
        };

        Response {
            response_type: ResponseType::Adaptive,
            intensity,
            energy_cost: intensity * 2.0,
            duration: intensity * 1.0,
            actions,
        }
    }

    /// Apply learning rules to update connection weights
    fn apply_learning(&mut self, dt: UITime) {
        // Simplified Hebbian learning: strengthen connections between co-active neurons
        let neuron_ids: Vec<Uuid> = self.neurons.keys().cloned().collect();

        for i in 0..neuron_ids.len() {
            for j in (i + 1)..neuron_ids.len() {
                let id_a = neuron_ids[i];
                let id_b = neuron_ids[j];

                let (activation_a, activation_b) = {
                    let neuron_a = self.neurons.get(&id_a).unwrap();
                    let neuron_b = self.neurons.get(&id_b).unwrap();
                    (neuron_a.activation, neuron_b.activation)
                };

                // Hebbian rule: Δw = η * a_i * a_j
                let weight_delta = self.config.learning_rate * activation_a * activation_b * dt;

                // Update weights if connection exists
                if let Some(neuron_a) = self.neurons.get_mut(&id_a) {
                    if neuron_a.outputs.contains_key(&id_b) {
                        neuron_a.update_weight(&id_b, weight_delta);
                    }
                }
                if let Some(neuron_b) = self.neurons.get_mut(&id_b) {
                    if neuron_b.outputs.contains_key(&id_a) {
                        neuron_b.update_weight(&id_a, weight_delta);
                    }
                }
            }
        }
    }

    /// Decay memories over time
    fn decay_memories(&mut self, dt: UITime) {
        let decay_factor = 1.0 - (self.config.memory_decay * dt);

        for neuron in self.neurons.values_mut() {
            for value in neuron.memory.values_mut() {
                *value *= decay_factor;
            }

            // Remove very weak memories
            neuron.memory.retain(|_, &mut value| value > 0.01);
        }
    }

    /// Learn a new response pattern
    pub fn learn_response(&mut self, stimulus_pattern: String, response: Response) {
        self.learned_responses.insert(stimulus_pattern, response);
    }

    /// Get network statistics
    pub fn get_statistics(&self) -> NetworkStatistics {
        let total_neurons = self.neurons.len();
        let active_neurons = self.neurons.values().filter(|n| n.activation > 0.1).count();

        let total_connections = self
            .neurons
            .values()
            .map(|n| n.inputs.len() + n.outputs.len())
            .sum::<usize>();

        let average_activation = if total_neurons > 0 {
            self.neurons.values().map(|n| n.activation).sum::<f64>() / total_neurons as f64
        } else {
            0.0
        };

        let memory_count = self.neurons.values().map(|n| n.memory.len()).sum::<usize>();

        NetworkStatistics {
            total_neurons,
            active_neurons,
            total_connections,
            average_activation,
            memory_count,
            learned_responses: self.learned_responses.len(),
        }
    }

    /// Reset network state
    pub fn reset(&mut self) {
        for neuron in self.neurons.values_mut() {
            neuron.activation = 0.0;
            neuron.fatigue = 0.0;
        }
        self.current_stimuli.clear();
    }

    /// Update configuration
    pub fn set_config(&mut self, config: NetworkConfig) {
        // Extract values before moving config
        let learning_rate = config.learning_rate;
        self.config = config;

        // Update all neurons with new learning rate
        for neuron in self.neurons.values_mut() {
            neuron.learning_rate = learning_rate;
        }
    }
}

/// Statistics about the neural network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatistics {
    pub total_neurons: usize,
    pub active_neurons: usize,
    pub total_connections: usize,
    pub average_activation: f64,
    pub memory_count: usize,
    pub learned_responses: usize,
}

/// Main nervous system that coordinates multiple cell networks
pub struct NervousSystem {
    /// Neural networks for each cell
    cell_networks: HashMap<Uuid, CellNeuralNetwork>,

    /// Global stimulus queue
    stimulus_queue: Vec<Stimulus>,

    /// System-wide configuration
    #[allow(dead_code)]
    config: NetworkConfig,

    /// Global memory shared across cells
    global_memory: HashMap<String, f64>,
}

impl Default for NervousSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl NervousSystem {
    /// Create a new nervous system
    pub fn new() -> Self {
        Self {
            cell_networks: HashMap::new(),
            stimulus_queue: Vec::new(),
            config: NetworkConfig::default(),
            global_memory: HashMap::new(),
        }
    }

    /// Add a cell to the nervous system
    pub fn add_cell(&mut self, cell: &UICell) {
        let network = CellNeuralNetwork::new(cell.cell_type());
        self.cell_networks.insert(cell.id(), network);
    }

    /// Remove a cell from the nervous system
    pub fn remove_cell(&mut self, cell_id: &Uuid) {
        self.cell_networks.remove(cell_id);
    }

    /// Process a global stimulus
    pub fn process_stimulus(&mut self, stimulus: Stimulus) {
        self.stimulus_queue.push(stimulus);
    }

    /// Update all neural networks
    pub fn update(
        &mut self,
        dt: UITime,
        cell_positions: &HashMap<Uuid, GA3>,
    ) -> HashMap<Uuid, Vec<Response>> {
        let mut all_responses = HashMap::new();

        // Process queued stimuli
        for stimulus in &self.stimulus_queue {
            for (cell_id, network) in &mut self.cell_networks {
                if let Some(position) = cell_positions.get(cell_id) {
                    let responses = network.process_stimulus(stimulus.clone(), position);
                    if !responses.is_empty() {
                        all_responses
                            .entry(*cell_id)
                            .or_insert_with(Vec::new)
                            .extend(responses);
                    }
                }
            }
        }

        // Clear processed stimuli
        self.stimulus_queue.clear();

        // Update all networks
        for network in self.cell_networks.values_mut() {
            network.update_network(dt);
        }

        all_responses
    }

    /// Get network for a specific cell
    pub fn get_network(&self, cell_id: &Uuid) -> Option<&CellNeuralNetwork> {
        self.cell_networks.get(cell_id)
    }

    /// Get mutable network for a specific cell
    pub fn get_network_mut(&mut self, cell_id: &Uuid) -> Option<&mut CellNeuralNetwork> {
        self.cell_networks.get_mut(cell_id)
    }

    /// Get system-wide statistics
    pub fn get_system_statistics(&self) -> SystemStatistics {
        let network_stats: Vec<_> = self
            .cell_networks
            .values()
            .map(|n| n.get_statistics())
            .collect();

        let total_neurons = network_stats.iter().map(|s| s.total_neurons).sum();
        let total_active = network_stats.iter().map(|s| s.active_neurons).sum();
        let total_connections = network_stats.iter().map(|s| s.total_connections).sum();
        let total_memories = network_stats.iter().map(|s| s.memory_count).sum();
        let total_learned = network_stats.iter().map(|s| s.learned_responses).sum();

        let average_activation = if !network_stats.is_empty() {
            network_stats
                .iter()
                .map(|s| s.average_activation)
                .sum::<f64>()
                / network_stats.len() as f64
        } else {
            0.0
        };

        SystemStatistics {
            total_cells: self.cell_networks.len(),
            total_neurons,
            active_neurons: total_active,
            total_connections,
            average_activation,
            memory_count: total_memories,
            learned_responses: total_learned,
            global_memories: self.global_memory.len(),
        }
    }
}

/// Statistics for the entire nervous system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatistics {
    pub total_cells: usize,
    pub total_neurons: usize,
    pub active_neurons: usize,
    pub total_connections: usize,
    pub average_activation: f64,
    pub memory_count: usize,
    pub learned_responses: usize,
    pub global_memories: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_cell::UICellType;

    #[test]
    fn test_stimulus_creation() {
        let position = vector3(1.0, 2.0, 0.0);
        let stimulus = Stimulus::new(SensorType::Touch, position, 0.8);

        assert_eq!(stimulus.sensor_type, SensorType::Touch);
        assert_eq!(stimulus.intensity, 0.8);
    }

    #[test]
    fn test_stimulus_affects_position() {
        let stimulus_pos = vector3(0.0, 0.0, 0.0);
        let stimulus = Stimulus::new(SensorType::Touch, stimulus_pos, 1.0);

        let close_pos = vector3(0.5, 0.0, 0.0);
        let far_pos = vector3(5.0, 0.0, 0.0);

        assert!(stimulus.affects_position(&close_pos));
        assert!(!stimulus.affects_position(&far_pos));
    }

    #[test]
    fn test_neuron_creation() {
        let neuron = Neuron::new(NeuronType::Sensor);

        assert_eq!(neuron.neuron_type, NeuronType::Sensor);
        assert_eq!(neuron.activation, 0.0);
        assert!(neuron.inputs.is_empty());
    }

    #[test]
    fn test_neural_network_creation() {
        let network = CellNeuralNetwork::new(UICellType::ButtonCore);

        assert!(!network.sensor_neurons.is_empty());
        assert!(!network.motor_neurons.is_empty());
        assert!(!network.neurons.is_empty());
    }

    #[test]
    fn test_nervous_system() {
        let mut system = NervousSystem::new();
        let position = vector3(0.0, 0.0, 0.0);
        let cell = UICell::new_at_position(UICellType::ButtonCore, position);

        system.add_cell(&cell);
        assert!(system.get_network(&cell.id()).is_some());

        let stats = system.get_system_statistics();
        assert_eq!(stats.total_cells, 1);
        assert!(stats.total_neurons > 0);
    }

    #[test]
    fn test_sensor_types() {
        assert_eq!(SensorType::Touch.detection_range(), 1.0);
        assert_eq!(SensorType::Proximity.detection_range(), 3.0);
        assert!(SensorType::Touch.sensitivity() > SensorType::Attention.sensitivity());
    }
}
