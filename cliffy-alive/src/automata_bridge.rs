//! Bridge to amari-automata for advanced CA operations
//!
//! This module provides adapters between cliffy-alive's `UIOrganismField` and
//! amari-automata's `UIAssembler` and `InverseCADesigner`, enabling:
//!
//! - **Layout inference**: Use `UIAssembler` for responsive layout computation
//! - **Inverse design**: Use `InverseCADesigner` to find seed patterns that
//!   evolve into target layouts
//!
//! This module is only available with the `automata` feature flag:
//! ```toml
//! cliffy-alive = { version = "0.1", features = ["automata"] }
//! ```

#[cfg(feature = "automata")]
use amari_automata::{
    ComponentType, InverseCADesigner, Layout, UIAssembler, UIAssemblyConfig, UIComponent,
    UIComponentType, Vector,
};

#[cfg(feature = "automata")]
use crate::{ui_organism::UIOrganismField, UICellType};

/// Adapter that converts cliffy-alive UIOrganismField cells to
/// amari-automata UIComponents for layout assembly.
#[cfg(feature = "automata")]
pub struct LayoutAdapter {
    assembler: UIAssembler<3, 0, 0>,
}

#[cfg(feature = "automata")]
impl Default for LayoutAdapter {
    fn default() -> Self {
        Self {
            assembler: UIAssembler::with_default_config(),
        }
    }
}

#[cfg(feature = "automata")]
impl LayoutAdapter {
    /// Create a new layout adapter with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom configuration
    pub fn with_config(config: UIAssemblyConfig) -> Self {
        Self {
            assembler: UIAssembler::new(config),
        }
    }

    /// Set viewport dimensions for responsive layout
    pub fn set_viewport(&mut self, width: f64, height: f64) {
        self.assembler.set_viewport_size(width, height);
    }

    /// Convert organism cells to UIComponents and compute layout
    pub fn compute_layout(&self, organism: &UIOrganismField) -> Result<Vec<LayoutResult>, String> {
        let components: Vec<UIComponent<3, 0, 0>> = organism
            .iter_cells()
            .map(|((x, y), cell)| {
                let signature = amari_automata::Multivector::<3, 0, 0>::basis_vector(0)
                    * (x as f64)
                    + amari_automata::Multivector::<3, 0, 0>::basis_vector(1) * (y as f64);
                let position = Vector::<3, 0, 0>::from_components(x as f64, y as f64, 0.0);
                let component_type = cell_type_to_automata(cell.cell_type());
                let layout = Layout::default();

                UIComponent::new(signature, position, component_type, layout)
            })
            .collect();

        if components.is_empty() {
            return Ok(Vec::new());
        }

        let assembly = self
            .assembler
            .assemble_ui(&components)
            .map_err(|e| format!("Assembly failed: {:?}", e))?;

        let mut results = Vec::new();
        for i in 0..assembly.component_count() {
            if let Ok(rect) = assembly.get_computed_rect(i) {
                let (cx, cy, _) = rect.center();
                results.push(LayoutResult {
                    index: i,
                    x: cx,
                    y: cy,
                    width: rect.width,
                    height: rect.height,
                });
            }
        }

        Ok(results)
    }
}

/// Result of layout computation for a single cell
#[cfg(feature = "automata")]
#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub index: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Adapter for inverse design — finding seed patterns that evolve into target layouts
#[cfg(feature = "automata")]
pub struct InverseDesignAdapter {
    designer: InverseCADesigner,
}

#[cfg(feature = "automata")]
impl Default for InverseDesignAdapter {
    fn default() -> Self {
        Self {
            designer: InverseCADesigner::new(),
        }
    }
}

#[cfg(feature = "automata")]
impl InverseDesignAdapter {
    /// Create a new inverse design adapter
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a reference to the underlying designer
    pub fn designer(&self) -> &InverseCADesigner {
        &self.designer
    }
}

/// Map cliffy-alive cell types to amari-automata component types
#[cfg(feature = "automata")]
fn cell_type_to_automata(cell_type: UICellType) -> ComponentType {
    match cell_type {
        UICellType::ButtonCore | UICellType::ButtonEdge => {
            ComponentType::UIElement(UIComponentType::Button)
        }
        UICellType::InputField => ComponentType::UIElement(UIComponentType::Input),
        UICellType::TextDisplay => ComponentType::UIElement(UIComponentType::Label),
        UICellType::Container => ComponentType::UIElement(UIComponentType::Container),
        UICellType::Spacer => ComponentType::UIElement(UIComponentType::Spacer),
        UICellType::Header => ComponentType::UIElement(UIComponentType::Header),
        UICellType::Navigation => ComponentType::UIElement(UIComponentType::Navigation),
        UICellType::Content => ComponentType::UIElement(UIComponentType::Content),
        UICellType::Connector => ComponentType::Junction,
        UICellType::Decoration => ComponentType::Basic,
        UICellType::Sensor => ComponentType::Basic,
        UICellType::Memory => ComponentType::Basic,
        UICellType::Generic => ComponentType::Basic,
        UICellType::DataDisplay => ComponentType::UIElement(UIComponentType::Panel),
        UICellType::DataVisualization => ComponentType::UIElement(UIComponentType::Panel),
    }
}

#[cfg(all(test, feature = "automata"))]
mod tests {
    use super::*;
    use crate::{AliveConfig, UICellType};

    #[test]
    fn test_layout_adapter_creation() {
        let adapter = LayoutAdapter::new();
        let config = AliveConfig::default();
        let organism = UIOrganismField::new(config.field_dimensions, config);
        let result = adapter.compute_layout(&organism);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_layout_with_cells() {
        let adapter = LayoutAdapter::new();
        let config = AliveConfig::default();
        let mut organism = UIOrganismField::new(config.field_dimensions, config);

        organism.plant_seed(5, 5, UICellType::ButtonCore).unwrap();
        organism.plant_seed(10, 10, UICellType::InputField).unwrap();

        let result = adapter.compute_layout(&organism);
        assert!(result.is_ok());
        let layouts = result.unwrap();
        assert_eq!(layouts.len(), 2);
    }

    #[test]
    fn test_inverse_design_adapter_creation() {
        let adapter = InverseDesignAdapter::new();
        let _ = adapter.designer();
    }

    #[test]
    fn test_cell_type_mapping() {
        assert!(matches!(
            cell_type_to_automata(UICellType::ButtonCore),
            ComponentType::UIElement(UIComponentType::Button)
        ));
        assert!(matches!(
            cell_type_to_automata(UICellType::InputField),
            ComponentType::UIElement(UIComponentType::Input)
        ));
        assert!(matches!(
            cell_type_to_automata(UICellType::Generic),
            ComponentType::Basic
        ));
    }
}
