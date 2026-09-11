//! Evolution and learning systems for living UI cells
//!
//! This module implements genetic algorithms and adaptive learning mechanisms
//! that allow UI elements to evolve and improve over time based on user
//! interaction patterns and environmental pressures.

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    ui_cell::{CellGenome, UICell},
    ui_organism::UIOrganismField,
    UITime,
};

/// Evolutionary strategies that can be applied to the UI organism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvolutionStrategy {
    /// Natural selection based on survival and reproduction
    NaturalSelection,
    /// Directed evolution toward specific traits
    DirectedEvolution { target_traits: HashMap<String, f64> },
    /// User-guided evolution based on interaction patterns
    UserGuided {
        interaction_weights: HashMap<String, f64>,
    },
    /// Hybrid approach combining multiple strategies
    Hybrid { strategies: Vec<EvolutionStrategy> },
}

/// Learning mechanisms for individual cells
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningMechanism {
    /// Reinforcement learning from user interactions
    Reinforcement,
    /// Imitation learning from successful neighbors
    Imitation,
    /// Adaptive learning that adjusts to environment
    Adaptive,
    /// Hebbian learning for connection strengthening
    Hebbian,
}

/// Mutation types that can occur during evolution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationType {
    /// Small random changes to gene values
    PointMutation,
    /// Larger jumps in gene values
    LargeMutation,
    /// Gene duplication
    GeneDuplication,
    /// Gene deletion
    GeneDeletion,
    /// Gene recombination between cells
    Recombination,
}

impl MutationType {
    /// Get the probability of this mutation type occurring
    pub fn probability(&self) -> f64 {
        match self {
            MutationType::PointMutation => 0.8,
            MutationType::LargeMutation => 0.15,
            MutationType::GeneDuplication => 0.02,
            MutationType::GeneDeletion => 0.02,
            MutationType::Recombination => 0.01,
        }
    }

    /// Get the intensity of this mutation type
    pub fn intensity(&self) -> f64 {
        match self {
            MutationType::PointMutation => 0.05,
            MutationType::LargeMutation => 0.3,
            MutationType::GeneDuplication => 1.0,
            MutationType::GeneDeletion => 1.0,
            MutationType::Recombination => 0.5,
        }
    }
}

/// Manages evolution and learning for the UI organism
#[derive(Debug, Clone)]
pub struct EvolutionEngine {
    /// Current evolution strategy
    strategy: EvolutionStrategy,

    /// Mutation rate (probability per generation)
    mutation_rate: f64,

    /// Selection pressure strength
    selection_strength: f64,

    /// Learning rate for adaptive behaviors
    learning_rate: f64,

    /// Memory of successful traits
    trait_memory: HashMap<String, f64>,

    /// Interaction history for learning
    interaction_history: Vec<InteractionEvent>,

    /// Performance metrics
    performance_metrics: PerformanceMetrics,
}

/// Record of user interactions for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionEvent {
    pub cell_id: uuid::Uuid,
    pub interaction_type: String,
    pub timestamp: UITime,
    pub intensity: f64,
    pub cell_traits: HashMap<String, f64>,
    pub outcome: InteractionOutcome,
}

/// Outcome of a user interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionOutcome {
    /// User responded positively (continued interaction)
    Positive,
    /// User responded neutrally
    Neutral,
    /// User responded negatively (stopped interaction)
    Negative,
    /// User performed desired action
    Success,
    /// User abandoned interaction
    Failure,
}

/// Performance metrics for tracking evolution success
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub generation: u64,
    pub average_fitness: f64,
    pub trait_averages: HashMap<String, f64>,
    pub interaction_success_rate: f64,
    pub adaptation_speed: f64,
    pub diversity_index: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            generation: 0,
            average_fitness: 0.0,
            trait_averages: HashMap::new(),
            interaction_success_rate: 0.0,
            adaptation_speed: 0.0,
            diversity_index: 0.0,
        }
    }
}

impl EvolutionEngine {
    /// Create a new evolution engine
    pub fn new(strategy: EvolutionStrategy) -> Self {
        Self {
            strategy,
            mutation_rate: 0.05,
            selection_strength: 1.0,
            learning_rate: 0.1,
            trait_memory: HashMap::new(),
            interaction_history: Vec::new(),
            performance_metrics: PerformanceMetrics::default(),
        }
    }

    /// Create an evolution engine with natural selection
    pub fn natural_selection() -> Self {
        Self::new(EvolutionStrategy::NaturalSelection)
    }

    /// Create an evolution engine with directed evolution
    pub fn directed_evolution(target_traits: HashMap<String, f64>) -> Self {
        Self::new(EvolutionStrategy::DirectedEvolution { target_traits })
    }

    /// Create an evolution engine with user-guided evolution
    pub fn user_guided(interaction_weights: HashMap<String, f64>) -> Self {
        Self::new(EvolutionStrategy::UserGuided {
            interaction_weights,
        })
    }

    /// Apply evolutionary pressure to an organism
    pub fn evolve_organism(&mut self, organism: &mut UIOrganismField, dt: UITime) {
        match &self.strategy.clone() {
            EvolutionStrategy::NaturalSelection => {
                self.apply_natural_selection(organism, dt);
            }
            EvolutionStrategy::DirectedEvolution { target_traits } => {
                self.apply_directed_evolution(organism, target_traits, dt);
            }
            EvolutionStrategy::UserGuided {
                interaction_weights,
            } => {
                self.apply_user_guided_evolution(organism, interaction_weights, dt);
            }
            EvolutionStrategy::Hybrid { strategies } => {
                for strategy in strategies {
                    let mut sub_engine = EvolutionEngine::new(strategy.clone());
                    sub_engine.evolve_organism(organism, dt / strategies.len() as f64);
                }
            }
        }

        // Update performance metrics
        self.update_performance_metrics(organism);
    }

    /// Apply natural selection pressure
    fn apply_natural_selection(&mut self, organism: &mut UIOrganismField, dt: UITime) {
        // Calculate fitness for each cell based on survival metrics
        let mut fitness_scores = HashMap::new();

        for ((_x, _y), cell) in organism.iter_cells() {
            let fitness = self.calculate_natural_fitness(cell);
            fitness_scores.insert(cell.id(), fitness);
        }

        // Apply selection pressure based on fitness
        self.apply_fitness_selection(organism, &fitness_scores, dt);
    }

    /// Apply directed evolution toward specific traits
    fn apply_directed_evolution(
        &mut self,
        organism: &mut UIOrganismField,
        target_traits: &HashMap<String, f64>,
        dt: UITime,
    ) {
        // Calculate fitness based on proximity to target traits
        let mut fitness_scores = HashMap::new();

        for ((_x, _y), cell) in organism.iter_cells() {
            let fitness = self.calculate_trait_fitness(cell, target_traits);
            fitness_scores.insert(cell.id(), fitness);
        }

        // Apply selection pressure
        self.apply_fitness_selection(organism, &fitness_scores, dt);

        // Apply directed mutations
        self.apply_directed_mutations(organism, target_traits, dt);
    }

    /// Apply user-guided evolution based on interaction patterns
    fn apply_user_guided_evolution(
        &mut self,
        organism: &mut UIOrganismField,
        interaction_weights: &HashMap<String, f64>,
        dt: UITime,
    ) {
        // Calculate fitness based on user interaction success
        let mut fitness_scores = HashMap::new();

        for ((_x, _y), cell) in organism.iter_cells() {
            let fitness = self.calculate_interaction_fitness(cell, interaction_weights);
            fitness_scores.insert(cell.id(), fitness);
        }

        // Apply selection pressure
        self.apply_fitness_selection(organism, &fitness_scores, dt);

        // Learn from interaction patterns
        self.apply_interaction_learning(organism, dt);
    }

    /// Calculate natural fitness based on survival metrics
    fn calculate_natural_fitness(&self, cell: &UICell) -> f64 {
        let energy_ratio = cell.energy_level() / 100.0; // Normalized energy
        let age_factor = (cell.age() / 50.0).min(1.0); // Age contribution
        let efficiency = cell.genome().get_gene("energy_efficiency");

        (energy_ratio * 0.4 + age_factor * 0.3 + efficiency * 0.3).clamp(0.0, 1.0)
    }

    /// Calculate fitness based on proximity to target traits
    fn calculate_trait_fitness(&self, cell: &UICell, target_traits: &HashMap<String, f64>) -> f64 {
        let mut total_distance = 0.0;
        let mut trait_count = 0;

        for (trait_name, target_value) in target_traits {
            let current_value = cell.genome().get_gene(trait_name);
            let distance = (current_value - target_value).abs();
            total_distance += distance;
            trait_count += 1;
        }

        if trait_count > 0 {
            1.0 - (total_distance / trait_count as f64)
        } else {
            0.5
        }
    }

    /// Calculate fitness based on user interaction success
    fn calculate_interaction_fitness(&self, cell: &UICell, weights: &HashMap<String, f64>) -> f64 {
        // Find interactions for this cell
        let cell_interactions: Vec<_> = self
            .interaction_history
            .iter()
            .filter(|event| event.cell_id == cell.id())
            .collect();

        if cell_interactions.is_empty() {
            return 0.5; // Neutral fitness for cells without interactions
        }

        let mut weighted_score = 0.0;
        let mut total_weight = 0.0;

        for interaction in cell_interactions {
            let base_score = match interaction.outcome {
                InteractionOutcome::Positive => 0.8,
                InteractionOutcome::Success => 1.0,
                InteractionOutcome::Neutral => 0.5,
                InteractionOutcome::Negative => 0.2,
                InteractionOutcome::Failure => 0.0,
            };

            let weight = weights
                .get(&interaction.interaction_type)
                .copied()
                .unwrap_or(1.0);

            weighted_score += base_score * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            0.5
        }
    }

    /// Apply fitness-based selection pressure
    fn apply_fitness_selection(
        &mut self,
        organism: &mut UIOrganismField,
        fitness_scores: &HashMap<uuid::Uuid, f64>,
        dt: UITime,
    ) {
        // Reward high-fitness cells with energy
        for ((_x, _y), cell) in organism.iter_cells() {
            if let Some(&fitness) = fitness_scores.get(&cell.id()) {
                let _energy_reward = fitness * self.selection_strength * dt * 5.0;
                // Note: Would need mutable access to organism to apply reward
                // This would be implemented in the organism's step function
            }
        }
    }

    /// Apply directed mutations toward target traits
    fn apply_directed_mutations(
        &mut self,
        organism: &mut UIOrganismField,
        _target_traits: &HashMap<String, f64>,
        dt: UITime,
    ) {
        let mut rng = thread_rng();

        // Note: This would require mutable access to cells
        // In practice, this would be applied during cell reproduction
        for ((_x, _y), _cell) in organism.iter_cells() {
            if rng.gen::<f64>() < self.mutation_rate * dt {
                // Apply directed mutation toward targets
                // This would modify the cell's genome
            }
        }
    }

    /// Apply learning from interaction patterns
    fn apply_interaction_learning(&mut self, organism: &mut UIOrganismField, _dt: UITime) {
        // Analyze recent interactions to identify successful patterns
        let recent_interactions: Vec<_> = self
            .interaction_history
            .iter()
            .filter(|event| event.timestamp > organism.total_time() - 10.0) // Last 10 time units
            .collect();

        // Update trait memory based on successful interactions
        for interaction in recent_interactions {
            if matches!(
                interaction.outcome,
                InteractionOutcome::Success | InteractionOutcome::Positive
            ) {
                for (trait_name, trait_value) in &interaction.cell_traits {
                    let current_memory = self.trait_memory.get(trait_name).copied().unwrap_or(0.5);
                    let new_memory =
                        current_memory + (trait_value - current_memory) * self.learning_rate;
                    self.trait_memory.insert(trait_name.clone(), new_memory);
                }
            }
        }
    }

    /// Record a user interaction event
    pub fn record_interaction(&mut self, event: InteractionEvent) {
        self.interaction_history.push(event);

        // Keep history size manageable
        if self.interaction_history.len() > 1000 {
            self.interaction_history.drain(0..100); // Remove oldest 100 events
        }
    }

    /// Mutate a cell's genome
    pub fn mutate_genome(&self, genome: &mut CellGenome) {
        let _rng = thread_rng();

        // Determine mutation type
        let mutation_type = self.select_mutation_type();

        match mutation_type {
            MutationType::PointMutation => {
                self.apply_point_mutation(genome);
            }
            MutationType::LargeMutation => {
                self.apply_large_mutation(genome);
            }
            MutationType::GeneDuplication => {
                self.apply_gene_duplication(genome);
            }
            MutationType::GeneDeletion => {
                self.apply_gene_deletion(genome);
            }
            MutationType::Recombination => {
                // Would require access to another genome
                self.apply_point_mutation(genome); // Fallback
            }
        }
    }

    /// Select a mutation type based on probabilities
    fn select_mutation_type(&self) -> MutationType {
        let mut rng = thread_rng();
        let roll = rng.gen::<f64>();

        let types = [
            MutationType::PointMutation,
            MutationType::LargeMutation,
            MutationType::GeneDuplication,
            MutationType::GeneDeletion,
            MutationType::Recombination,
        ];

        let mut cumulative = 0.0;
        for mutation_type in types {
            cumulative += mutation_type.probability();
            if roll < cumulative {
                return mutation_type;
            }
        }

        MutationType::PointMutation // Fallback
    }

    /// Apply a point mutation to a genome
    fn apply_point_mutation(&self, genome: &mut CellGenome) {
        let mut rng = thread_rng();
        let gene_names = [
            "growth_rate",
            "energy_efficiency",
            "cooperation",
            "adaptability",
            "visual_appeal",
            "responsiveness",
        ];

        let gene_name = gene_names[rng.gen_range(0..gene_names.len())];
        let current_value = genome.get_gene(gene_name);
        let mutation_amount = rng.gen_range(-0.1..=0.1);
        let new_value = (current_value + mutation_amount).clamp(0.0, 1.0);

        genome.set_gene(gene_name, new_value);
    }

    /// Apply a large mutation to a genome
    fn apply_large_mutation(&self, genome: &mut CellGenome) {
        let mut rng = thread_rng();
        let gene_names = [
            "growth_rate",
            "energy_efficiency",
            "cooperation",
            "adaptability",
            "visual_appeal",
            "responsiveness",
        ];

        let gene_name = gene_names[rng.gen_range(0..gene_names.len())];
        let new_value = rng.gen::<f64>();

        genome.set_gene(gene_name, new_value);
    }

    /// Apply gene duplication
    fn apply_gene_duplication(&self, genome: &mut CellGenome) {
        // For simplicity, just boost a random gene
        let mut rng = thread_rng();
        let gene_names = [
            "growth_rate",
            "energy_efficiency",
            "cooperation",
            "adaptability",
            "visual_appeal",
            "responsiveness",
        ];

        let gene_name = gene_names[rng.gen_range(0..gene_names.len())];
        let current_value = genome.get_gene(gene_name);
        let boosted_value = (current_value * 1.5).min(1.0);

        genome.set_gene(gene_name, boosted_value);
    }

    /// Apply gene deletion
    fn apply_gene_deletion(&self, genome: &mut CellGenome) {
        // For simplicity, just reduce a random gene
        let mut rng = thread_rng();
        let gene_names = [
            "growth_rate",
            "energy_efficiency",
            "cooperation",
            "adaptability",
            "visual_appeal",
            "responsiveness",
        ];

        let gene_name = gene_names[rng.gen_range(0..gene_names.len())];
        let current_value = genome.get_gene(gene_name);
        let reduced_value = (current_value * 0.5).max(0.0);

        genome.set_gene(gene_name, reduced_value);
    }

    /// Update performance metrics
    fn update_performance_metrics(&mut self, organism: &UIOrganismField) {
        let mut total_fitness = 0.0;
        let mut cell_count = 0;
        let mut trait_sums: HashMap<String, f64> = HashMap::new();
        let mut trait_counts: HashMap<String, usize> = HashMap::new();

        // Calculate metrics from all cells
        for ((_x, _y), cell) in organism.iter_cells() {
            let fitness = self.calculate_natural_fitness(cell);
            total_fitness += fitness;
            cell_count += 1;

            // Collect trait values
            for trait_name in [
                "growth_rate",
                "energy_efficiency",
                "cooperation",
                "adaptability",
                "visual_appeal",
                "responsiveness",
            ] {
                let value = cell.genome().get_gene(trait_name);
                *trait_sums.entry(trait_name.to_string()).or_insert(0.0) += value;
                *trait_counts.entry(trait_name.to_string()).or_insert(0) += 1;
            }
        }

        // Update metrics
        self.performance_metrics.average_fitness = if cell_count > 0 {
            total_fitness / cell_count as f64
        } else {
            0.0
        };

        self.performance_metrics.trait_averages.clear();
        for (trait_name, sum) in &trait_sums {
            if let Some(&count) = trait_counts.get(trait_name) {
                if count > 0 {
                    self.performance_metrics
                        .trait_averages
                        .insert(trait_name.clone(), sum / count as f64);
                }
            }
        }

        // Calculate interaction success rate
        let recent_interactions: Vec<_> = self
            .interaction_history
            .iter()
            .filter(|event| event.timestamp > organism.total_time() - 10.0)
            .collect();

        if !recent_interactions.is_empty() {
            let success_count = recent_interactions
                .iter()
                .filter(|event| {
                    matches!(
                        event.outcome,
                        InteractionOutcome::Success | InteractionOutcome::Positive
                    )
                })
                .count();

            self.performance_metrics.interaction_success_rate =
                success_count as f64 / recent_interactions.len() as f64;
        }

        // Calculate diversity index (simplified)
        let num_traits = trait_sums.len();
        let mut diversity = 0.0;
        for trait_values in trait_sums.values() {
            diversity += trait_values.abs(); // Simplified diversity measure
        }
        self.performance_metrics.diversity_index = diversity / num_traits as f64;
    }

    /// Get current performance metrics
    pub fn performance_metrics(&self) -> &PerformanceMetrics {
        &self.performance_metrics
    }

    /// Get learned trait patterns
    pub fn trait_memory(&self) -> &HashMap<String, f64> {
        &self.trait_memory
    }

    /// Set evolution parameters
    pub fn set_mutation_rate(&mut self, rate: f64) {
        self.mutation_rate = rate.clamp(0.0, 1.0);
    }

    pub fn set_selection_strength(&mut self, strength: f64) {
        self.selection_strength = strength.max(0.0);
    }

    pub fn set_learning_rate(&mut self, rate: f64) {
        self.learning_rate = rate.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_cell::UICellType;
    use cliffy_core::GA3;

    #[test]
    fn test_evolution_engine_creation() {
        let engine = EvolutionEngine::natural_selection();
        assert!(matches!(
            engine.strategy,
            EvolutionStrategy::NaturalSelection
        ));
    }

    #[test]
    fn test_mutation_type_probabilities() {
        let point_prob = MutationType::PointMutation.probability();
        let large_prob = MutationType::LargeMutation.probability();

        assert!(point_prob > large_prob);
        assert_eq!(point_prob, 0.8);
        assert_eq!(large_prob, 0.15);
    }

    #[test]
    fn test_fitness_calculation() {
        let engine = EvolutionEngine::natural_selection();
        let position = GA3::scalar(1.0);
        let cell = UICell::new_at_position(UICellType::ButtonCore, position);

        let fitness = engine.calculate_natural_fitness(&cell);
        assert!((0.0..=1.0).contains(&fitness));
    }

    #[test]
    fn test_interaction_recording() {
        let mut engine = EvolutionEngine::natural_selection();
        let cell_id = uuid::Uuid::new_v4();

        let event = InteractionEvent {
            cell_id,
            interaction_type: "click".to_string(),
            timestamp: 1.0,
            intensity: 0.8,
            cell_traits: HashMap::new(),
            outcome: InteractionOutcome::Success,
        };

        engine.record_interaction(event);
        assert_eq!(engine.interaction_history.len(), 1);
    }

    #[test]
    fn test_genome_mutation() {
        let engine = EvolutionEngine::natural_selection();
        let mut genome = CellGenome::new();

        let _original_value = genome.get_gene("growth_rate");
        engine.mutate_genome(&mut genome);

        // Mutation might or might not change this specific gene
        // Just verify the genome is still valid
        let new_value = genome.get_gene("growth_rate");
        assert!((0.0..=1.0).contains(&new_value));
    }

    #[test]
    fn test_trait_fitness() {
        let engine = EvolutionEngine::natural_selection();
        let position = GA3::scalar(1.0);
        let cell = UICell::new_at_position(UICellType::ButtonCore, position);

        let mut target_traits = HashMap::new();
        target_traits.insert("energy_efficiency".to_string(), 0.8);
        target_traits.insert("cooperation".to_string(), 0.6);

        let fitness = engine.calculate_trait_fitness(&cell, &target_traits);
        assert!((0.0..=1.0).contains(&fitness));
    }
}
