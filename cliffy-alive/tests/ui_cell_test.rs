use cliffy_alive::ui_cell::{CellGenome, UICell, UICellType};

#[test]
fn test_ui_cell_has_geometric_state() {
    // UI cells exist in 3D geometric space with separate visual properties
    let cell = UICell::new(UICellType::ButtonCore);

    let state = cell.nucleus();
    // GeometricState wraps a GA3 (3D Clifford algebra)
    assert!(state.multivector().magnitude() >= 0.0);

    // Can extract visual properties
    let position = cell.position(); // x, y from geometric state
    let size = cell.size(); // width, height from visual properties
    let opacity = cell.opacity(); // separate visual property

    assert!(position.x >= 0.0);
    assert!(size.width > 0.0);
    assert!((0.0..=1.0).contains(&opacity));
}

#[test]
fn test_cell_dna_determines_behavior() {
    let mut cell = UICell::new(UICellType::DataDisplay);

    // DNA determines affinity to other cells
    cell.dna.add_affinity_gene(UICellType::DataDisplay, 0.8); // likes similar cells
    cell.dna.add_affinity_gene(UICellType::Navigation, -0.3); // avoids nav

    let other_data = UICell::new(UICellType::DataDisplay);
    let nav = UICell::new(UICellType::Navigation);

    assert!(cell.affinity_to(&other_data) > 0.5);
    assert!(cell.affinity_to(&nav) < 0.0);
}

#[test]
fn test_cells_interact_geometrically() {
    let mut cell_a = UICell::new(UICellType::ButtonCore);
    let mut cell_b = UICell::new(UICellType::ButtonCore);

    // Set affinity between cells (negative for repulsion)
    cell_a
        .genome_mut()
        .add_affinity_gene(UICellType::ButtonCore, -50.0);

    // Position them near each other
    cell_a.set_position(100.0, 100.0);
    cell_b.set_position(110.0, 100.0);

    // Compute interaction force using geometric product
    let force = cell_a.interaction_force(&cell_b);

    // Should have some repulsion (negative affinity, close distance)
    assert!(force.magnitude() > 0.0);
}

#[test]
fn test_cell_energy_and_lifecycle() {
    let mut cell = UICell::new(UICellType::Generic);
    cell.set_energy(100.0);

    // Cells lose energy over time
    let _ = cell.metabolize(1.0); // 1 second
    assert!(cell.energy() < 100.0);

    // Cells gain energy from user interaction
    cell.receive_click();
    assert!(cell.energy() > 90.0);

    // Cells die when energy depleted
    cell.set_energy(0.5);
    assert!(cell.should_die());
}

#[test]
fn test_cell_geometric_transformations() {
    let mut cell = UICell::new(UICellType::Generic);

    // Test position mutation
    let original_pos = cell.position();
    cell.apply_geometric_mutation(0.1);
    let new_pos = cell.position();

    // Position should have changed slightly
    let distance =
        ((new_pos.x - original_pos.x).powi(2) + (new_pos.y - original_pos.y).powi(2)).sqrt();
    assert!(distance > 0.0);
    assert!(distance < 50.0); // But not too much
}

#[test]
fn test_cell_vitals_tracking() {
    let mut cell = UICell::new(UICellType::ButtonCore);

    // Initial interaction state should be healthy
    let state = cell.get_interaction_state();
    assert!(state.is_healthy());
    assert_eq!(state.stress_level, 0.0);

    // Stress the cell
    for _ in 0..10 {
        cell.apply_stress(0.1);
    }

    let stressed_state = cell.get_interaction_state();
    assert!(stressed_state.stress_level > 0.5);
    assert!(!stressed_state.is_healthy());
}

#[test]
fn test_cell_dna_inheritance() {
    let parent_a = UICell::new(UICellType::ButtonCore);
    let parent_b = UICell::new(UICellType::ButtonCore);

    // Modify parent DNAs
    let mut dna_a = parent_a.get_genome().clone();
    let mut dna_b = parent_b.get_genome().clone();

    dna_a.traits.insert("energy_efficiency".to_string(), 0.8);
    dna_b.traits.insert("cooperation".to_string(), 0.6);

    // Create child through crossover
    let child_dna = CellGenome::crossover(&dna_a, &dna_b, 0.5);

    // Child should have traits from both parents
    assert!(
        child_dna.traits.contains_key("energy_efficiency")
            || child_dna.traits.contains_key("cooperation")
    );
}

#[test]
fn test_cell_interaction_types() {
    let mut cell = UICell::new(UICellType::ButtonCore);

    // Test different interaction types
    cell.receive_hover();
    let hover_energy = cell.energy();

    cell.receive_click();
    let click_energy = cell.energy();

    // Click should provide more energy than hover
    assert!(click_energy > hover_energy);

    // Test focus interaction
    cell.receive_focus();
    assert!(cell.is_focused());
}

#[test]
fn test_cell_age_and_lifecycle_stages() {
    let mut cell = UICell::new(UICellType::Generic);

    // Start as juvenile
    assert!(cell.is_juvenile());

    // Age the cell
    for _ in 0..100 {
        cell.step(0.1);
    }

    // Should progress through lifecycle stages
    assert!(cell.age() > 5.0);
    assert!(!cell.is_juvenile());
}

#[test]
fn test_cell_genetic_expression() {
    let mut cell = UICell::new(UICellType::DataDisplay);

    // Modify genetic traits
    cell.dna
        .traits
        .insert("display_brightness".to_string(), 0.9);
    cell.dna.traits.insert("update_frequency".to_string(), 0.7);

    // Express genes
    cell.express_genes();

    // Visual properties should reflect genetic traits
    assert!(cell.opacity() > 0.8); // High brightness gene
    assert!(cell.get_update_frequency() > 0.6); // High update frequency
}
