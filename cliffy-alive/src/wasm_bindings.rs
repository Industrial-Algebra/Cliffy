//! WASM bindings for the Living UI API
//!
//! Exposes `ReactiveAliveUI` to JavaScript via `wasm-bindgen`.
//!
//! # JavaScript Usage
//!
//! ```js
//! import { WasmAliveUI } from 'cliffy-alive';
//!
//! const ui = new WasmAliveUI();
//! const id = ui.plant_seed(10, 10, "ButtonCore");
//! ui.feed_region(10, 10, 5, 50.0);
//!
//! function animate() {
//!     ui.step(1.0 / 60.0);
//!     ui.render();
//!     requestAnimationFrame(animate);
//! }
//! animate();
//! ```

use wasm_bindgen::prelude::*;

use crate::frp_bridge::ReactiveAliveUI;
use crate::{AliveConfig, AliveError, CanvasRenderer, DOMRenderer, UICellType, UIRenderer};

/// Parse a string cell type name from JavaScript into the Rust enum
fn parse_cell_type(name: &str) -> Result<UICellType, JsValue> {
    match name {
        "ButtonCore" => Ok(UICellType::ButtonCore),
        "ButtonEdge" => Ok(UICellType::ButtonEdge),
        "InputField" => Ok(UICellType::InputField),
        "TextDisplay" => Ok(UICellType::TextDisplay),
        "Container" => Ok(UICellType::Container),
        "Spacer" => Ok(UICellType::Spacer),
        "Connector" => Ok(UICellType::Connector),
        "Decoration" => Ok(UICellType::Decoration),
        "Sensor" => Ok(UICellType::Sensor),
        "Memory" => Ok(UICellType::Memory),
        "Generic" => Ok(UICellType::Generic),
        "Header" => Ok(UICellType::Header),
        "Content" => Ok(UICellType::Content),
        "Navigation" => Ok(UICellType::Navigation),
        "DataDisplay" => Ok(UICellType::DataDisplay),
        "DataVisualization" => Ok(UICellType::DataVisualization),
        _ => Err(JsValue::from_str(&format!("Unknown cell type: {}", name))),
    }
}

fn alive_error_to_js(err: AliveError) -> JsValue {
    JsValue::from_str(&err.to_string())
}

/// The main Living UI entry point for JavaScript
#[wasm_bindgen]
pub struct WasmAliveUI {
    inner: ReactiveAliveUI,
}

impl Default for WasmAliveUI {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmAliveUI {
    /// Create a new Living UI with default configuration
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: ReactiveAliveUI::new(),
        }
    }

    /// Create a Living UI with a custom field size
    #[wasm_bindgen(js_name = "withFieldSize")]
    pub fn with_field_size(width: usize, height: usize) -> Self {
        let config = AliveConfig {
            field_dimensions: (width, height),
            ..AliveConfig::default()
        };
        Self {
            inner: ReactiveAliveUI::with_config(config),
        }
    }

    /// Set the renderer to DOM mode
    #[wasm_bindgen(js_name = "useDomRenderer")]
    pub fn use_dom_renderer(&mut self, container_id: &str) {
        let renderer = DOMRenderer::with_container(container_id.to_string());
        self.inner
            .inner_mut()
            .set_renderer(Box::new(renderer) as Box<dyn UIRenderer>);
    }

    /// Set the renderer to Canvas mode
    #[wasm_bindgen(js_name = "useCanvasRenderer")]
    pub fn use_canvas_renderer(&mut self, canvas_id: &str, width: u32, height: u32) {
        let renderer = CanvasRenderer::new(canvas_id.to_string(), width, height);
        self.inner
            .inner_mut()
            .set_renderer(Box::new(renderer) as Box<dyn UIRenderer>);
    }

    /// Plant a seed cell at the given coordinates
    ///
    /// Returns the cell's UUID as a string, or throws on error.
    #[wasm_bindgen(js_name = "plantSeed")]
    pub fn plant_seed(&mut self, x: usize, y: usize, cell_type: &str) -> Result<String, JsValue> {
        let ct = parse_cell_type(cell_type)?;
        let id = self.inner.plant_seed(x, y, ct).map_err(alive_error_to_js)?;
        Ok(id.to_string())
    }

    /// Step the simulation forward by `dt` seconds
    pub fn step(&mut self, dt: f64) {
        self.inner.step(dt);
    }

    /// Render the current state to the configured renderer
    pub fn render(&self) -> Result<(), JsValue> {
        self.inner
            .render()
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    /// Feed energy to a circular region
    #[wasm_bindgen(js_name = "feedRegion")]
    pub fn feed_region(&mut self, x: usize, y: usize, radius: usize, energy: f64) {
        self.inner.feed_region(x, y, radius, energy);
    }

    /// Get statistics as a JavaScript object
    pub fn statistics(&self) -> JsValue {
        let stats = self.inner.statistics();
        serde_wasm_bindgen::to_value(&stats).unwrap_or(JsValue::NULL)
    }

    /// Get the current cell count
    #[wasm_bindgen(js_name = "cellCount")]
    pub fn cell_count(&self) -> usize {
        self.inner.behaviors().cell_count.sample()
    }

    /// Get the current total energy
    #[wasm_bindgen(js_name = "totalEnergy")]
    pub fn total_energy(&self) -> f64 {
        self.inner.behaviors().total_energy.sample()
    }

    /// Get the current simulation time
    pub fn time(&self) -> f64 {
        self.inner.behaviors().time.sample()
    }

    /// Export the organism state as a JSON string
    pub fn export(&self) -> Result<String, JsValue> {
        let snapshot = self.inner.export_organism();
        serde_json::to_string(&snapshot)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Import organism state from a JSON string
    pub fn import(&mut self, json: &str) -> Result<(), JsValue> {
        let snapshot: crate::OrganismSnapshot = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;
        self.inner
            .import_organism(snapshot)
            .map_err(alive_error_to_js)
    }

    /// Apply selection pressure
    #[wasm_bindgen(js_name = "applySelectionPressure")]
    pub fn apply_selection_pressure(&mut self, pressure: &str) -> Result<(), JsValue> {
        let p = match pressure {
            "Responsiveness" => crate::SelectionPressure::Responsiveness,
            "EnergyEfficiency" => crate::SelectionPressure::EnergyEfficiency,
            "Cooperation" => crate::SelectionPressure::Cooperation,
            "VisualAppeal" => crate::SelectionPressure::VisualAppeal,
            "Adaptability" => crate::SelectionPressure::Adaptability,
            _ => {
                return Err(JsValue::from_str(&format!(
                    "Unknown selection pressure: {}",
                    pressure
                )))
            }
        };
        self.inner.apply_selection_pressure(p);
        Ok(())
    }
}
