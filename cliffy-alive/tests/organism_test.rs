use cliffy_alive::ui_cell::{InteractionType, UICellType};
use cliffy_alive::ui_organism::{EnergySource, SelectionPressure, UIOrganismField};
use cliffy_alive::AliveConfig;

#[test]
fn test_create_organism() {
    let config = AliveConfig::default();
    let organism = UIOrganismField::new((10, 10), config);

    assert_eq!(organism.dimensions(), (10, 10));
    assert_eq!(organism.cell_count(), 0);
    assert_eq!(organism.generation(), 0);
}

#[test]
fn test_plant_and_remove_cells() {
    let config = AliveConfig::default();
    let mut organism = UIOrganismField::new((10, 10), config);

    // Plant a cell
    let result = organism.plant_seed(5, 5, UICellType::ButtonCore);
    assert!(result.is_ok());
    assert_eq!(organism.cell_count(), 1);

    // Get the cell
    let cell = organism.get_cell(5, 5);
    assert!(cell.is_some());

    // Remove the cell
    let removed = organism.remove_cell(5, 5);
    assert!(removed.is_some());
    assert_eq!(organism.cell_count(), 0);
}

#[test]
fn test_step_advances_time() {
    let config = AliveConfig::default();
    let mut organism = UIOrganismField::new((5, 5), config);

    let initial_time = organism.total_time();
    organism.step(0.016);

    assert!(organism.total_time() > initial_time);
}

#[test]
fn test_energy_sources() {
    let config = AliveConfig::default();
    let mut organism = UIOrganismField::new((10, 10), config);

    // Add an energy source
    let source = EnergySource {
        position: (5, 5),
        radius: 2,
        energy_per_second: 10.0,
        is_active: true,
    };
    organism.add_energy_source(source);

    // Plant cells near the energy source
    organism.plant_seed(5, 5, UICellType::Generic).ok();
    organism.plant_seed(6, 5, UICellType::Generic).ok();

    let initial_energy = organism.total_energy();

    // Step to allow energy distribution
    organism.step(1.0);

    // Total energy should increase
    assert!(organism.total_energy() >= initial_energy);
}

#[test]
fn test_selection_pressures() {
    let config = AliveConfig::default();
    let mut organism = UIOrganismField::new((10, 10), config);

    // Add selection pressures
    organism.apply_selection_pressure(SelectionPressure::EnergyEfficiency);
    organism.apply_selection_pressure(SelectionPressure::Cooperation);

    // Plant some cells
    organism.plant_seed(3, 3, UICellType::ButtonCore).ok();
    organism.plant_seed(4, 4, UICellType::DataDisplay).ok();

    // Step to apply pressures
    organism.step(0.1);

    // Just verify it doesn't crash
    assert!(organism.living_cell_count() > 0);
}

#[test]
fn test_handle_interaction() {
    let config = AliveConfig::default();
    let mut organism = UIOrganismField::new((10, 10), config);

    // Plant a cell
    organism.plant_seed(5, 5, UICellType::ButtonCore).ok();

    // Interact with it
    organism.handle_interaction(5, 5, InteractionType::Click, 1.0);
    organism.handle_interaction(5, 5, InteractionType::Hover, 0.5);

    // Cell should still exist
    assert!(organism.get_cell(5, 5).is_some());
}

#[test]
fn test_cell_statistics() {
    let config = AliveConfig::default();
    let mut organism = UIOrganismField::new((10, 10), config);

    // Plant several cells
    organism.plant_seed(2, 2, UICellType::ButtonCore).ok();
    organism.plant_seed(3, 3, UICellType::Generic).ok();
    organism.plant_seed(4, 4, UICellType::DataDisplay).ok();

    assert_eq!(organism.cell_count(), 3);
    assert_eq!(organism.living_cell_count(), 3);
    assert!(organism.total_energy() > 0.0);

    // Age the cells
    organism.step(5.0);

    assert!(organism.average_age() > 0.0);
}

#[test]
fn test_feed_region() {
    let config = AliveConfig::default();
    let mut organism = UIOrganismField::new((10, 10), config);

    // Plant cells
    organism.plant_seed(5, 5, UICellType::Generic).ok();
    organism.plant_seed(6, 5, UICellType::Generic).ok();

    let initial_energy = organism.total_energy();

    // Feed the region
    organism.feed_region(5, 5, 2, 100.0);

    assert!(organism.total_energy() > initial_energy);
}

#[test]
fn test_snapshot_and_restore() {
    let config = AliveConfig::default();
    let mut organism = UIOrganismField::new((10, 10), config);

    // Plant cells and configure organism
    organism.plant_seed(3, 3, UICellType::ButtonCore).ok();
    organism.plant_seed(5, 5, UICellType::DataDisplay).ok();
    organism.step(2.0);

    // Take a snapshot
    let snapshot = organism.snapshot();

    // Make changes
    organism.plant_seed(7, 7, UICellType::Generic).ok();
    organism.step(1.0);
    assert_eq!(organism.cell_count(), 3);

    // Restore snapshot
    let result = organism.load_snapshot(snapshot);
    assert!(result.is_ok());
    assert_eq!(organism.cell_count(), 2);
}

#[test]
fn test_iter_cells() {
    let config = AliveConfig::default();
    let mut organism = UIOrganismField::new((10, 10), config);

    organism.plant_seed(2, 3, UICellType::ButtonCore).ok();
    organism.plant_seed(5, 7, UICellType::Generic).ok();
    organism.plant_seed(8, 1, UICellType::DataDisplay).ok();

    let cells: Vec<_> = organism.iter_cells().collect();
    assert_eq!(cells.len(), 3);
}
