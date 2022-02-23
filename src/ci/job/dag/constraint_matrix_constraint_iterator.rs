use std::collections::{BTreeMap, BTreeSet};

pub struct ConstraintMatrixConstraintIterator {
    proximity: Vec<String>,
}

impl ConstraintMatrixConstraintIterator {
    pub fn new(cm: &BTreeMap<String, BTreeSet<String>>, blocking: String) -> Self {
        let mut accumulator = BTreeSet::new();
        let mut stack = vec![blocking];

        while let Some(current) = stack.pop() {
            let blocks = cm.get(&current).unwrap();
            for block in blocks {
                if accumulator.insert(block.clone()) {
                    stack.push(block.clone());
                }
            }
        }

        let proximity = accumulator.iter().rev().cloned().collect();

        Self { proximity }
    }
}

impl Iterator for ConstraintMatrixConstraintIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.proximity.pop()
    }
}
