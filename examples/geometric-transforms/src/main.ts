/**
 * Cliffy Geometric Transforms Example
 *
 * Demonstrates:
 * - Rotor rotations in XY, XZ, YZ planes
 * - Transform composition (rotation + translation)
 * - .blend() interpolation between transforms
 * - GeometricState for position tracking
 * - Real-time visualization of geometric algebra operations
 */

import init, {
  behavior,
  Rotor,
  Translation,
  Transform,
  GeometricState,
} from '@industrialalgebra/cliffy-core';
import { html, mount } from '@industrialalgebra/cliffy-core/html';

async function main() {
  await init();

  // =========================================================================
  // Demo 1: Rotor Rotations
  // =========================================================================

  const rotorAngle = behavior(0);
  const rotorPlane = behavior('xy');

  // Compute the current rotor based on angle and plane
  const currentRotor = behavior(Rotor.identity());

  // Update rotor when angle or plane changes
  rotorAngle.subscribe((angle: number) => {
    const plane = rotorPlane.sample() as 'xy' | 'xz' | 'yz';
    updateRotor(angle, plane);
  });

  rotorPlane.subscribe((plane: 'xy' | 'xz' | 'yz') => {
    const angle = rotorAngle.sample() as number;
    updateRotor(angle, plane);
  });

  function updateRotor(angle: number, plane: 'xy' | 'xz' | 'yz') {
    const radians = (angle * Math.PI) / 180;
    let rotor: Rotor;
    switch (plane) {
      case 'xy':
        rotor = Rotor.xy(radians);
        break;
      case 'xz':
        rotor = Rotor.xz(radians);
        break;
      case 'yz':
        rotor = Rotor.yz(radians);
        break;
    }
    currentRotor.set(rotor);
  }

  // Apply rotor to a point and get CSS transform
  // Note: Must return full style string since html`` doesn't support partial Behavior in attributes
  const rotorStyle = currentRotor.map((rotor: Rotor) => {
    // For 2D visualization, we project the 3D rotation to 2D
    // XY plane rotation maps directly to CSS rotate
    const angleDeg = (rotor.angle() * 180) / Math.PI;
    return `left: 50%; top: 50%; transform: translate(-50%, -50%) rotate(${angleDeg}deg)`;
  });

  // Display rotor info
  const rotorInfo = currentRotor.map((rotor: Rotor) => {
    const angle = rotor.angle();
    return {
      angleDeg: ((angle * 180) / Math.PI).toFixed(1),
      angleRad: angle.toFixed(3),
    };
  });

  // =========================================================================
  // Demo 2: Transform Composition
  // =========================================================================

  const translateX = behavior(0);
  const translateY = behavior(0);
  const composeAngle = behavior(0);

  // Create composed transform
  const composedTransform = behavior(Transform.identity());

  function updateComposedTransform() {
    const tx = translateX.sample() as number;
    const ty = translateY.sample() as number;
    const angle = composeAngle.sample() as number;
    const radians = (angle * Math.PI) / 180;

    const rotor = Rotor.xy(radians);
    const translation = new Translation(tx, ty, 0);
    const transform = Transform.fromRotorAndTranslation(rotor, translation);
    composedTransform.set(transform);
  }

  translateX.subscribe(() => updateComposedTransform());
  translateY.subscribe(() => updateComposedTransform());
  composeAngle.subscribe(() => updateComposedTransform());

  // CSS for composed transform - full style string for reactive binding
  const composeStyle = composedTransform.map(() => {
    const tx = translateX.sample() as number;
    const ty = translateY.sample() as number;
    const angle = composeAngle.sample() as number;
    // Note: CSS applies transforms right-to-left, so translate then rotate
    return `left: 50%; top: 50%; transform: translate(-50%, -50%) translate(${tx}px, ${ty}px) rotate(${angle}deg)`;
  });

  // =========================================================================
  // Demo 3: Blend Interpolation
  // =========================================================================

  const blendT = behavior(0);
  const startAngle = behavior(0);
  const endAngle = behavior(180);

  // Create start and end rotors
  const blendedRotor = behavior(Rotor.identity());

  function updateBlend() {
    const t = blendT.sample() as number;
    const start = (startAngle.sample() as number) * Math.PI / 180;
    const end = (endAngle.sample() as number) * Math.PI / 180;

    const startRotor = Rotor.xy(start);
    const endRotor = Rotor.xy(end);

    // Use .blend() for smooth interpolation (SLERP internally)
    const blended = startRotor.blend(endRotor, t);
    blendedRotor.set(blended);
  }

  blendT.subscribe(() => updateBlend());
  startAngle.subscribe(() => updateBlend());
  endAngle.subscribe(() => updateBlend());
  updateBlend();

  const blendStyle = blendedRotor.map((rotor: Rotor) => {
    const angleDeg = (rotor.angle() * 180) / Math.PI;
    return `left: 50%; top: 50%; transform: translate(-50%, -50%) rotate(${angleDeg}deg)`;
  });

  const blendInfo = blendedRotor.map((rotor: Rotor) => {
    const t = blendT.sample() as number;
    return {
      t: t.toFixed(2),
      angleDeg: ((rotor.angle() * 180) / Math.PI).toFixed(1),
    };
  });

  // Blend marker position style
  const blendMarkerStyle = blendT.map((t: number) => `left: ${t * 100}%`);

  // =========================================================================
  // Demo 4: GeometricState Transforms
  // =========================================================================

  const stateX = behavior(0);
  const stateY = behavior(0);
  const stateRotation = behavior(0);

  // Create and transform GeometricState
  const geometricPosition = behavior({ x: 100, y: 100 });

  function updateGeometricState() {
    const x = stateX.sample() as number;
    const y = stateY.sample() as number;
    const angle = (stateRotation.sample() as number) * Math.PI / 180;

    // Create a point as GeometricState
    const point = GeometricState.fromVector(50, 0, 0);

    // Apply rotation
    const rotor = Rotor.xy(angle);
    const rotated = point.applyRotor(rotor);

    // Get result as vector
    const vec = rotated.asVector();

    geometricPosition.set({
      x: 100 + x + vec[0],
      y: 100 + y + vec[1],
    });
  }

  stateX.subscribe(() => updateGeometricState());
  stateY.subscribe(() => updateGeometricState());
  stateRotation.subscribe(() => updateGeometricState());
  updateGeometricState();

  const stateStyle = geometricPosition.map((pos: { x: number; y: number }) => {
    return `left: 0; top: 0; transform: translate(${pos.x - 30}px, ${pos.y - 30}px)`;
  });

  const stateInfo = geometricPosition.map((pos: { x: number; y: number }) => {
    return {
      x: pos.x.toFixed(1),
      y: pos.y.toFixed(1),
    };
  });

  // =========================================================================
  // UI
  // =========================================================================

  const app = html`
    <div class="demo-grid">
      <!-- Demo 1: Rotor Rotations -->
      <div class="demo-card">
        <h2>Rotor Rotations</h2>
        <p>Rotations without gimbal lock using geometric algebra rotors</p>

        <div class="canvas-container">
          <div
            class="transform-target arrow"
            style=${rotorStyle}
          >
            R
          </div>
        </div>

        <div class="controls">
          <div class="control-row">
            <label>Angle</label>
            <input
              type="range"
              min="-180"
              max="180"
              value="0"
              oninput=${(e: Event) => rotorAngle.set(Number((e.target as HTMLInputElement).value))}
            />
            <span class="value">${rotorAngle.map((a: number) => `${a}°`)}</span>
          </div>

          <div class="button-row">
            <button
              class=${rotorPlane.map((p: string) => p === 'xy' ? 'active' : '')}
              onclick=${() => rotorPlane.set('xy')}
            >
              XY Plane
            </button>
            <button
              class=${rotorPlane.map((p: string) => p === 'xz' ? 'active' : '')}
              onclick=${() => rotorPlane.set('xz')}
            >
              XZ Plane
            </button>
            <button
              class=${rotorPlane.map((p: string) => p === 'yz' ? 'active' : '')}
              onclick=${() => rotorPlane.set('yz')}
            >
              YZ Plane
            </button>
          </div>
        </div>

        <div class="info-panel">
          <span class="label">Rotor angle: </span>
          <span class="value">${rotorInfo.map((i: { angleDeg: string }) => i.angleDeg)}° </span>
          <span class="label">(</span>
          <span class="value">${rotorInfo.map((i: { angleRad: string }) => i.angleRad)}</span>
          <span class="label"> rad)</span>
        </div>
      </div>

      <!-- Demo 2: Transform Composition -->
      <div class="demo-card">
        <h2>Transform Composition</h2>
        <p>Combine rotation and translation into a single transform</p>

        <div class="canvas-container">
          <div
            class="transform-target"
            style=${composeStyle}
          >
            T
          </div>
        </div>

        <div class="controls">
          <div class="control-row">
            <label>Translate X</label>
            <input
              type="range"
              min="-100"
              max="100"
              value="0"
              oninput=${(e: Event) => translateX.set(Number((e.target as HTMLInputElement).value))}
            />
            <span class="value">${translateX.map((v: number) => `${v}px`)}</span>
          </div>
          <div class="control-row">
            <label>Translate Y</label>
            <input
              type="range"
              min="-100"
              max="100"
              value="0"
              oninput=${(e: Event) => translateY.set(Number((e.target as HTMLInputElement).value))}
            />
            <span class="value">${translateY.map((v: number) => `${v}px`)}</span>
          </div>
          <div class="control-row">
            <label>Rotate</label>
            <input
              type="range"
              min="-180"
              max="180"
              value="0"
              oninput=${(e: Event) => composeAngle.set(Number((e.target as HTMLInputElement).value))}
            />
            <span class="value">${composeAngle.map((v: number) => `${v}°`)}</span>
          </div>
        </div>
      </div>

      <!-- Demo 3: Blend Interpolation -->
      <div class="demo-card">
        <h2>Blend Interpolation</h2>
        <p>Smooth SLERP interpolation between rotors using .blend()</p>

        <div class="canvas-container">
          <div
            class="transform-target arrow"
            style=${blendStyle}
          >
            B
          </div>
        </div>

        <div class="blend-visualization">
          <div class="blend-point start">${startAngle.map((a: number) => `${a}°`)}</div>
          <div class="blend-line">
            <div
              class="blend-marker"
              style=${blendMarkerStyle}
            ></div>
          </div>
          <div class="blend-point end">${endAngle.map((a: number) => `${a}°`)}</div>
        </div>

        <div class="controls">
          <div class="control-row">
            <label>t (blend)</label>
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value="0"
              oninput=${(e: Event) => blendT.set(Number((e.target as HTMLInputElement).value))}
            />
            <span class="value">${blendInfo.map((i: { t: string }) => i.t)}</span>
          </div>
          <div class="control-row">
            <label>Start</label>
            <input
              type="range"
              min="-180"
              max="180"
              value="0"
              oninput=${(e: Event) => startAngle.set(Number((e.target as HTMLInputElement).value))}
            />
            <span class="value">${startAngle.map((a: number) => `${a}°`)}</span>
          </div>
          <div class="control-row">
            <label>End</label>
            <input
              type="range"
              min="-180"
              max="180"
              value="180"
              oninput=${(e: Event) => endAngle.set(Number((e.target as HTMLInputElement).value))}
            />
            <span class="value">${endAngle.map((a: number) => `${a}°`)}</span>
          </div>
        </div>

        <div class="info-panel">
          <span class="label">Blended angle: </span>
          <span class="value">${blendInfo.map((i: { angleDeg: string }) => i.angleDeg)}°</span>
          <span class="label"> (t = </span>
          <span class="value">${blendInfo.map((i: { t: string }) => i.t)}</span>
          <span class="label">)</span>
        </div>
      </div>

      <!-- Demo 4: GeometricState -->
      <div class="demo-card">
        <h2>GeometricState Transforms</h2>
        <p>Apply rotors to geometric state vectors</p>

        <div class="canvas-container">
          <div
            class="transform-target"
            style=${stateStyle}
          >
            G
          </div>
          <!-- Origin marker -->
          <div style="position: absolute; left: calc(50% - 4px); top: calc(50% - 4px); width: 8px; height: 8px; background: rgba(255,255,255,0.3); border-radius: 50%;"></div>
        </div>

        <div class="controls">
          <div class="control-row">
            <label>Offset X</label>
            <input
              type="range"
              min="-50"
              max="50"
              value="0"
              oninput=${(e: Event) => stateX.set(Number((e.target as HTMLInputElement).value))}
            />
            <span class="value">${stateX.map((v: number) => `${v}`)}</span>
          </div>
          <div class="control-row">
            <label>Offset Y</label>
            <input
              type="range"
              min="-50"
              max="50"
              value="0"
              oninput=${(e: Event) => stateY.set(Number((e.target as HTMLInputElement).value))}
            />
            <span class="value">${stateY.map((v: number) => `${v}`)}</span>
          </div>
          <div class="control-row">
            <label>Rotation</label>
            <input
              type="range"
              min="0"
              max="360"
              value="0"
              oninput=${(e: Event) => stateRotation.set(Number((e.target as HTMLInputElement).value))}
            />
            <span class="value">${stateRotation.map((v: number) => `${v}°`)}</span>
          </div>
        </div>

        <div class="info-panel">
          <span class="label">Position: (</span>
          <span class="value">${stateInfo.map((i: { x: string }) => i.x)}</span>
          <span class="label">, </span>
          <span class="value">${stateInfo.map((i: { y: string }) => i.y)}</span>
          <span class="label">)</span>
        </div>
      </div>
    </div>
  `;

  mount(app, '#app');

  console.log('Cliffy Geometric Transforms initialized');
}

main().catch(console.error);
