/**
 * Alive Garden — Full Living UI demo
 *
 * Plant diverse seed types on a canvas, feed energy, apply selection
 * pressures, and watch a living UI ecosystem evolve.
 */

import init, { WasmAliveUI } from 'cliffy-alive';

const GRID_SIZE = 60;
const CANVAS_SIZE = 600;
const CELL_PX = CANVAS_SIZE / GRID_SIZE;

async function main() {
  await init();

  const ui = WasmAliveUI.withFieldSize(GRID_SIZE, GRID_SIZE);
  ui.useCanvasRenderer('garden-canvas', CANVAS_SIZE, CANVAS_SIZE);

  let selectedType = 'ButtonCore';

  // Seed type selection
  const seedButtons = document.querySelectorAll<HTMLButtonElement>('.seed-btn');
  seedButtons.forEach(btn => {
    btn.addEventListener('click', () => {
      seedButtons.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      selectedType = btn.dataset.type!;
    });
  });

  // Canvas click: plant seed + small energy feed
  const canvas = document.getElementById('garden-canvas') as HTMLCanvasElement;
  canvas.addEventListener('click', (e: MouseEvent) => {
    const rect = canvas.getBoundingClientRect();
    const gx = Math.floor(((e.clientX - rect.left) / rect.width) * GRID_SIZE);
    const gy = Math.floor(((e.clientY - rect.top) / rect.height) * GRID_SIZE);

    try {
      ui.plantSeed(gx, gy, selectedType);
      ui.feedRegion(gx, gy, 3, 30.0);
    } catch (err) {
      // Cell might already exist at position — silently ignore
    }
  });

  // Canvas drag: feed energy along path
  let dragging = false;
  canvas.addEventListener('mousedown', () => { dragging = true; });
  canvas.addEventListener('mouseup', () => { dragging = false; });
  canvas.addEventListener('mousemove', (e: MouseEvent) => {
    if (!dragging) return;
    const rect = canvas.getBoundingClientRect();
    const gx = Math.floor(((e.clientX - rect.left) / rect.width) * GRID_SIZE);
    const gy = Math.floor(((e.clientY - rect.top) / rect.height) * GRID_SIZE);
    ui.feedRegion(gx, gy, 2, 5.0);
  });

  // Action buttons
  document.getElementById('btn-feed-all')!.addEventListener('click', () => {
    ui.feedRegion(GRID_SIZE / 2, GRID_SIZE / 2, GRID_SIZE, 100.0);
  });

  document.getElementById('btn-efficiency')!.addEventListener('click', () => {
    ui.applySelectionPressure('EnergyEfficiency');
  });

  document.getElementById('btn-visual')!.addEventListener('click', () => {
    ui.applySelectionPressure('VisualAppeal');
  });

  document.getElementById('btn-cooperation')!.addEventListener('click', () => {
    ui.applySelectionPressure('Cooperation');
  });

  document.getElementById('btn-export')!.addEventListener('click', () => {
    try {
      const json = ui.export();
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'alive-garden-snapshot.json';
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error('Export failed:', err);
    }
  });

  // Stats elements
  const statCells = document.getElementById('stat-cells')!;
  const statEnergy = document.getElementById('stat-energy')!;
  const statTime = document.getElementById('stat-time')!;

  // Animation loop
  function animate() {
    ui.step(1.0 / 60.0);
    ui.render();

    // Update stats every ~10 frames
    if (Math.random() < 0.1) {
      statCells.textContent = String(ui.cellCount());
      statEnergy.textContent = ui.totalEnergy().toFixed(1);
      statTime.textContent = ui.time().toFixed(1) + 's';
    }

    requestAnimationFrame(animate);
  }

  requestAnimationFrame(animate);
}

main().catch(console.error);
