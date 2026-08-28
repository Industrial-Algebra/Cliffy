//! Rendering system for living UI organisms
//!
//! This module handles the rendering of UI organisms to various output targets:
//! - `DOMRenderer` — web-sys DOM manipulation (wasm32 only)
//! - `CanvasRenderer` — web-sys Canvas2D drawing (wasm32 only)
//! - `TestRenderer` — records render calls for native testing

use crate::{ui_cell::UICell, UICoordinates};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Error types for rendering operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderError {
    /// Failed to access DOM
    DomAccessError(String),

    /// Failed to create element
    ElementCreationError(String),

    /// Invalid coordinates
    InvalidCoordinates { x: usize, y: usize },

    /// Rendering backend not available
    BackendUnavailable(String),
}

/// Trait for rendering UI organisms
pub trait UIRenderer: Send + Sync {
    /// Render a single UI cell
    fn render_cell(&self, cell: &UICell, position: UICoordinates) -> Result<(), RenderError>;

    /// Clear the rendering surface
    fn clear(&self) -> Result<(), RenderError>;

    /// Get the renderer type name
    fn renderer_type(&self) -> &str;
}

// ============================================================================
// DOM Renderer
// ============================================================================

/// DOM-based renderer for web browsers
#[derive(Debug, Clone)]
pub struct DOMRenderer {
    #[allow(dead_code)]
    container_id: String,
}

impl DOMRenderer {
    /// Create a new DOM renderer
    pub fn new() -> Self {
        Self {
            container_id: "cliffy-alive-container".to_string(),
        }
    }

    /// Create a DOM renderer with a specific container ID
    pub fn with_container(container_id: String) -> Self {
        Self { container_id }
    }
}

impl Default for DOMRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl UIRenderer for DOMRenderer {
    fn render_cell(&self, cell: &UICell, position: UICoordinates) -> Result<(), RenderError> {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            let window =
                web_sys::window().ok_or_else(|| RenderError::DomAccessError("No window".into()))?;
            let document = window
                .document()
                .ok_or_else(|| RenderError::DomAccessError("No document".into()))?;
            let container = document
                .get_element_by_id(&self.container_id)
                .ok_or_else(|| {
                    RenderError::DomAccessError(format!(
                        "Container '{}' not found",
                        self.container_id
                    ))
                })?;

            let elem = document
                .create_element("div")
                .map_err(|e| RenderError::ElementCreationError(format!("{:?}", e)))?;

            let html_elem = elem
                .dyn_ref::<web_sys::HtmlElement>()
                .ok_or_else(|| RenderError::ElementCreationError("Not an HtmlElement".into()))?;

            let state = cell.nucleus().multivector();
            let css = cell.to_css_string(&state);
            html_elem.style().set_css_text(&css);

            elem.set_class_name(&format!("alive-cell alive-{:?}", cell.cell_type()));
            elem.set_id(&format!("cell-{}", cell.id()));

            container
                .append_child(&elem)
                .map_err(|e| RenderError::ElementCreationError(format!("{:?}", e)))?;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (cell, position);
        }

        Ok(())
    }

    fn clear(&self) -> Result<(), RenderError> {
        #[cfg(target_arch = "wasm32")]
        {
            let window =
                web_sys::window().ok_or_else(|| RenderError::DomAccessError("No window".into()))?;
            let document = window
                .document()
                .ok_or_else(|| RenderError::DomAccessError("No document".into()))?;
            if let Some(container) = document.get_element_by_id(&self.container_id) {
                container.set_inner_html("");
            }
        }

        Ok(())
    }

    fn renderer_type(&self) -> &str {
        "DOM"
    }
}

// ============================================================================
// Canvas Renderer
// ============================================================================

/// Canvas-based renderer for 2D graphics
///
/// On wasm32, uses `web-sys` `CanvasRenderingContext2d` APIs.
/// On native, acts as a no-op (use `TestRenderer` for testing).
#[derive(Debug, Clone)]
pub struct CanvasRenderer {
    #[allow(dead_code)]
    canvas_id: String,
    #[allow(dead_code)]
    width: u32,
    #[allow(dead_code)]
    height: u32,
    #[allow(dead_code)]
    cell_size: f64,
}

impl CanvasRenderer {
    /// Create a new canvas renderer
    pub fn new(canvas_id: String, width: u32, height: u32) -> Self {
        Self {
            canvas_id,
            width,
            height,
            cell_size: 10.0,
        }
    }

    /// Set the cell rendering size in pixels
    pub fn with_cell_size(mut self, size: f64) -> Self {
        self.cell_size = size;
        self
    }

    /// Get the canvas context (wasm32 only)
    #[cfg(target_arch = "wasm32")]
    fn get_context(&self) -> Result<web_sys::CanvasRenderingContext2d, RenderError> {
        use wasm_bindgen::JsCast;
        let window =
            web_sys::window().ok_or_else(|| RenderError::DomAccessError("No window".into()))?;
        let document = window
            .document()
            .ok_or_else(|| RenderError::DomAccessError("No document".into()))?;
        let canvas = document.get_element_by_id(&self.canvas_id).ok_or_else(|| {
            RenderError::DomAccessError(format!("Canvas '{}' not found", self.canvas_id))
        })?;
        let canvas = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| RenderError::DomAccessError("Element is not a canvas".into()))?;
        let ctx = canvas
            .get_context("2d")
            .map_err(|e| RenderError::BackendUnavailable(format!("{:?}", e)))?
            .ok_or_else(|| RenderError::BackendUnavailable("No 2d context".into()))?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .map_err(|_| {
                RenderError::BackendUnavailable("Not a CanvasRenderingContext2d".into())
            })?;
        Ok(ctx)
    }
}

impl UIRenderer for CanvasRenderer {
    fn render_cell(&self, cell: &UICell, position: UICoordinates) -> Result<(), RenderError> {
        #[cfg(target_arch = "wasm32")]
        {
            let ctx = self.get_context()?;

            let vis = cell.visual_properties();
            let (x, y) = position;
            let px = x as f64 * self.cell_size;
            let py = y as f64 * self.cell_size;
            let size = self.cell_size * vis.size;

            // Fill color
            let r = (vis.color[0] * 255.0) as u8;
            let g = (vis.color[1] * 255.0) as u8;
            let b = (vis.color[2] * 255.0) as u8;
            let fill = format!("rgba({},{},{},{})", r, g, b, vis.opacity);

            ctx.set_global_alpha(vis.opacity);
            ctx.set_fill_style_str(&fill);

            // Glow effect via shadow
            if vis.glow_intensity > 0.01 {
                ctx.set_shadow_blur(vis.glow_intensity * 20.0);
                ctx.set_shadow_color(&format!("rgba({},{},{},0.6)", r, g, b));
            } else {
                ctx.set_shadow_blur(0.0);
            }

            ctx.fill_rect(px, py, size, size);

            // Border
            if vis.border_width > 0.0 {
                let br = (vis.border_color[0] * 255.0) as u8;
                let bg = (vis.border_color[1] * 255.0) as u8;
                let bb = (vis.border_color[2] * 255.0) as u8;
                ctx.set_stroke_style_str(&format!("rgb({},{},{})", br, bg, bb));
                ctx.set_line_width(vis.border_width);
                ctx.stroke_rect(px, py, size, size);
            }

            // Reset
            ctx.set_global_alpha(1.0);
            ctx.set_shadow_blur(0.0);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (cell, position);
        }

        Ok(())
    }

    fn clear(&self) -> Result<(), RenderError> {
        #[cfg(target_arch = "wasm32")]
        {
            let ctx = self.get_context()?;
            ctx.clear_rect(0.0, 0.0, self.width as f64, self.height as f64);
        }

        Ok(())
    }

    fn renderer_type(&self) -> &str {
        "Canvas"
    }
}

// ============================================================================
// Test Renderer
// ============================================================================

/// A recorded render call for testing
#[derive(Debug, Clone)]
pub struct RenderCall {
    pub cell_type: crate::UICellType,
    pub position: UICoordinates,
    pub opacity: f64,
    pub size: f64,
    pub color: [f64; 4],
    pub glow_intensity: f64,
}

/// A renderer that records all render calls for native testing
///
/// Use this in tests to verify rendering behavior without a browser.
#[derive(Debug, Clone)]
pub struct TestRenderer {
    calls: Arc<Mutex<Vec<RenderCall>>>,
    clear_count: Arc<Mutex<usize>>,
}

impl TestRenderer {
    /// Create a new test renderer
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            clear_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Get all recorded render calls
    pub fn calls(&self) -> Vec<RenderCall> {
        self.calls.lock().unwrap().clone()
    }

    /// Get the number of times clear was called
    pub fn clear_count(&self) -> usize {
        *self.clear_count.lock().unwrap()
    }

    /// Reset all recorded state
    pub fn reset(&self) {
        self.calls.lock().unwrap().clear();
        *self.clear_count.lock().unwrap() = 0;
    }
}

impl Default for TestRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl UIRenderer for TestRenderer {
    fn render_cell(&self, cell: &UICell, position: UICoordinates) -> Result<(), RenderError> {
        let vis = cell.visual_properties();
        self.calls.lock().unwrap().push(RenderCall {
            cell_type: cell.cell_type(),
            position,
            opacity: vis.opacity,
            size: vis.size,
            color: vis.color,
            glow_intensity: vis.glow_intensity,
        });
        Ok(())
    }

    fn clear(&self) -> Result<(), RenderError> {
        *self.clear_count.lock().unwrap() += 1;
        Ok(())
    }

    fn renderer_type(&self) -> &str {
        "Test"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AliveUI, UICellType};

    #[test]
    fn test_test_renderer_records_calls() {
        let renderer = TestRenderer::new();
        let mut ui = AliveUI::new().with_renderer(Box::new(renderer.clone()));

        ui.plant_seed(5, 5, UICellType::ButtonCore).unwrap();
        ui.plant_seed(6, 6, UICellType::InputField).unwrap();
        ui.render().unwrap();

        let calls = renderer.calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].cell_type, UICellType::ButtonCore);
        assert_eq!(calls[0].position, (5, 5));
        assert_eq!(calls[1].cell_type, UICellType::InputField);
        assert_eq!(calls[1].position, (6, 6));
    }

    #[test]
    fn test_test_renderer_clear() {
        let renderer = TestRenderer::new();
        let ui = AliveUI::new().with_renderer(Box::new(renderer.clone()));

        ui.render().unwrap();
        assert_eq!(renderer.clear_count(), 1);

        ui.render().unwrap();
        assert_eq!(renderer.clear_count(), 2);
    }

    #[test]
    fn test_test_renderer_visual_properties() {
        let renderer = TestRenderer::new();
        let mut ui = AliveUI::new().with_renderer(Box::new(renderer.clone()));

        ui.plant_seed(5, 5, UICellType::ButtonCore).unwrap();
        ui.render().unwrap();

        let calls = renderer.calls();
        assert_eq!(calls.len(), 1);

        // ButtonCore has specific visual properties
        let call = &calls[0];
        assert!(call.size > 0.0);
        assert!(call.opacity > 0.0);
        assert!(call.glow_intensity >= 0.0);
    }

    #[test]
    fn test_test_renderer_reset() {
        let renderer = TestRenderer::new();
        let mut ui = AliveUI::new().with_renderer(Box::new(renderer.clone()));

        ui.plant_seed(5, 5, UICellType::ButtonCore).unwrap();
        ui.render().unwrap();
        assert_eq!(renderer.calls().len(), 1);
        assert_eq!(renderer.clear_count(), 1);

        renderer.reset();
        assert_eq!(renderer.calls().len(), 0);
        assert_eq!(renderer.clear_count(), 0);
    }

    #[test]
    fn test_dom_renderer_native_noop() {
        // On native, DOMRenderer should be a no-op
        let renderer = DOMRenderer::new();
        assert_eq!(renderer.renderer_type(), "DOM");
        assert!(renderer.clear().is_ok());
    }

    #[test]
    fn test_canvas_renderer_native_noop() {
        // On native, CanvasRenderer should be a no-op
        let renderer = CanvasRenderer::new("test".into(), 800, 600);
        assert_eq!(renderer.renderer_type(), "Canvas");
        assert!(renderer.clear().is_ok());
    }
}
