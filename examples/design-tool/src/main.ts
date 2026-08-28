/**
 * Cliffy Design Tool Example
 *
 * Demonstrates:
 * - Shape manipulation via rotors
 * - Transform composition (rotation + translation + scale)
 * - Undo/redo using Behavior history patterns
 * - Real-time property binding
 * - Event-driven interactions
 */

import init, {
  behavior,
  event,
  combine,
  Rotor,
  Translation,
  Transform,
} from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';

// ============================================================================
// Types
// ============================================================================

type Tool = 'select' | 'rect' | 'circle' | 'triangle' | 'move';

interface Shape {
  id: string;
  type: 'rect' | 'circle' | 'triangle';
  x: number;
  y: number;
  width: number;
  height: number;
  rotation: number; // degrees
  fill: string;
}

interface HistoryEntry {
  shapes: Shape[];
  selectedId: string | null;
}

// ============================================================================
// Main
// ============================================================================

async function main() {
  await init();

  // ==========================================================================
  // State Behaviors
  // ==========================================================================

  const shapes = behavior<Shape[]>([]);
  const selectedId = behavior<string | null>(null);
  const currentTool = behavior<Tool>('select');

  // History for undo/redo
  const history = behavior<HistoryEntry[]>([{ shapes: [], selectedId: null }]);
  const historyIndex = behavior(0);

  // Derived: selected shape
  const selectedShape = combine(shapes, selectedId, (allShapes, id) => {
    return id ? allShapes.find(s => s.id === id) || null : null;
  });

  // Derived: can undo/redo
  const canUndo = historyIndex.map(idx => idx > 0);
  const canRedo = combine(history, historyIndex, (h, idx) => idx < h.length - 1);

  // ==========================================================================
  // Events
  // ==========================================================================

  const canvasClick = event<{ x: number; y: number; target: HTMLElement }>();
  const canvasDrag = event<{ startX: number; startY: number; x: number; y: number }>();
  const canvasDragEnd = event<void>();

  // Drag state
  let isDragging = false;
  let dragStartX = 0;
  let dragStartY = 0;
  let dragShapeStartX = 0;
  let dragShapeStartY = 0;

  // ==========================================================================
  // History Management
  // ==========================================================================

  function pushHistory() {
    const currentShapes = shapes.sample() as Shape[];
    const currentSelected = selectedId.sample() as string | null;
    const currentHistory = history.sample() as HistoryEntry[];
    const currentIndex = historyIndex.sample() as number;

    // Truncate forward history if we're not at the end
    const newHistory = currentHistory.slice(0, currentIndex + 1);
    newHistory.push({
      shapes: JSON.parse(JSON.stringify(currentShapes)),
      selectedId: currentSelected,
    });

    // Limit history size
    if (newHistory.length > 50) {
      newHistory.shift();
    } else {
      historyIndex.set(currentIndex + 1);
    }

    history.set(newHistory);
  }

  function undo() {
    const currentIndex = historyIndex.sample() as number;
    if (currentIndex > 0) {
      historyIndex.set(currentIndex - 1);
      const entry = (history.sample() as HistoryEntry[])[currentIndex - 1];
      shapes.set(JSON.parse(JSON.stringify(entry.shapes)));
      selectedId.set(entry.selectedId);
    }
  }

  function redo() {
    const currentHistory = history.sample() as HistoryEntry[];
    const currentIndex = historyIndex.sample() as number;
    if (currentIndex < currentHistory.length - 1) {
      historyIndex.set(currentIndex + 1);
      const entry = currentHistory[currentIndex + 1];
      shapes.set(JSON.parse(JSON.stringify(entry.shapes)));
      selectedId.set(entry.selectedId);
    }
  }

  // ==========================================================================
  // Shape Operations
  // ==========================================================================

  function generateId(): string {
    return `shape-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  function createShape(type: Shape['type'], x: number, y: number): Shape {
    const fills: Record<Shape['type'], string> = {
      rect: '#7c3aed',
      circle: '#00d4ff',
      triangle: '#f59e0b',
    };

    return {
      id: generateId(),
      type,
      x,
      y,
      width: 80,
      height: 80,
      rotation: 0,
      fill: fills[type],
    };
  }

  function updateShape(id: string, updates: Partial<Shape>) {
    const currentShapes = shapes.sample() as Shape[];
    shapes.set(
      currentShapes.map(s => (s.id === id ? { ...s, ...updates } : s))
    );
  }

  function deleteSelected() {
    const id = selectedId.sample() as string | null;
    if (id) {
      const currentShapes = shapes.sample() as Shape[];
      shapes.set(currentShapes.filter(s => s.id !== id));
      selectedId.set(null);
      pushHistory();
    }
  }

  // ==========================================================================
  // Transform Helpers (GA-based)
  // ==========================================================================

  function getShapeTransform(shape: Shape): string {
    // Use GA Transform to compose rotation and translation
    const radians = (shape.rotation * Math.PI) / 180;
    const rotor = Rotor.xy(radians);
    const translation = new Translation(shape.x, shape.y, 0);
    const transform = Transform.fromRotorAndTranslation(rotor, translation);

    // For CSS, we need translate then rotate
    return `translate(${shape.x}px, ${shape.y}px) rotate(${shape.rotation}deg)`;
  }

  function rotateShape(shape: Shape, deltaAngle: number): Shape {
    // Use rotor composition for rotation
    const currentRadians = (shape.rotation * Math.PI) / 180;
    const deltaRadians = (deltaAngle * Math.PI) / 180;

    const currentRotor = Rotor.xy(currentRadians);
    const deltaRotor = Rotor.xy(deltaRadians);

    // Compose rotors
    const newRotor = deltaRotor.compose(currentRotor);
    const newAngle = (newRotor.angle() * 180) / Math.PI;

    return { ...shape, rotation: newAngle };
  }

  // ==========================================================================
  // Event Handlers
  // ==========================================================================

  canvasClick.subscribe(({ x, y, target }) => {
    const tool = currentTool.sample() as Tool;

    if (tool === 'select') {
      // Check if clicking on a shape
      const shapeEl = target.closest('.shape') as HTMLElement | null;
      if (shapeEl) {
        selectedId.set(shapeEl.dataset.id || null);
      } else {
        selectedId.set(null);
      }
    } else if (tool === 'rect' || tool === 'circle' || tool === 'triangle') {
      // Create new shape
      const shape = createShape(tool, x - 40, y - 40);
      const currentShapes = shapes.sample() as Shape[];
      shapes.set([...currentShapes, shape]);
      selectedId.set(shape.id);
      currentTool.set('select');
      pushHistory();
    }
  });

  canvasDrag.subscribe(({ startX, startY, x, y }) => {
    const tool = currentTool.sample() as Tool;
    const id = selectedId.sample() as string | null;

    if ((tool === 'select' || tool === 'move') && id) {
      const dx = x - startX;
      const dy = y - startY;
      updateShape(id, {
        x: dragShapeStartX + dx,
        y: dragShapeStartY + dy,
      });
    }
  });

  canvasDragEnd.subscribe(() => {
    if (isDragging) {
      pushHistory();
    }
    isDragging = false;
  });

  // ==========================================================================
  // Canvas Event Setup
  // ==========================================================================

  function setupCanvasEvents(canvas: HTMLElement) {
    canvas.addEventListener('mousedown', (e) => {
      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      const shapeEl = (e.target as HTMLElement).closest('.shape') as HTMLElement | null;

      if (shapeEl) {
        const id = shapeEl.dataset.id;
        selectedId.set(id || null);

        const currentShapes = shapes.sample() as Shape[];
        const shape = currentShapes.find(s => s.id === id);
        if (shape) {
          isDragging = true;
          dragStartX = x;
          dragStartY = y;
          dragShapeStartX = shape.x;
          dragShapeStartY = shape.y;
        }
      } else {
        canvasClick.emit({ x, y, target: e.target as HTMLElement });
      }
    });

    canvas.addEventListener('mousemove', (e) => {
      if (isDragging) {
        const rect = canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        canvasDrag.emit({ startX: dragStartX, startY: dragStartY, x, y });
      }
    });

    canvas.addEventListener('mouseup', () => {
      canvasDragEnd.emit();
    });

    canvas.addEventListener('mouseleave', () => {
      canvasDragEnd.emit();
    });
  }

  // ==========================================================================
  // Export
  // ==========================================================================

  function exportDesign() {
    const currentShapes = shapes.sample() as Shape[];
    const data = JSON.stringify({ shapes: currentShapes }, null, 2);
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'design.json';
    a.click();
    URL.revokeObjectURL(url);
  }

  // ==========================================================================
  // UI
  // ==========================================================================

  const toolButtons: { tool: Tool; icon: string }[] = [
    { tool: 'select', icon: '&#x2794;' },
    { tool: 'rect', icon: '&#x25A1;' },
    { tool: 'circle', icon: '&#x25CB;' },
    { tool: 'triangle', icon: '&#x25B3;' },
    { tool: 'move', icon: '&#x271A;' },
  ];

  const app = html`
    <div class="app-container">
      <!-- Toolbar -->
      <div class="toolbar">
        ${toolButtons.map(
          ({ tool, icon }) => html`
            <button
              class=${currentTool.map((t: Tool) => `tool-btn ${t === tool ? 'active' : ''}`)}
              onclick=${() => currentTool.set(tool)}
              title=${tool}
            >
              <span>${icon}</span>
            </button>
          `
        )}
      </div>

      <div class="main-area">
        <!-- Top Bar -->
        <div class="top-bar">
          <h1>Cliffy Design Tool</h1>
          <span class="spacer"></span>
          <button
            class="action-btn"
            onclick=${undo}
            disabled=${canUndo.map((can: boolean) => !can)}
          >
            Undo
          </button>
          <button
            class="action-btn"
            onclick=${redo}
            disabled=${canRedo.map((can: boolean) => !can)}
          >
            Redo
          </button>
          <button
            class="action-btn"
            onclick=${deleteSelected}
            disabled=${selectedId.map((id: string | null) => !id)}
          >
            Delete
          </button>
          <button class="action-btn primary" onclick=${exportDesign}>
            Export
          </button>
        </div>

        <!-- Canvas -->
        <div class="canvas-area">
          <div
            class=${currentTool.map((t: Tool) => `canvas tool-${t}`)}
            id="canvas"
          >
            ${shapes.map((allShapes: Shape[]) =>
              allShapes.map((shape) => {
                const isSelected = (selectedId.sample() as string | null) === shape.id;
                let shapeClass = `shape shape-${shape.type}`;
                if (isSelected) shapeClass += ' selected';

                const style =
                  shape.type === 'triangle'
                    ? `transform: ${getShapeTransform(shape)}; border-bottom-color: ${shape.fill};`
                    : `transform: ${getShapeTransform(shape)}; width: ${shape.width}px; height: ${shape.height}px; background: ${shape.fill};`;

                return html`
                  <div
                    class=${shapeClass}
                    data-id=${shape.id}
                    style=${style}
                  >
                    ${isSelected
                      ? html`
                          <div class="resize-handle nw"></div>
                          <div class="resize-handle ne"></div>
                          <div class="resize-handle sw"></div>
                          <div class="resize-handle se"></div>
                          <div class="rotate-handle"></div>
                        `
                      : ''}
                  </div>
                `;
              })
            )}
          </div>
        </div>

        <!-- Status Bar -->
        <div class="status-bar">
          <span>Tool: ${currentTool}</span>
          <span class="separator">|</span>
          <span>Shapes: ${shapes.map((s: Shape[]) => s.length)}</span>
          <span class="separator">|</span>
          <span>Selected: ${selectedId.map((id: string | null) => id || 'None')}</span>
        </div>
      </div>

      <!-- Properties Panel -->
      <div class="properties-panel">
        <h3>Properties</h3>

        ${selectedShape.map((shape: Shape | null) =>
          shape
            ? html`
                <div class="property-group">
                  <div class="property-row">
                    <label>Type</label>
                    <input type="text" value=${shape.type} readonly />
                  </div>
                  <div class="property-row">
                    <label>X</label>
                    <input
                      type="number"
                      value=${shape.x}
                      oninput=${(e: Event) => {
                        updateShape(shape.id, {
                          x: Number((e.target as HTMLInputElement).value),
                        });
                      }}
                    />
                  </div>
                  <div class="property-row">
                    <label>Y</label>
                    <input
                      type="number"
                      value=${shape.y}
                      oninput=${(e: Event) => {
                        updateShape(shape.id, {
                          y: Number((e.target as HTMLInputElement).value),
                        });
                      }}
                    />
                  </div>
                  <div class="property-row">
                    <label>Width</label>
                    <input
                      type="number"
                      value=${shape.width}
                      oninput=${(e: Event) => {
                        updateShape(shape.id, {
                          width: Number((e.target as HTMLInputElement).value),
                        });
                      }}
                    />
                  </div>
                  <div class="property-row">
                    <label>Height</label>
                    <input
                      type="number"
                      value=${shape.height}
                      oninput=${(e: Event) => {
                        updateShape(shape.id, {
                          height: Number((e.target as HTMLInputElement).value),
                        });
                      }}
                    />
                  </div>
                  <div class="property-row">
                    <label>Rotation</label>
                    <input
                      type="number"
                      value=${shape.rotation}
                      oninput=${(e: Event) => {
                        updateShape(shape.id, {
                          rotation: Number((e.target as HTMLInputElement).value),
                        });
                      }}
                    />
                  </div>
                  <div class="property-row">
                    <label>Fill</label>
                    <input
                      type="color"
                      value=${shape.fill}
                      oninput=${(e: Event) => {
                        updateShape(shape.id, {
                          fill: (e.target as HTMLInputElement).value,
                        });
                      }}
                    />
                  </div>
                </div>
              `
            : html`<p class="no-selection">Select a shape to edit its properties</p>`
        )}
      </div>
    </div>
  `;

  mount(app, '#app');

  // Setup canvas events after mount
  requestAnimationFrame(() => {
    const canvas = document.getElementById('canvas');
    if (canvas) {
      setupCanvasEvents(canvas);
    }
  });

  // Keyboard shortcuts
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Delete' || e.key === 'Backspace') {
      deleteSelected();
    } else if (e.key === 'z' && (e.ctrlKey || e.metaKey)) {
      if (e.shiftKey) {
        redo();
      } else {
        undo();
      }
      e.preventDefault();
    } else if (e.key === 'Escape') {
      selectedId.set(null);
    }
  });

  console.log('Cliffy Design Tool initialized');
}

main().catch(console.error);
