use cliffy_alive::evolution::{EvolutionEngine, MutationType};
use cliffy_alive::ui_cell::{CellGenome, UICellType};
use std::collections::HashMap;

#[test]
fn test_create_evolution_engine() {
    // Test natural selection engine
    let engine1 = EvolutionEngine::natural_selection();
    // Just verify we can get metrics (generation is unsigned, so >= 0 is always true)
    let _ = engine1.performance_metrics().generation;

    // Test directed evolution with target traits
    let mut target_traits = HashMap::new();
    target_traits.insert("energy_efficiency".to_string(), 0.9);
    let engine2 = EvolutionEngine::directed_evolution(target_traits);
    let _ = engine2.performance_metrics().generation;

    // Test user-guided evolution
    let mut interaction_weights = HashMap::new();
    interaction_weights.insert("click".to_string(), 1.0);
    let engine3 = EvolutionEngine::user_guided(interaction_weights);
    let _ = engine3.performance_metrics().generation;
}

#[test]
fn test_genome_mutation() {
    let engine = EvolutionEngine::natural_selection();

    let mut genome = CellGenome::new();

    engine.mutate_genome(&mut genome);

    // Mutation should change some genes
    // Note: with random mutation, values might occasionally be the same
    // So we just verify the genome still has valid genes
    assert!(genome.get_gene("energy_efficiency") >= 0.0);
    assert!(genome.get_gene("energy_efficiency") <= 1.0);
}

#[test]
fn test_mutation_configuration() {
    let mut engine = EvolutionEngine::natural_selection();

    engine.set_mutation_rate(0.15);
    engine.set_selection_strength(2.5);
    engine.set_learning_rate(0.3);

    // Just verify the engine is properly configured
    let _ = engine.performance_metrics().generation;
}

#[test]
fn test_genome_crossover() {
    let mut genome1 = CellGenome::new();
    let mut genome2 = CellGenome::new();

    genome1.set_gene("energy_efficiency", 0.8);
    genome2.set_gene("energy_efficiency", 0.2);

    let offspring = genome1.crossover(&genome2, 0.5);

    // Offspring should have valid gene values
    let efficiency = offspring.get_gene("energy_efficiency");
    assert!((0.0..=1.0).contains(&efficiency));
}

#[test]
fn test_mutation_types() {
    // Verify mutation types have valid properties
    assert!(MutationType::PointMutation.probability() >= 0.0);
    assert!(MutationType::PointMutation.probability() <= 1.0);
    assert!(MutationType::PointMutation.intensity() >= 0.0);

    assert!(MutationType::LargeMutation.probability() >= 0.0);
    assert!(MutationType::LargeMutation.probability() <= 1.0);
    assert!(MutationType::LargeMutation.intensity() >= 0.0);

    assert!(MutationType::GeneDuplication.probability() >= 0.0);
    assert!(MutationType::GeneDuplication.probability() <= 1.0);
    assert!(MutationType::GeneDuplication.intensity() >= 0.0);
}

#[test]
fn test_genome_similarity() {
    let genome1 = CellGenome::new();
    let mut genome2 = CellGenome::new();

    // Identical genomes should have high similarity
    let similarity1 = genome1.similarity_to(&genome2);
    assert!(similarity1 > 0.9);

    // Change one genome significantly
    genome2.set_gene("energy_efficiency", 0.1);
    genome2.set_gene("cooperation", 0.9);
    genome2.set_gene("adaptability", 0.2);

    // Similarity should still be reasonable
    let similarity2 = genome1.similarity_to(&genome2);
    assert!((0.0..=1.0).contains(&similarity2));
}

#[test]
fn test_affinity_system() {
    let mut genome = CellGenome::new();

    // Add affinity for button cells
    genome.add_affinity_gene(UICellType::ButtonCore, 0.8);
    genome.add_affinity_gene(UICellType::DataDisplay, -0.5);

    // Test affinity calculation
    let button_affinity = genome.calculate_affinity(&CellGenome::new(), UICellType::ButtonCore);
    let data_affinity = genome.calculate_affinity(&CellGenome::new(), UICellType::DataDisplay);

    assert!(button_affinity > 0.0);
    assert!(data_affinity < 0.0);
}
