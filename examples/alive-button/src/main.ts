/**
 * Alive Button — Simplest Living UI demo
 *
 * A button that grows, evolves, and responds to clicks.
 * Demonstrates: WasmAliveUI, canvas rendering, energy feeding, selection pressure.
 */

import init, { WasmAliveUI } from 'cliffy-alive';

async function main() {
  await init();

  const ui = new WasmAliveUI();
  ui.useCanvasRenderer('alive-canvas', 400, 400);

  // Plant seeds in a button-like pattern (center of 50x50 grid)
  ui.plantSeed(25, 25, 'ButtonCore');
  ui.plantSeed(26, 25, 'ButtonEdge');
  ui.plantSeed(24, 25, 'ButtonEdge');
  ui.plantSeed(25, 26, 'ButtonEdge');
  ui.plantSeed(25, 24, 'ButtonEdge');

  // Initial energy boost
  ui.feedRegion(25, 25, 5, 50.0);

  const statsEl = document.getElementById('stats')!;
  const canvas = document.getElementById('alive-canvas') as HTMLCanvasElement;

  // Click on canvas to feed energy at that location
  canvas.addEventListener('click', (e: MouseEvent) => {
    const rect = canvas.getBoundingClientRect();
    const gridX = Math.floor(((e.clientX - rect.left) / rect.width) * 50);
    const gridY = Math.floor(((e.clientY - rect.top) / rect.height) * 50);
    ui.feedRegion(gridX, gridY, 3, 30.0);
  });

  // Control buttons
  document.getElementById('btn-feed')!.addEventListener('click', () => {
    ui.feedRegion(25, 25, 8, 100.0);
  });

  document.getElementById('btn-pressure')!.addEventListener('click', () => {
    ui.applySelectionPressure('VisualAppeal');
  });

  document.getElementById('btn-reset')!.addEventListener('click', () => {
    window.location.reload();
  });

  // Animation loop
  function animate() {
    ui.step(1.0 / 60.0);
    ui.render();

    // Update stats
    const cells = ui.cellCount();
    const energy = ui.totalEnergy();
    const time = ui.time();
    statsEl.textContent = `Cells: ${cells} | Energy: ${energy.toFixed(1)} | Time: ${time.toFixed(1)}s`;

    requestAnimationFrame(animate);
  }

  requestAnimationFrame(animate);
}

main().catch(console.error);
