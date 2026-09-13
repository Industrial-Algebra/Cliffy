/**
 * Tests for cliffy-tsukoshi protocols.
 *
 * These tests use the WASM-exposed cliffy-test framework for algebraic testing.
 */

import { describe, it, expect } from 'vitest';

// Import cliffy-test from WASM
import {
  testImpossible,
  testRare,
  TestResult,
  InvariantCategory,
  type InvariantTestReport,
} from '@industrialalgebra/cliffy-core';

// Import the TypeScript protocols
import { VectorClock } from './vector-clock.js';
import { GeometricCRDT, OperationType } from './crdt.js';
import { latticeJoin, latticeMeet } from './lattice.js';
import {
  DeltaBatch,
  computeDelta,
  applyAdditiveDelta,
  additiveDelta,
} from './delta.js';
import { MemoryStore } from './storage.js';
import { SyncState, PeerConnectionState } from './sync.js';
import { GeometricConsensus } from './consensus.js';

// Import GA3 functions from cliffy-tsukoshi
import {
  zero,
  scalar,
  vector,
  add,
  sub,
  magnitude,
  equals,
} from '../ga3.js';

// =============================================================================
// Vector Clock Tests
// =============================================================================

describe('VectorClock', () => {
  it('tick increments correctly', () => {
    const clock = new VectorClock();
    clock.tick('node-1');
    clock.tick('node-1');
    clock.tick('node-2');

    expect(clock.get('node-1')).toBe(2);
    expect(clock.get('node-2')).toBe(1);
    expect(clock.get('node-3')).toBe(0);
  });

  it('happensBefore establishes causal order', () => {
    const clock1 = new VectorClock();
    const clock2 = new VectorClock();

    clock1.tick('node-1');
    clock2.update(clock1);
    clock2.tick('node-2');

    expect(clock1.happensBefore(clock2)).toBe(true);
    expect(clock2.happensBefore(clock1)).toBe(false);
  });

  it('concurrent clocks are detected', () => {
    const clock1 = new VectorClock();
    const clock2 = new VectorClock();

    clock1.tick('node-1');
    clock2.tick('node-2');

    expect(clock1.concurrent(clock2)).toBe(true);
  });

  it('merge combines clocks correctly', () => {
    const clock1 = new VectorClock();
    const clock2 = new VectorClock();

    clock1.tick('node-1');
    clock1.tick('node-1');
    clock2.tick('node-2');

    const merged = clock1.merge(clock2);
    expect(merged.get('node-1')).toBe(2);
    expect(merged.get('node-2')).toBe(1);
  });

  it('serialization roundtrip preserves data', () => {
    const clock = new VectorClock();
    clock.tick('node-1');
    clock.tick('node-2');

    const json = clock.toJSON();
    const restored = VectorClock.fromJSON(json);

    expect(clock.equals(restored)).toBe(true);
  });

  it('equals correctly compares clocks', () => {
    const clock1 = new VectorClock();
    const clock2 = new VectorClock();

    clock1.tick('node-1');
    clock2.tick('node-1');

    expect(clock1.equals(clock2)).toBe(true);

    clock1.tick('node-1');
    expect(clock1.equals(clock2)).toBe(false);
  });
});

// =============================================================================
// GeometricCRDT Tests
// =============================================================================

describe('GeometricCRDT', () => {
  it('creates CRDT with initial state', () => {
    const crdt = new GeometricCRDT('node-1', scalar(10));
    expect(crdt.state[0]).toBe(10);
    expect(crdt.nodeId).toBe('node-1');
  });

  it('addition operations modify state correctly', () => {
    const crdt = new GeometricCRDT('node-1', scalar(10));
    const op = crdt.createOperation(scalar(5), OperationType.Addition);

    crdt.applyOperation(op);

    expect(crdt.state[0]).toBeCloseTo(15);
  });

  it('operations are idempotent', () => {
    const crdt = new GeometricCRDT('node-1', scalar(10));
    const op = crdt.createOperation(scalar(5), OperationType.Addition);

    crdt.applyOperation(op);
    const state1 = crdt.state[0];

    crdt.applyOperation(op); // Apply same op again
    const state2 = crdt.state[0];

    expect(state1).toBe(state2);
  });

  it('createOperation increments clock', () => {
    const crdt = new GeometricCRDT('node-1', scalar(0));

    const op1 = crdt.createOperation(scalar(5), OperationType.Addition);
    const op2 = crdt.createOperation(scalar(3), OperationType.Addition);

    expect(op1.id).toBe(0);
    expect(op2.id).toBe(1);
    expect(crdt.vectorClock.get('node-1')).toBe(2);
  });

  it('geometric product operation works', () => {
    const crdt = new GeometricCRDT('node-1', scalar(2));
    const op = crdt.createOperation(scalar(3), OperationType.GeometricProduct);

    crdt.applyOperation(op);

    expect(crdt.state[0]).toBeCloseTo(6);
  });
});

// =============================================================================
// Lattice Tests
// =============================================================================

describe('Lattice Operations', () => {
  it('latticeJoin computes component-wise maximum', () => {
    const a = vector(1, 3, 2);
    const b = vector(2, 1, 4);

    const joined = latticeJoin(a, b);

    expect(joined[1]).toBe(2); // e1: max(1, 2)
    expect(joined[2]).toBe(3); // e2: max(3, 1)
    expect(joined[4]).toBe(4); // e3: max(2, 4)
  });

  it('latticeMeet computes component-wise minimum', () => {
    const a = vector(1, 3, 2);
    const b = vector(2, 1, 4);

    const met = latticeMeet(a, b);

    expect(met[1]).toBe(1); // e1: min(1, 2)
    expect(met[2]).toBe(1); // e2: min(3, 1)
    expect(met[4]).toBe(2); // e3: min(2, 4)
  });

  it('latticeJoin is commutative', () => {
    const a = vector(1, 3, 5);
    const b = vector(2, 2, 4);

    const joinAB = latticeJoin(a, b);
    const joinBA = latticeJoin(b, a);

    expect(equals(joinAB, joinBA)).toBe(true);
  });

  it('latticeMeet is commutative', () => {
    const a = vector(1, 3, 5);
    const b = vector(2, 2, 4);

    const meetAB = latticeMeet(a, b);
    const meetBA = latticeMeet(b, a);

    expect(equals(meetAB, meetBA)).toBe(true);
  });

  it('lattice absorption law: a ∨ (a ∧ b) = a', () => {
    const a = vector(3, 5, 7);
    const b = vector(4, 4, 4);

    const meet = latticeMeet(a, b);
    const result = latticeJoin(a, meet);

    expect(equals(result, a)).toBe(true);
  });

  it('lattice absorption law: a ∧ (a ∨ b) = a', () => {
    const a = vector(3, 5, 7);
    const b = vector(4, 4, 4);

    const join = latticeJoin(a, b);
    const result = latticeMeet(a, join);

    expect(equals(result, a)).toBe(true);
  });
});

// =============================================================================
// Delta Synchronization Tests
// =============================================================================

describe('Delta Synchronization', () => {
  it('computeDelta produces correct difference', () => {
    const from = scalar(10);
    const to = scalar(25);
    const delta = computeDelta(from, to);

    expect(delta[0]).toBeCloseTo(15);
  });

  it('computeDelta works for vectors', () => {
    const from = vector(1, 2, 3);
    const to = vector(4, 5, 6);
    const delta = computeDelta(from, to);

    expect(delta[1]).toBeCloseTo(3); // e1: 4-1
    expect(delta[2]).toBeCloseTo(3); // e2: 5-2
    expect(delta[4]).toBeCloseTo(3); // e3: 6-3
  });

  it('applyAdditiveDelta reconstructs target', () => {
    const from = vector(1, 2, 3);
    const to = vector(4, 5, 6);
    const delta = computeDelta(from, to);

    const reconstructed = applyAdditiveDelta(from, delta);

    expect(equals(reconstructed, to)).toBe(true);
  });

  it('DeltaBatch accumulates deltas', () => {
    const batch = new DeltaBatch();
    const clock1 = new VectorClock();
    const clock2 = new VectorClock();
    clock1.tick('node-1');
    clock2.tick('node-1');

    batch.push(additiveDelta(scalar(5), clock1, clock2, 'node-1'));
    batch.push(additiveDelta(scalar(3), clock2, clock2.clone(), 'node-1'));

    expect(batch.length).toBe(2);
    expect(batch.isEmpty()).toBe(false);
  });

  it('DeltaBatch combineAdditive sums transforms', () => {
    const batch = new DeltaBatch();
    const clock1 = new VectorClock();
    const clock2 = new VectorClock();
    clock1.tick('node-1');
    clock2.tick('node-1');

    batch.push(additiveDelta(scalar(5), clock1, clock2, 'node-1'));
    batch.push(additiveDelta(scalar(3), clock2, clock2.clone(), 'node-1'));

    const combined = batch.combineAdditive();
    expect(combined).not.toBeNull();
    expect(combined![0]).toBeCloseTo(8);
  });

  it('DeltaBatch applyTo modifies state', () => {
    const batch = new DeltaBatch();
    const clock1 = new VectorClock();
    const clock2 = new VectorClock();
    clock1.tick('node-1');
    clock2.tick('node-1');

    batch.push(additiveDelta(scalar(5), clock1, clock2, 'node-1'));
    batch.push(additiveDelta(scalar(3), clock2, clock2.clone(), 'node-1'));

    const state = scalar(10);
    const result = batch.applyTo(state);

    expect(result[0]).toBeCloseTo(18); // 10 + 5 + 3
  });

  it('DeltaBatch serialization roundtrip', () => {
    const batch = new DeltaBatch();
    const clock1 = new VectorClock();
    const clock2 = new VectorClock();
    clock1.tick('node-1');
    clock2.tick('node-1');

    batch.push(additiveDelta(scalar(5), clock1, clock2, 'node-1'));

    const json = batch.toJSON();
    const restored = DeltaBatch.fromJSON(json);

    expect(restored.length).toBe(1);
  });
});

// =============================================================================
// Storage Tests
// =============================================================================

describe('Storage', () => {
  it('MemoryStore saves and retrieves snapshots', async () => {
    const store = new MemoryStore();
    const state = vector(1, 2, 3);
    const clock = new VectorClock();
    clock.tick('node-1');

    await store.saveSnapshot(state, clock);
    const result = await store.loadLatestSnapshot();

    expect(result).not.toBeNull();
    expect(equals(result!.state, state)).toBe(true);
  });

  it('MemoryStore clear removes all data', async () => {
    const store = new MemoryStore();
    const state = vector(1, 2, 3);
    const clock = new VectorClock();
    clock.tick('node-1');

    await store.saveSnapshot(state, clock);
    await store.clear();

    const result = await store.loadLatestSnapshot();
    expect(result).toBeNull();
  });

  it('MemoryStore stats returns correct counts', async () => {
    const store = new MemoryStore();
    const state = vector(1, 2, 3);
    const clock = new VectorClock();
    clock.tick('node-1');

    await store.saveSnapshot(state, clock);
    const stats = store.stats();

    expect(stats.snapshotCount).toBe(1);
  });
});

// =============================================================================
// Sync Protocol Tests
// =============================================================================

describe('Sync Protocol', () => {
  it('SyncState initializes correctly', () => {
    const sync = new SyncState('node-1');
    expect(sync.nodeId).toBe('node-1');
    expect(sync.peers.size).toBe(0);
  });

  it('SyncState registers and removes peers', () => {
    const sync = new SyncState('node-1');
    const clock = new VectorClock();

    sync.registerPeer('node-2', clock);
    expect(sync.peers.size).toBe(1);
    expect(sync.getPeer('node-2')).toBeDefined();

    sync.removePeer('node-2');
    expect(sync.peers.size).toBe(0);
  });

  it('SyncState creates hello message', () => {
    const sync = new SyncState('node-1');
    const msg = sync.createHello('Test Node');

    expect(msg.sender).toBe('node-1');
    expect(msg.payload.type).toBe('Hello');
    if (msg.payload.type === 'Hello') {
      expect(msg.payload.info.name).toBe('Test Node');
    }
  });

  it('SyncState creates heartbeat message', () => {
    const sync = new SyncState('node-1');
    const msg = sync.createHeartbeat();

    expect(msg.sender).toBe('node-1');
    expect(msg.payload.type).toBe('Heartbeat');
  });

  it('SyncState creates delta request message', () => {
    const sync = new SyncState('node-1');
    const clock = new VectorClock();
    clock.tick('node-2');

    const msg = sync.createDeltaRequest(clock);

    expect(msg.sender).toBe('node-1');
    expect(msg.payload.type).toBe('DeltaRequest');
  });

  it('tick increments clock and message ID', () => {
    const sync = new SyncState('node-1');

    const id1 = sync.tick();
    const id2 = sync.tick();

    expect(id1).toBe(0);
    expect(id2).toBe(1);
    expect(sync.clock.get('node-1')).toBe(2);
  });

  it('handleMessage updates peer state on hello', () => {
    const sync = new SyncState('node-1');

    const otherSync = new SyncState('node-2');
    const helloMsg = otherSync.createHello('Node 2');

    const response = sync.handleMessage(helloMsg);

    expect(response).not.toBeNull();
    expect(sync.peers.has('node-2')).toBe(true);
  });
});

// =============================================================================
// Consensus Tests
// =============================================================================

describe('Consensus', () => {
  it('consensus returns zero for empty proposals', () => {
    const consensus = new GeometricConsensus('node-1', scalar(0));
    const result = consensus.geometricConsensus([], 0.1);

    expect(equals(result, zero())).toBe(true);
  });

  it('propose increments round', () => {
    const consensus = new GeometricConsensus('node-1', scalar(0));

    const round1 = consensus.propose(scalar(10));
    const round2 = consensus.propose(scalar(20));

    expect(round1).toBe(0);
    expect(round2).toBe(1);
  });

  it('voting and commit workflow', () => {
    const consensus = new GeometricConsensus('node-1', scalar(0));

    // Receive proposals
    consensus.receiveProposal('node-2', scalar(10), 0);
    consensus.receiveProposal('node-3', scalar(12), 0);
    consensus.receiveProposal('node-4', scalar(14), 0);

    // Vote
    consensus.vote(0, true, scalar(12));
    consensus.receiveVote('node-2', 0, true);
    consensus.receiveVote('node-3', 0, true);

    // Try to commit (3 out of 4 voted yes)
    const committed = consensus.tryCommit(0, 4);
    expect(committed).not.toBeNull();
    expect(consensus.isCommitted(0)).toBe(true);
  });

  it('commit requires majority', () => {
    const consensus = new GeometricConsensus('node-1', scalar(0));

    consensus.receiveProposal('node-2', scalar(10), 0);
    consensus.vote(0, true, scalar(10));

    // Only 1 yes vote out of 4 participants - no majority
    const committed = consensus.tryCommit(0, 4);
    expect(committed).toBeNull();
    expect(consensus.isCommitted(0)).toBe(false);
  });

  it('getCommittedValue returns committed state', () => {
    const consensus = new GeometricConsensus('node-1', scalar(0));

    consensus.receiveProposal('node-2', scalar(10), 0);
    consensus.vote(0, true, scalar(10));
    consensus.receiveVote('node-2', 0, true);
    consensus.receiveVote('node-3', 0, true);

    consensus.tryCommit(0, 3);

    const value = consensus.getCommittedValue(0);
    expect(value).not.toBeNull();
  });

  it('message handlers receive broadcasts', () => {
    const consensus = new GeometricConsensus('node-1', scalar(0));
    const messages: any[] = [];

    consensus.onMessage((msg) => {
      messages.push(msg);
    });

    consensus.propose(scalar(10));

    expect(messages.length).toBe(1);
    expect(messages[0].messageType.type).toBe('Propose');
  });

  it('CRDT state updates after commit', () => {
    const consensus = new GeometricConsensus('node-1', scalar(0));

    consensus.receiveProposal('node-2', scalar(10), 0);
    consensus.vote(0, true, scalar(10));
    consensus.receiveVote('node-2', 0, true);
    consensus.receiveVote('node-3', 0, true);

    const initialState = consensus.getCurrentState();
    consensus.tryCommit(0, 3);
    const finalState = consensus.getCurrentState();

    // State should have changed after commit
    expect(magnitude(sub(finalState, initialState))).toBeGreaterThan(0);
  });
});

// =============================================================================
// Invariant Tests using cliffy-test WASM bindings
// =============================================================================

describe('Algebraic Invariants (cliffy-test)', () => {
  it('IMPOSSIBLE: Vector clock happensBefore is transitive', () => {
    const report = testImpossible(
      'VectorClock.happensBefore transitivity',
      () => {
        const a = new VectorClock();
        const b = new VectorClock();
        const c = new VectorClock();

        a.tick('n1');
        b.update(a);
        b.tick('n2');
        c.update(b);
        c.tick('n3');

        // If A -> B and B -> C, then A -> C
        const ab = a.happensBefore(b);
        const bc = b.happensBefore(c);
        const ac = a.happensBefore(c);

        return ab && bc && ac;
      },
      100
    );

    expect(report.verified).toBe(true);
    expect(report.category).toBe(InvariantCategory.Impossible);
    expect(report.failures).toBe(0);
  });

  it('IMPOSSIBLE: Lattice join is idempotent (a ∨ a = a)', () => {
    const report = testImpossible(
      'Lattice join idempotence',
      () => {
        const a = vector(
          Math.random() * 10,
          Math.random() * 10,
          Math.random() * 10
        );
        const result = latticeJoin(a, a);
        return equals(result, a);
      },
      100
    );

    expect(report.verified).toBe(true);
    expect(report.failures).toBe(0);
  });

  it('IMPOSSIBLE: Delta roundtrip preserves state', () => {
    const report = testImpossible(
      'Delta roundtrip preservation',
      () => {
        const from = vector(
          Math.random() * 10,
          Math.random() * 10,
          Math.random() * 10
        );
        const to = vector(
          Math.random() * 10,
          Math.random() * 10,
          Math.random() * 10
        );

        const delta = computeDelta(from, to);
        const reconstructed = applyAdditiveDelta(from, delta);

        return equals(reconstructed, to, 1e-10);
      },
      100
    );

    expect(report.verified).toBe(true);
    expect(report.failures).toBe(0);
  });

  it('IMPOSSIBLE: CRDT operations are idempotent', () => {
    const report = testImpossible(
      'CRDT operation idempotence',
      () => {
        const crdt = new GeometricCRDT('test', scalar(0));
        const op = crdt.createOperation(
          scalar(Math.random() * 10),
          OperationType.Addition
        );

        crdt.applyOperation(op);
        const state1 = crdt.state[0];

        crdt.applyOperation(op); // Apply same op again
        const state2 = crdt.state[0];

        return state1 === state2;
      },
      50
    );

    expect(report.verified).toBe(true);
    expect(report.failures).toBe(0);
  });

  it('IMPOSSIBLE: Vector clock merge is commutative', () => {
    const report = testImpossible(
      'VectorClock merge commutativity',
      () => {
        const clock1 = new VectorClock();
        const clock2 = new VectorClock();

        // Random ticks
        for (let i = 0; i < Math.floor(Math.random() * 5) + 1; i++) {
          clock1.tick('n1');
        }
        for (let i = 0; i < Math.floor(Math.random() * 5) + 1; i++) {
          clock2.tick('n2');
        }

        const merge12 = clock1.merge(clock2);
        const merge21 = clock2.merge(clock1);

        return merge12.equals(merge21);
      },
      100
    );

    expect(report.verified).toBe(true);
    expect(report.failures).toBe(0);
  });

  it('IMPOSSIBLE: Lattice join is associative', () => {
    const report = testImpossible(
      'Lattice join associativity',
      () => {
        const a = vector(Math.random() * 10, Math.random() * 10, Math.random() * 10);
        const b = vector(Math.random() * 10, Math.random() * 10, Math.random() * 10);
        const c = vector(Math.random() * 10, Math.random() * 10, Math.random() * 10);

        // (a ∨ b) ∨ c = a ∨ (b ∨ c)
        const left = latticeJoin(latticeJoin(a, b), c);
        const right = latticeJoin(a, latticeJoin(b, c));

        return equals(left, right);
      },
      100
    );

    expect(report.verified).toBe(true);
    expect(report.failures).toBe(0);
  });

  it('IMPOSSIBLE: Lattice meet is associative', () => {
    const report = testImpossible(
      'Lattice meet associativity',
      () => {
        const a = vector(Math.random() * 10, Math.random() * 10, Math.random() * 10);
        const b = vector(Math.random() * 10, Math.random() * 10, Math.random() * 10);
        const c = vector(Math.random() * 10, Math.random() * 10, Math.random() * 10);

        // (a ∧ b) ∧ c = a ∧ (b ∧ c)
        const left = latticeMeet(latticeMeet(a, b), c);
        const right = latticeMeet(a, latticeMeet(b, c));

        return equals(left, right);
      },
      100
    );

    expect(report.verified).toBe(true);
    expect(report.failures).toBe(0);
  });

  it('RARE: Concurrent vector clocks occur with random operations', () => {
    // When two nodes operate independently, they should often be concurrent
    const report = testRare(
      'Concurrent clocks from independent operations',
      0.3, // Allow up to 30% non-concurrent (sequential happens-before)
      () => {
        const clock1 = new VectorClock();
        const clock2 = new VectorClock();

        // Independent operations
        clock1.tick('n1');
        clock2.tick('n2');

        // They should be concurrent
        return clock1.concurrent(clock2);
      },
      100
    );

    expect(report.verified).toBe(true);
  });
});
