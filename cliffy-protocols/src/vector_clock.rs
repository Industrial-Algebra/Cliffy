// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    pub clocks: HashMap<Uuid, u64>,
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorClock {
    #[must_use]
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    pub fn tick(&mut self, node_id: Uuid) {
        *self.clocks.entry(node_id).or_insert(0) += 1;
    }

    pub fn update(&mut self, other: &Self) {
        for (&node_id, &timestamp) in &other.clocks {
            let current = self.clocks.entry(node_id).or_insert(0);
            *current = (*current).max(timestamp);
        }
    }

    #[must_use]
    pub fn happens_before(&self, other: &Self) -> bool {
        let mut has_smaller = false;

        // Check all entries in other
        for (&node_id, &other_time) in &other.clocks {
            let self_time = *self.clocks.get(&node_id).unwrap_or(&0);
            if self_time > other_time {
                return false;
            }
            if self_time < other_time {
                has_smaller = true;
            }
        }

        // Check entries in self that aren't in other
        for (&node_id, &self_time) in &self.clocks {
            if !other.clocks.contains_key(&node_id) {
                // self has a non-zero entry that other doesn't have (implicitly 0)
                if self_time > 0 {
                    return false;
                }
            }
        }

        has_smaller
    }

    #[must_use]
    pub fn concurrent(&self, other: &Self) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }

    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.update(other);
        result
    }
}
