// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Propagation benchmarks for the cliffy-core FRP graph.
//!
//! Covers the hot paths named in the v0.5.0 backlog (performance profiling
//! of subscription propagation): fan-out to N subscribers, map-chain depth,
//! and raw update throughput.

use cliffy_core::Behavior;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::cell::Cell;
use std::rc::Rc;

/// One `update` → N subscriber callbacks.
fn fanout(c: &mut Criterion) {
    let mut group = c.benchmark_group("propagation/fanout");
    for &subscriber_count in &[10usize, 100, 1000] {
        let behavior = Behavior::new(0.0f64);
        let ticks = Rc::new(Cell::new(0u64));
        let subscriptions: Vec<_> = (0..subscriber_count)
            .map(|_| {
                let ticks = ticks.clone();
                behavior.subscribe(move |_| ticks.set(ticks.get().saturating_add(1)))
            })
            .collect();

        group.throughput(Throughput::Elements(subscriber_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(subscriber_count),
            &subscriber_count,
            |b, _| {
                b.iter(|| behavior.update(|v| v + 1.0));
            },
        );
        drop(subscriptions);
    }
    group.finish();
}

/// One `update` at the head → propagation down a `map` chain of depth D.
fn map_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("propagation/map_chain");
    for &depth in &[1usize, 4, 8, 16] {
        let head = Behavior::new(1.0f64);
        let tail = {
            let mut current = head.clone();
            for _ in 0..depth {
                current = current.map(|v: f64| v + 1.0);
            }
            current
        };
        // Sink subscription keeps the chain live and observable.
        let ticks = Rc::new(Cell::new(0u64));
        let sink_ticks = ticks.clone();
        let sink = tail.subscribe(move |_| sink_ticks.set(sink_ticks.get().saturating_add(1)));

        group.throughput(Throughput::Elements(depth as u64));
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, _| {
            b.iter(|| head.update(|v| v + 1.0));
        });
        drop(sink);
    }
    group.finish();
}

/// Raw `set` throughput on an unsubscribed behavior (floor cost).
fn set_floor(c: &mut Criterion) {
    let behavior = Behavior::new(0.0f64);
    c.bench_function("propagation/set_floor", |b| {
        let mut i = 0.0f64;
        b.iter(|| {
            i += 1.0;
            behavior.set(i);
        });
    });
}

criterion_group!(benches, fanout, map_chain, set_floor);
criterion_main!(benches);
