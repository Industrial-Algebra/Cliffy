use cliffy_alive::{LivingUI, AliveUI, LivingComponent};
use cliffy_alive::ui_cell::{UICell, UICellType, InteractionType};
use cliffy_alive::ui_organism::UIOrganismField;
use wasm_bindgen_test::*;
use web_sys::{Event, MouseEvent};
use std::collections::HashMap;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_living_ui_responds_to_user() {
    let mut ui = LivingUI::new();
    
    // Spawn initial button
    ui.spawn_cell(UICellType::ButtonCore, 100.0, 100.0);
    
    // Simulate user hovering
    ui.handle_event(Event::MouseMove(100.0, 100.0));
    
    // Button should respond (grow slightly, increase opacity)
    let button = ui.get_cell_at(100.0, 100.0).unwrap();
    assert!(button.scale() > 1.0);
}

#[wasm_bindgen_test]
fn test_ui_self_assembles() {
    let mut ui = LivingUI::new();
    
    // Just add data sources, no positioning
    ui.add_data_source("users", DataStream::new());
    ui.add_data_source("sales", DataStream::new());
    ui.add_data_source("metrics", DataStream::new());
    
    // Let it self-assemble for a few seconds
    for _ in 0..180 { // 3 seconds at 60fps
        ui.evolve(0.016);
    }
    
    // Should have organized into a coherent layout
    let layout_score = ui.evaluate_layout_coherence();
    assert!(layout_score > 0.7);
    
    // Related data should be near each other
    let users_pos = ui.find_cell_by_data("users").position();
    let sales_pos = ui.find_cell_by_data("sales").position();
    let distance = (users_pos - sales_pos).magnitude();
    
    assert!(distance < 200.0); // Should be relatively close
}

#[wasm_bindgen_test]
fn test_ui_learns_user_preferences() {
    let mut ui = LivingUI::with_learning();
    
    // User repeatedly clicks on data viz
    for _ in 0..10 {
        ui.handle_event(Event::Click(300.0, 300.0)); // data viz location
        ui.evolve(0.1);
    }
    
    // Data viz should become more prominent
    let data_viz = ui.find_cell_by_type(UICellType::DataVisualization);
    assert!(data_viz.size().area() > INITIAL_SIZE);
    assert!(data_viz.position().distance_to_center() < INITIAL_DISTANCE);
}

#[wasm_bindgen_test]
fn test_living_ui_creation() {
    let ui = LivingUI::new();
    
    // Should start with empty organism field
    assert_eq!(ui.population(), 0);
    assert!(ui.is_alive());
    
    // Should have default configuration
    let config = ui.get_config();
    assert!(config.max_population > 0);
    assert!(config.mutation_rate > 0.0);
}

#[wasm_bindgen_test]
fn test_cell_lifecycle_in_ui() {
    let mut ui = LivingUI::new();
    
    // Spawn a cell
    let cell_id = ui.spawn_cell(UICellType::Generic, 50.0, 50.0);
    assert_eq!(ui.population(), 1);
    
    // Cell should be alive initially
    let cell = ui.get_cell(cell_id).unwrap();
    assert!(cell.is_alive());
    assert!(cell.energy() > 50.0);
    
    // Drain cell energy
    for _ in 0..1000 {
        ui.step(0.1); // Large time steps to drain energy quickly
    }
    
    // Cell should eventually die and be removed
    assert!(ui.population() == 0 || ui.get_cell(cell_id).is_none());
}

#[wasm_bindgen_test]
fn test_ui_interaction_events() {
    let mut ui = LivingUI::new();
    
    // Spawn interactive cells
    ui.spawn_cell(UICellType::ButtonCore, 100.0, 100.0);
    ui.spawn_cell(UICellType::InputField, 200.0, 100.0);
    
    // Test mouse events
    ui.handle_mouse_move(105.0, 105.0);
    ui.handle_mouse_click(105.0, 105.0);
    
    // Button should respond to interaction
    let button = ui.get_cell_at(100.0, 100.0).unwrap();
    assert!(button.energy() > 80.0); // Should have gained energy from click
    
    // Test keyboard events
    ui.handle_key_press("Tab");
    
    // Should affect focus state
    let input = ui.get_cell_at(200.0, 100.0).unwrap();
    assert!(input.is_focused() || button.is_focused());
}

#[wasm_bindgen_test]
fn test_ui_responsive_adaptation() {
    let mut ui = LivingUI::new();
    
    // Start with desktop layout
    ui.set_viewport(1920, 1080);
    ui.populate_standard_layout();
    
    let desktop_layout = ui.capture_layout_snapshot();
    
    // Switch to mobile
    ui.set_viewport(375, 667);
    ui.trigger_responsive_adaptation();
    
    // Allow time to adapt
    for _ in 0..120 { // 2 seconds at 60fps
        ui.step(0.016);
    }
    
    let mobile_layout = ui.capture_layout_snapshot();
    
    // Layouts should be significantly different
    let layout_difference = ui.compare_layouts(&desktop_layout, &mobile_layout);
    assert!(layout_difference > 0.5);
}

#[wasm_bindgen_test]
fn test_ui_energy_system() {
    let mut ui = LivingUI::new();
    
    // Add energy sources (like user attention areas)
    ui.add_energy_source(100.0, 100.0, 10.0); // High energy area
    ui.add_energy_source(500.0, 500.0, 2.0);  // Low energy area
    
    // Spawn cells in different locations
    ui.spawn_cell(UICellType::Generic, 105.0, 105.0); // Near high energy
    ui.spawn_cell(UICellType::Generic, 505.0, 505.0); // Near low energy
    ui.spawn_cell(UICellType::Generic, 300.0, 300.0); // Middle
    
    // Run for some time
    for _ in 0..100 {
        ui.step(0.016);
    }
    
    // Cells near high energy source should have more energy
    let high_energy_cell = ui.get_cell_at(105.0, 105.0).unwrap();
    let low_energy_cell = ui.get_cell_at(505.0, 505.0).unwrap();
    
    assert!(high_energy_cell.energy() > low_energy_cell.energy());
}

#[wasm_bindgen_test]
fn test_ui_genetic_evolution() {
    let mut ui = LivingUI::with_genetics();
    
    // Set evolutionary pressure for user engagement
    ui.set_selection_pressure("user_engagement", 0.8);
    
    // Spawn population
    for i in 0..20 {
        let cell = ui.spawn_cell(UICellType::Generic, i as f64 * 30.0, 100.0);
        
        // Give some cells user interactions to create fitness variance
        if i % 3 == 0 {
            ui.simulate_user_interaction_on_cell(cell);
        }
    }
    
    let initial_avg_engagement = ui.average_trait_value("user_engagement");
    
    // Run evolution
    for _ in 0..50 {
        ui.evolve(0.1);
    }
    
    let final_avg_engagement = ui.average_trait_value("user_engagement");
    
    // User engagement should improve over time
    assert!(final_avg_engagement > initial_avg_engagement);
}

#[wasm_bindgen_test]
fn test_ui_spatial_organization() {
    let mut ui = LivingUI::new();
    
    // Configure cell affinities
    ui.set_cell_affinity(UICellType::DataDisplay, UICellType::DataVisualization, 0.8);
    ui.set_cell_affinity(UICellType::Navigation, UICellType::Header, 0.6);
    ui.set_cell_affinity(UICellType::DataDisplay, UICellType::Navigation, -0.3);
    
    // Spawn cells randomly
    ui.spawn_cell(UICellType::DataDisplay, 100.0, 300.0);
    ui.spawn_cell(UICellType::DataVisualization, 400.0, 100.0);
    ui.spawn_cell(UICellType::Navigation, 200.0, 400.0);
    ui.spawn_cell(UICellType::Header, 350.0, 200.0);
    
    // Let spatial forces organize the layout
    for _ in 0..200 {
        ui.spatial_step(0.016);
    }
    
    // Related cells should be closer together
    let data_pos = ui.find_cell_by_type(UICellType::DataDisplay).position();
    let viz_pos = ui.find_cell_by_type(UICellType::DataVisualization).position();
    let nav_pos = ui.find_cell_by_type(UICellType::Navigation).position();
    
    let data_viz_distance = data_pos.distance_to(&viz_pos);
    let data_nav_distance = data_pos.distance_to(&nav_pos);
    
    // Data display should be closer to visualization than navigation
    assert!(data_viz_distance < data_nav_distance);
}

#[wasm_bindgen_test]
fn test_ui_performance_monitoring() {
    let mut ui = LivingUI::new();
    ui.enable_performance_monitoring(true);
    
    // Populate with many cells
    for i in 0..50 {
        ui.spawn_cell(UICellType::Generic, (i % 10) as f64 * 80.0, (i / 10) as f64 * 60.0);
    }
    
    // Run simulation
    for frame in 0..120 {
        let start_time = web_sys::js_sys::Date::now();
        ui.step(0.016);
        let end_time = web_sys::js_sys::Date::now();
        
        ui.record_frame_time(end_time - start_time);
    }
    
    let perf_metrics = ui.get_performance_metrics();
    
    // Should maintain reasonable performance
    assert!(perf_metrics.average_frame_time < 16.0); // Under 16ms for 60fps
    assert!(perf_metrics.memory_usage < 100.0); // Reasonable memory usage
    assert!(perf_metrics.active_cells > 40); // Most cells should be active
}

#[wasm_bindgen_test]
fn test_ui_state_persistence() {
    let mut ui = LivingUI::new();
    
    // Create complex state
    ui.spawn_cell(UICellType::ButtonCore, 100.0, 100.0);
    ui.spawn_cell(UICellType::DataDisplay, 200.0, 200.0);
    ui.set_global_state("theme", "dark");
    ui.set_global_state("user_id", "12345");
    
    // Serialize state
    let serialized_state = ui.serialize_state().unwrap();
    
    // Create new UI and restore state
    let mut restored_ui = LivingUI::new();
    restored_ui.deserialize_state(&serialized_state).unwrap();
    
    // State should be restored
    assert_eq!(restored_ui.population(), 2);
    assert!(restored_ui.get_cell_at(100.0, 100.0).is_some());
    assert_eq!(restored_ui.get_global_state("theme").unwrap(), "dark");
    assert_eq!(restored_ui.get_global_state("user_id").unwrap(), "12345");
}

#[wasm_bindgen_test]
fn test_ui_accessibility_adaptation() {
    let mut ui = LivingUI::new();
    ui.enable_accessibility_mode(true);
    
    // Set accessibility preferences
    ui.set_accessibility_preference("high_contrast", true);
    ui.set_accessibility_preference("large_text", true);
    ui.set_accessibility_preference("motion_reduced", true);
    
    // Spawn cells
    ui.spawn_cell(UICellType::ButtonCore, 100.0, 100.0);
    ui.spawn_cell(UICellType::Header, 200.0, 50.0);
    
    // Allow adaptation
    for _ in 0..60 {
        ui.step(0.016);
    }
    
    // Cells should adapt to accessibility needs
    let button = ui.get_cell_at(100.0, 100.0).unwrap();
    let header = ui.get_cell_at(200.0, 50.0).unwrap();
    
    // Should have high contrast colors
    assert!(button.get_contrast_ratio() > 4.5);
    assert!(header.get_contrast_ratio() > 4.5);
    
    // Should have larger text
    assert!(button.get_font_size() > 16.0);
    assert!(header.get_font_size() > 20.0);
    
    // Should have reduced motion
    assert!(button.get_animation_speed() < 0.5);
    assert!(header.get_animation_speed() < 0.5);
}

// Constants for tests
const INITIAL_SIZE: f64 = 100.0;
const INITIAL_DISTANCE: f64 = 400.0;

// Helper structs
#[wasm_bindgen]
extern "C" {
    type DataStream;
    
    #[wasm_bindgen(constructor)]
    fn new() -> DataStream;
}

// Mock Event enum for testing
enum Event {
    MouseMove(f64, f64),
    Click(f64, f64),
}