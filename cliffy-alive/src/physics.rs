//! Geometric physics for UI movement and dynamics
//!
//! This module implements physical simulation using Amari's geometric algebra
//! to create natural movement, forces, and interactions for living UI elements.

use cliffy_core::{GeometricState, Multivector, GA3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    ui_cell::{UICell, UICellType},
    UITime,
};

/// Default reactive GA3 for deserialization
fn default_reactive_ga3() -> GeometricState {
    GeometricState::new(GA3::zero())
}

/// Physical forces that can act on UI cells
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ForceType {
    /// Gravitational attraction between cells
    Gravity,
    /// Electromagnetic-like forces for interaction
    Electromagnetic,
    /// Spring forces for maintaining connections
    Spring,
    /// Damping forces to prevent excessive motion
    Damping,
    /// User interaction forces
    UserInteraction,
    /// Random thermal motion
    Thermal,
    /// Pressure from neighboring cells
    Pressure,
    /// Alignment forces for organizing structures
    Alignment,
}

impl ForceType {
    /// Get the base strength of this force type
    pub fn base_strength(&self) -> f64 {
        match self {
            ForceType::Gravity => 0.1,
            ForceType::Electromagnetic => 0.5,
            ForceType::Spring => 2.0,
            ForceType::Damping => 0.8,
            ForceType::UserInteraction => 5.0,
            ForceType::Thermal => 0.05,
            ForceType::Pressure => 1.0,
            ForceType::Alignment => 0.3,
        }
    }

    /// Get the effective range of this force
    pub fn range(&self) -> f64 {
        match self {
            ForceType::Gravity => 10.0,
            ForceType::Electromagnetic => 5.0,
            ForceType::Spring => 2.0,
            ForceType::Damping => 1.0,
            ForceType::UserInteraction => 3.0,
            ForceType::Thermal => 1.0,
            ForceType::Pressure => 1.5,
            ForceType::Alignment => 4.0,
        }
    }
}

/// Physical properties of a UI cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellPhysics {
    /// Current position in 3D space
    #[serde(skip, default = "default_reactive_ga3")]
    pub position: GeometricState,

    /// Current velocity
    #[serde(skip, default = "GA3::zero")]
    pub velocity: GA3,

    /// Current acceleration
    #[serde(skip, default = "GA3::zero")]
    pub acceleration: GA3,

    /// Mass of the cell (affects inertia)
    pub mass: f64,

    /// Charge for electromagnetic interactions
    pub charge: f64,

    /// Moment of inertia for rotational dynamics
    pub moment_of_inertia: f64,

    /// Angular velocity
    #[serde(skip, default = "GA3::zero")]
    pub angular_velocity: GA3,

    /// Angular acceleration
    #[serde(skip, default = "GA3::zero")]
    pub angular_acceleration: GA3,

    /// Friction coefficient
    pub friction: f64,

    /// Elasticity coefficient for collisions
    pub elasticity: f64,

    /// Whether the cell is affected by physics
    pub is_kinematic: bool,
}

impl CellPhysics {
    /// Create new physics properties for a cell
    pub fn new(position: GA3, cell_type: UICellType) -> Self {
        let reactive_position = GeometricState::new(position);

        // Set mass based on cell type
        let mass = match cell_type {
            UICellType::ButtonCore => 2.0,
            UICellType::ButtonEdge => 1.0,
            UICellType::InputField => 1.5,
            UICellType::TextDisplay => 0.5,
            UICellType::Container => 3.0,
            UICellType::Spacer => 0.1,
            UICellType::Connector => 0.3,
            UICellType::Decoration => 0.2,
            UICellType::Sensor => 0.8,
            UICellType::Memory => 1.2,
            _ => 1.0, // Default mass for other cell types
        };

        // Set charge based on cell type (for electromagnetic interactions)
        let charge = match cell_type {
            UICellType::ButtonCore => 1.0,
            UICellType::InputField => -0.5,
            UICellType::Sensor => 0.8,
            UICellType::Memory => -0.3,
            _ => 0.0,
        };

        Self {
            position: reactive_position,
            velocity: GA3::zero(),
            acceleration: GA3::zero(),
            mass,
            charge,
            moment_of_inertia: mass * 0.5, // Simplified moment of inertia
            angular_velocity: GA3::zero(),
            angular_acceleration: GA3::zero(),
            friction: 0.1,
            elasticity: 0.7,
            is_kinematic: false,
        }
    }

    /// Update physics state with Verlet integration
    pub fn update(&mut self, dt: UITime) {
        if self.is_kinematic {
            return; // Kinematic objects don't respond to forces
        }

        // Velocity Verlet integration
        let old_position = self.position.multivector();
        let old_velocity = self.velocity.clone();

        // Update position: x = x + v*dt + 0.5*a*dt²
        let position_delta = &old_velocity * dt + &self.acceleration * (0.5 * dt * dt);
        let new_position = &old_position + &position_delta;
        self.position.set(new_position);

        // Update velocity: v = v + a*dt
        let accel_delta = &self.acceleration * dt;
        self.velocity = &old_velocity + &accel_delta;

        // Apply friction to velocity
        self.velocity = &self.velocity * (1.0 - self.friction * dt);

        // Update angular motion
        let angular_accel_delta = &self.angular_acceleration * dt;
        self.angular_velocity = &self.angular_velocity + &angular_accel_delta;
        self.angular_velocity = &self.angular_velocity * (1.0 - self.friction * dt);

        // Reset acceleration (will be recalculated by forces)
        self.acceleration = GA3::zero();
        self.angular_acceleration = GA3::zero();
    }

    /// Apply a force to the cell
    pub fn apply_force(&mut self, force: GA3) {
        if !self.is_kinematic {
            let force_accel = &force * (1.0 / self.mass);
            self.acceleration = &self.acceleration + &force_accel;
        }
    }

    /// Apply a torque to the cell
    pub fn apply_torque(&mut self, torque: GA3) {
        if !self.is_kinematic {
            let torque_accel = &torque * (1.0 / self.moment_of_inertia);
            self.angular_acceleration = &self.angular_acceleration + &torque_accel;
        }
    }

    /// Get the kinetic energy of the cell
    pub fn kinetic_energy(&self) -> f64 {
        let linear_ke = 0.5 * self.mass * self.velocity.norm_squared();
        let angular_ke = 0.5 * self.moment_of_inertia * self.angular_velocity.norm_squared();
        linear_ke + angular_ke
    }

    /// Get the current position as a 3D vector
    pub fn position_vector(&self) -> GA3 {
        self.position.multivector()
    }

    /// Set position directly (for teleportation, initial placement, etc.)
    pub fn set_position(&mut self, position: GA3) {
        self.position.set(position);
    }

    /// Set velocity directly
    pub fn set_velocity(&mut self, velocity: GA3) {
        self.velocity = velocity;
    }

    /// Make the cell kinematic (not affected by forces)
    pub fn set_kinematic(&mut self, kinematic: bool) {
        self.is_kinematic = kinematic;
        if kinematic {
            self.velocity = GA3::zero();
            self.acceleration = GA3::zero();
            self.angular_velocity = GA3::zero();
            self.angular_acceleration = GA3::zero();
        }
    }
}

/// A physics force acting on cells
#[derive(Debug, Clone)]
pub struct Force {
    pub force_type: ForceType,
    pub strength: f64,
    pub direction: GA3,
    pub range: f64,
    pub is_active: bool,
}

impl Force {
    /// Create a new force
    pub fn new(force_type: ForceType, strength: f64, direction: GA3) -> Self {
        Self {
            force_type,
            strength,
            direction: direction.normalize().unwrap_or_else(Multivector::zero),
            range: force_type.range(),
            is_active: true,
        }
    }

    /// Calculate the force applied to a cell at a given position
    pub fn calculate_force_at(&self, position: &GA3, cell_physics: &CellPhysics) -> GA3 {
        if !self.is_active {
            return GA3::zero();
        }

        match self.force_type {
            ForceType::Gravity => self.calculate_gravity(position, cell_physics),
            ForceType::Electromagnetic => self.calculate_electromagnetic(position, cell_physics),
            ForceType::Spring => self.calculate_spring(position, cell_physics),
            ForceType::Damping => self.calculate_damping(position, cell_physics),
            ForceType::UserInteraction => self.calculate_user_interaction(position, cell_physics),
            ForceType::Thermal => self.calculate_thermal(position, cell_physics),
            ForceType::Pressure => self.calculate_pressure(position, cell_physics),
            ForceType::Alignment => self.calculate_alignment(position, cell_physics),
        }
    }

    fn calculate_gravity(&self, _position: &GA3, physics: &CellPhysics) -> GA3 {
        // Simple gravitational force toward direction
        &self.direction * (self.strength * physics.mass)
    }

    fn calculate_electromagnetic(&self, _position: &GA3, physics: &CellPhysics) -> GA3 {
        // Force based on charge and electromagnetic field
        &self.direction * (self.strength * physics.charge)
    }

    fn calculate_spring(&self, position: &GA3, _physics: &CellPhysics) -> GA3 {
        // Spring force: F = -k * displacement
        let displacement = position - &self.direction; // direction as rest position
        &displacement * (-self.strength)
    }

    fn calculate_damping(&self, _position: &GA3, physics: &CellPhysics) -> GA3 {
        // Damping force: F = -b * velocity
        &physics.velocity * (-self.strength)
    }

    fn calculate_user_interaction(&self, _position: &GA3, _physics: &CellPhysics) -> GA3 {
        // Direct force application
        &self.direction * self.strength
    }

    fn calculate_thermal(&self, _position: &GA3, _physics: &CellPhysics) -> GA3 {
        // Random thermal motion
        use amari_core::Vector;
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let thermal_vec = Vector::<3, 0, 0>::from_components(
            rng.gen_range(-1.0..=1.0),
            rng.gen_range(-1.0..=1.0),
            rng.gen_range(-1.0..=1.0),
        );
        &GA3::from_vector(&thermal_vec) * self.strength
    }

    fn calculate_pressure(&self, _position: &GA3, _physics: &CellPhysics) -> GA3 {
        // Pressure force away from crowded areas
        &self.direction * self.strength
    }

    fn calculate_alignment(&self, _position: &GA3, physics: &CellPhysics) -> GA3 {
        // Force to align velocity with neighbors
        let velocity_diff = &self.direction - &physics.velocity;
        &velocity_diff * self.strength
    }
}

/// Manages physics simulation for the UI organism
#[derive(Debug)]
pub struct PhysicsEngine {
    /// Physics properties for each cell
    cell_physics: HashMap<Uuid, CellPhysics>,

    /// Global forces affecting all cells
    global_forces: Vec<Force>,

    /// Local forces between specific cells
    local_forces: HashMap<(Uuid, Uuid), Force>,

    /// Physics configuration
    config: PhysicsConfig,

    /// Spatial grid for efficient collision detection
    spatial_grid: SpatialGrid,
}

/// Configuration for physics behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// Global gravity strength
    pub gravity: f64,

    /// Global damping coefficient
    pub damping: f64,

    /// Maximum velocity magnitude
    pub max_velocity: f64,

    /// Maximum acceleration magnitude
    pub max_acceleration: f64,

    /// Collision detection enabled
    pub enable_collisions: bool,

    /// Connection spring strength
    pub connection_spring_strength: f64,

    /// Thermal motion strength
    pub thermal_strength: f64,

    /// Time step for physics integration
    pub time_step: UITime,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: 0.1,
            damping: 0.1,
            max_velocity: 10.0,
            max_acceleration: 50.0,
            enable_collisions: true,
            connection_spring_strength: 2.0,
            thermal_strength: 0.05,
            time_step: 0.016, // ~60 FPS
        }
    }
}

/// Spatial grid for efficient collision detection and neighbor finding
#[derive(Debug)]
pub struct SpatialGrid {
    grid: HashMap<(i32, i32), Vec<Uuid>>,
    cell_size: f64,
}

impl SpatialGrid {
    fn new(cell_size: f64) -> Self {
        Self {
            grid: HashMap::new(),
            cell_size,
        }
    }

    fn clear(&mut self) {
        self.grid.clear();
    }

    fn insert(&mut self, id: Uuid, position: &GA3) {
        let grid_pos = self.world_to_grid(position);
        self.grid.entry(grid_pos).or_default().push(id);
    }

    fn get_neighbors(&self, position: &GA3, radius: f64) -> Vec<Uuid> {
        let mut neighbors = Vec::new();
        let center_grid = self.world_to_grid(position);
        let grid_radius = (radius / self.cell_size).ceil() as i32;

        for dx in -grid_radius..=grid_radius {
            for dy in -grid_radius..=grid_radius {
                let grid_pos = (center_grid.0 + dx, center_grid.1 + dy);
                if let Some(cell_list) = self.grid.get(&grid_pos) {
                    neighbors.extend(cell_list.iter());
                }
            }
        }

        neighbors
    }

    fn world_to_grid(&self, position: &GA3) -> (i32, i32) {
        // Access vector components directly from the multivector
        let x = (position.vector_component(0) / self.cell_size).floor() as i32;
        let y = (position.vector_component(1) / self.cell_size).floor() as i32;
        (x, y)
    }
}

impl PhysicsEngine {
    /// Create a new physics engine
    pub fn new(config: PhysicsConfig) -> Self {
        Self {
            cell_physics: HashMap::new(),
            global_forces: Vec::new(),
            local_forces: HashMap::new(),
            config,
            spatial_grid: SpatialGrid::new(5.0), // 5-unit grid cells
        }
    }

    /// Create a physics engine with default configuration
    pub fn with_default_config() -> Self {
        Self::new(PhysicsConfig::default())
    }

    /// Add a cell to the physics simulation
    pub fn add_cell(&mut self, cell: &UICell) {
        let position = cell.nucleus().multivector();
        let physics = CellPhysics::new(position, cell.cell_type());
        self.cell_physics.insert(cell.id(), physics);
    }

    /// Remove a cell from the physics simulation
    pub fn remove_cell(&mut self, cell_id: &Uuid) {
        self.cell_physics.remove(cell_id);

        // Remove any local forces involving this cell
        let keys_to_remove: Vec<_> = self
            .local_forces
            .keys()
            .filter(|(a, b)| a == cell_id || b == cell_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.local_forces.remove(&key);
        }
    }

    /// Add a global force affecting all cells
    pub fn add_global_force(&mut self, force: Force) {
        self.global_forces.push(force);
    }

    /// Add a local force between two specific cells
    pub fn add_local_force(&mut self, cell_a: Uuid, cell_b: Uuid, force: Force) {
        self.local_forces.insert((cell_a, cell_b), force);
    }

    /// Remove a global force
    pub fn remove_global_force(&mut self, index: usize) {
        if index < self.global_forces.len() {
            self.global_forces.remove(index);
        }
    }

    /// Step the physics simulation forward
    pub fn step(&mut self, dt: UITime) {
        // Update spatial grid
        self.update_spatial_grid();

        // Calculate forces
        self.calculate_forces(dt);

        // Apply connection forces
        self.apply_connection_forces(dt);

        // Update physics
        self.update_physics(dt);

        // Handle collisions
        if self.config.enable_collisions {
            self.handle_collisions();
        }

        // Enforce limits
        self.enforce_limits();
    }

    /// Update the spatial grid with current cell positions
    fn update_spatial_grid(&mut self) {
        self.spatial_grid.clear();

        for (id, physics) in &self.cell_physics {
            let position = physics.position_vector();
            self.spatial_grid.insert(*id, &position);
        }
    }

    /// Calculate and apply forces to all cells
    fn calculate_forces(&mut self, _dt: UITime) {
        let cell_positions: HashMap<Uuid, GA3> = self
            .cell_physics
            .iter()
            .map(|(id, physics)| (*id, physics.position_vector()))
            .collect();

        // Apply global forces
        for (id, physics) in &mut self.cell_physics {
            for force in &self.global_forces {
                if let Some(position) = cell_positions.get(id) {
                    let force_vector = force.calculate_force_at(position, physics);
                    physics.apply_force(force_vector);
                }
            }
        }

        // Apply local forces
        for ((id_a, id_b), force) in &self.local_forces {
            if let (Some(pos_a), Some(pos_b)) = (cell_positions.get(id_a), cell_positions.get(id_b))
            {
                let distance_vector = pos_b.clone() - pos_a.clone();
                let distance = distance_vector.magnitude();

                if distance <= force.range && distance > 0.0 {
                    let force_vector =
                        force.calculate_force_at(pos_a, self.cell_physics.get(id_a).unwrap());

                    // Apply force to both cells (Newton's third law)
                    if let Some(physics_a) = self.cell_physics.get_mut(id_a) {
                        physics_a.apply_force(force_vector.clone());
                    }
                    if let Some(physics_b) = self.cell_physics.get_mut(id_b) {
                        physics_b.apply_force(&force_vector * -1.0);
                    }
                }
            }
        }
    }

    /// Apply spring forces for connected cells
    fn apply_connection_forces(&mut self, _dt: UITime) {
        // This would iterate through cell connections and apply spring forces
        // For now, we'll implement a simplified version

        let cell_positions: HashMap<Uuid, GA3> = self
            .cell_physics
            .iter()
            .map(|(id, physics)| (*id, physics.position_vector()))
            .collect();

        // Find nearby cells and apply weak attraction
        for (id, physics) in &mut self.cell_physics {
            let position = physics.position_vector();
            let neighbors = self.spatial_grid.get_neighbors(&position, 3.0);

            for neighbor_id in neighbors {
                if neighbor_id != *id {
                    if let Some(neighbor_pos) = cell_positions.get(&neighbor_id) {
                        let distance_vector = neighbor_pos.clone() - position.clone();
                        let distance = distance_vector.magnitude();

                        if distance > 0.0 && distance < 3.0 {
                            let spring_constant =
                                self.config.connection_spring_strength * (3.0 - distance) / 3.0;
                            let spring_force = distance_vector
                                .normalize()
                                .unwrap_or_else(Multivector::zero)
                                * spring_constant;

                            physics.apply_force(spring_force);
                        }
                    }
                }
            }
        }
    }

    /// Update physics state for all cells
    fn update_physics(&mut self, dt: UITime) {
        for physics in self.cell_physics.values_mut() {
            physics.update(dt);
        }
    }

    /// Handle collisions between cells
    fn handle_collisions(&mut self) {
        let cell_ids: Vec<Uuid> = self.cell_physics.keys().cloned().collect();

        for i in 0..cell_ids.len() {
            for j in (i + 1)..cell_ids.len() {
                let id_a = cell_ids[i];
                let id_b = cell_ids[j];

                if let (Some(physics_a), Some(physics_b)) =
                    (self.cell_physics.get(&id_a), self.cell_physics.get(&id_b))
                {
                    let pos_a = physics_a.position_vector();
                    let pos_b = physics_b.position_vector();
                    let distance = (pos_b - pos_a).magnitude();

                    // Collision threshold (simplified as unit radius)
                    let collision_distance = 1.0;

                    if distance < collision_distance && distance > 0.0 {
                        self.resolve_collision(id_a, id_b, distance, collision_distance);
                    }
                }
            }
        }
    }

    /// Resolve collision between two cells
    fn resolve_collision(
        &mut self,
        id_a: Uuid,
        id_b: Uuid,
        distance: f64,
        collision_distance: f64,
    ) {
        let (pos_a, pos_b, vel_a, vel_b, mass_a, mass_b, elasticity_a, elasticity_b) = {
            let physics_a = self.cell_physics.get(&id_a).unwrap();
            let physics_b = self.cell_physics.get(&id_b).unwrap();

            (
                physics_a.position_vector(),
                physics_b.position_vector(),
                physics_a.velocity.clone(),
                physics_b.velocity.clone(),
                physics_a.mass,
                physics_b.mass,
                physics_a.elasticity,
                physics_b.elasticity,
            )
        };

        // Collision normal
        let normal = (pos_b - pos_a)
            .normalize()
            .unwrap_or_else(Multivector::zero);

        // Separate objects
        let overlap = collision_distance - distance;
        let separation = &normal * (overlap * 0.5);

        // Update positions
        if let Some(physics_a) = self.cell_physics.get_mut(&id_a) {
            let new_pos = physics_a.position_vector() - separation.clone();
            physics_a.set_position(new_pos);
        }
        if let Some(physics_b) = self.cell_physics.get_mut(&id_b) {
            let new_pos = physics_b.position_vector() + separation;
            physics_b.set_position(new_pos);
        }

        // Calculate collision response velocities
        let relative_velocity = &vel_b - &vel_a;
        let velocity_along_normal = relative_velocity.scalar_product(&normal);

        if velocity_along_normal > 0.0 {
            return; // Objects separating
        }

        let elasticity = (elasticity_a + elasticity_b) * 0.5;
        let impulse_magnitude =
            -(1.0 + elasticity) * velocity_along_normal / (1.0 / mass_a + 1.0 / mass_b);
        let impulse = &normal * impulse_magnitude;

        // Apply impulse
        if let Some(physics_a) = self.cell_physics.get_mut(&id_a) {
            let impulse_a = &impulse * (1.0 / mass_a);
            physics_a.set_velocity(&vel_a - &impulse_a);
        }
        if let Some(physics_b) = self.cell_physics.get_mut(&id_b) {
            let impulse_b = &impulse * (1.0 / mass_b);
            physics_b.set_velocity(&vel_b + &impulse_b);
        }
    }

    /// Enforce velocity and acceleration limits
    fn enforce_limits(&mut self) {
        for physics in self.cell_physics.values_mut() {
            // Limit velocity
            let velocity_magnitude = physics.velocity.magnitude();
            if velocity_magnitude > self.config.max_velocity {
                physics.velocity = physics
                    .velocity
                    .normalize()
                    .unwrap_or_else(Multivector::zero)
                    * self.config.max_velocity;
            }

            // Limit acceleration
            let acceleration_magnitude = physics.acceleration.magnitude();
            if acceleration_magnitude > self.config.max_acceleration {
                physics.acceleration = physics
                    .acceleration
                    .normalize()
                    .unwrap_or_else(Multivector::zero)
                    * self.config.max_acceleration;
            }
        }
    }

    /// Get physics properties for a cell
    pub fn get_physics(&self, cell_id: &Uuid) -> Option<&CellPhysics> {
        self.cell_physics.get(cell_id)
    }

    /// Get mutable physics properties for a cell
    pub fn get_physics_mut(&mut self, cell_id: &Uuid) -> Option<&mut CellPhysics> {
        self.cell_physics.get_mut(cell_id)
    }

    /// Apply an impulse to a cell
    pub fn apply_impulse(&mut self, cell_id: &Uuid, impulse: GA3) {
        if let Some(physics) = self.cell_physics.get_mut(cell_id) {
            let velocity_change = &impulse * (1.0 / physics.mass);
            physics.set_velocity(&physics.velocity + &velocity_change);
        }
    }

    /// Get total kinetic energy in the system
    pub fn total_kinetic_energy(&self) -> f64 {
        self.cell_physics
            .values()
            .map(|physics| physics.kinetic_energy())
            .sum()
    }

    /// Update physics configuration
    pub fn set_config(&mut self, config: PhysicsConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn config(&self) -> &PhysicsConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_cell::{UICell, UICellType};
    use amari_core::Vector;

    /// Helper function to create a 3D vector as a GA3 multivector
    fn vector3(x: f64, y: f64, z: f64) -> GA3 {
        let v = Vector::<3, 0, 0>::from_components(x, y, z);
        GA3::from_vector(&v)
    }

    #[test]
    fn test_cell_physics_creation() {
        let position = vector3(1.0, 2.0, 0.0);
        let physics = CellPhysics::new(position, UICellType::ButtonCore);

        assert_eq!(physics.mass, 2.0);
        assert_eq!(physics.charge, 1.0);
        assert!(!physics.is_kinematic);
    }

    #[test]
    fn test_force_calculation() {
        let force = Force::new(ForceType::Gravity, 1.0, vector3(0.0, -1.0, 0.0));
        let position = vector3(0.0, 0.0, 0.0);
        let physics = CellPhysics::new(position.clone(), UICellType::ButtonCore);

        let calculated_force = force.calculate_force_at(&position, &physics);
        assert!(calculated_force.magnitude() > 0.0);
    }

    #[test]
    fn test_physics_engine() {
        let mut engine = PhysicsEngine::with_default_config();
        let position = vector3(0.0, 0.0, 0.0);
        let cell = UICell::new_at_position(UICellType::ButtonCore, position);

        engine.add_cell(&cell);
        assert!(engine.get_physics(&cell.id()).is_some());

        engine.step(0.016);
        // Should complete without errors
    }

    #[test]
    fn test_force_types() {
        assert_eq!(ForceType::Gravity.base_strength(), 0.1);
        assert_eq!(ForceType::UserInteraction.base_strength(), 5.0);
        assert!(ForceType::Gravity.range() > ForceType::Spring.range());
    }

    #[test]
    fn test_physics_update() {
        let position = vector3(0.0, 0.0, 0.0);
        let mut physics = CellPhysics::new(position, UICellType::ButtonCore);

        // Apply a force
        let force = vector3(1.0, 0.0, 0.0);
        physics.apply_force(force);

        // Update physics
        physics.update(0.1);

        // Should have moved in the positive x direction
        let new_position = physics.position_vector();
        let x_component = new_position.vector_component(0);
        assert!(x_component > 0.0);
    }

    #[test]
    fn test_kinetic_energy() {
        let position = vector3(0.0, 0.0, 0.0);
        let mut physics = CellPhysics::new(position, UICellType::ButtonCore);

        physics.set_velocity(vector3(2.0, 0.0, 0.0));
        let ke = physics.kinetic_energy();

        // KE = 0.5 * m * v²
        let expected_ke = 0.5 * physics.mass * 4.0; // v² = 4
        assert!((ke - expected_ke).abs() < 1e-10);
    }
}
