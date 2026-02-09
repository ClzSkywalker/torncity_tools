use std::collections::{HashMap, HashSet};
use std::time::Instant;

pub use change::*;
pub use error::*;
pub use hash::*;
pub use result::*;
pub use recovery::*;

pub mod hash;

mod change;
mod error;
mod result;
mod recovery;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    NoChange,
    ContentChanged,
    OrderChanged,
    Added,
    Removed,
}

#[derive(Debug, Clone)]
pub struct OrderChangeDetector<T> {
    old_data: Vec<T>,
    new_data: Vec<T>,
    config: DetectorConfig,
}

#[derive(Debug, Clone)]
pub struct DetectorConfig {
    pub enable_hash_cache: bool,
    pub enable_performance_metrics: bool,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            enable_hash_cache: true,
            enable_performance_metrics: true,
        }
    }
}

impl<T: ContentHashable + PartialEq + Clone> OrderChangeDetector<T> {
    pub fn new(old_data: Vec<T>, new_data: Vec<T>) -> Self {
        Self {
            old_data,
            new_data,
            config: DetectorConfig::default(),
        }
    }

    pub fn with_config(mut self, config: DetectorConfig) -> Self {
        self.config = config;
        self
    }

    pub fn detect(&self) -> Result<OrderChangeReport<T>> {
        let start_time = if self.config.enable_performance_metrics {
            Some(Instant::now())
        } else {
            None
        };

        let mut report = OrderChangeReport::default();

        let old_map: HashMap<ContentHash, T> = self
            .old_data
            .iter()
            .map(|item| (item.content_hash(), item.clone()))
            .collect();

        let new_map: HashMap<ContentHash, T> = self
            .new_data
            .iter()
            .map(|item| (item.content_hash(), item.clone()))
            .collect();

        let old_hashes: HashSet<ContentHash> = old_map.keys().cloned().collect();
        let new_hashes: HashSet<ContentHash> = new_map.keys().cloned().collect();

        let added_hashes = &new_hashes - &old_hashes;
        let removed_hashes = &old_hashes - &new_hashes;
        let common_hashes = &old_hashes & &new_hashes;

        report.added_count = added_hashes.len();
        report.removed_count = removed_hashes.len();

        let old_positions: HashMap<ContentHash, usize> = self
            .old_data
            .iter()
            .enumerate()
            .map(|(idx, item)| (item.content_hash(), idx))
            .collect();

        let mut order_changed_items: Vec<OrderChangeItem<T>> = Vec::new();
        let mut unchanged_items: Vec<OrderChangeItem<T>> = Vec::new();
        let mut content_changed_items: Vec<OrderChangeItem<T>> = Vec::new();

        for (new_idx, new_item) in self.new_data.iter().enumerate() {
            let new_hash = new_item.content_hash();

            if common_hashes.contains(&new_hash) {
                let old_idx = old_positions[&new_hash];
                let old_item = &self.old_data[old_idx];

                if new_item != old_item {
                    content_changed_items.push(OrderChangeItem {
                        item: new_item.clone(),
                        old_position: Some(old_idx),
                        new_position: Some(new_idx),
                        change_type: ChangeType::ContentChanged,
                    });
                } else if old_idx != new_idx {
                    order_changed_items.push(OrderChangeItem {
                        item: new_item.clone(),
                        old_position: Some(old_idx),
                        new_position: Some(new_idx),
                        change_type: ChangeType::OrderChanged,
                    });
                } else {
                    unchanged_items.push(OrderChangeItem {
                        item: new_item.clone(),
                        old_position: Some(old_idx),
                        new_position: Some(new_idx),
                        change_type: ChangeType::NoChange,
                    });
                }
            }
        }

        for hash in added_hashes {
            if let Some(item) = new_map.get(&hash) {
                let new_idx = self.new_data.iter().position(|x| x.content_hash() == hash);
                report.items.push(OrderChangeItem {
                    item: item.clone(),
                    old_position: None,
                    new_position: new_idx,
                    change_type: ChangeType::Added,
                });
            }
        }

        for hash in removed_hashes {
            if let Some(item) = old_map.get(&hash) {
                let old_idx = self.old_data.iter().position(|x| x.content_hash() == hash);
                report.items.push(OrderChangeItem {
                    item: item.clone(),
                    old_position: old_idx,
                    new_position: None,
                    change_type: ChangeType::Removed,
                });
            }
        }

        let order_changed_count = order_changed_items.len();
        let unchanged_count = unchanged_items.len();
        let content_changed_count = content_changed_items.len();

        report.order_changed_count = order_changed_count;
        report.unchanged_count = unchanged_count;
        report.content_changed_count = content_changed_count;

        report.items.extend(content_changed_items);
        report.items.extend(order_changed_items);
        report.items.extend(unchanged_items);

        report.items.sort_by(|a, b| {
            match (a.new_position, b.new_position) {
                (Some(a_pos), Some(b_pos)) => a_pos.cmp(&b_pos),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        report.has_changes = report.added_count > 0
            || report.removed_count > 0
            || report.content_changed_count > 0
            || report.order_changed_count > 0;

        if let Some(start) = start_time {
            report.detection_time_us = start.elapsed().as_micros();
        }

        Ok(report)
    }

    pub fn is_content_only_changed(&self) -> bool {
        if self.old_data.len() != self.new_data.len() {
            return false;
        }

        let mut old_set: HashSet<ContentHash> = self
            .old_data
            .iter()
            .map(|x| x.content_hash())
            .collect();

        for item in &self.new_data {
            let hash = item.content_hash();
            if !old_set.contains(&hash) {
                return false;
            }
            old_set.remove(&hash);
        }

        true
    }

    pub fn is_order_only_changed(&self) -> bool {
        if !self.is_content_only_changed() {
            return false;
        }

        for (idx, item) in self.new_data.iter().enumerate() {
            if self.old_data.get(idx).map(|x| x.content_hash()) != Some(item.content_hash()) {
                return true;
            }
        }

        false
    }
}

pub struct DataReorderer<T> {
    items: Vec<T>,
}

impl<T: ContentHashable + PartialEq + Clone> DataReorderer<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self { items }
    }

    pub fn reorder(&mut self, target_order: &[T]) -> Result<()> {
        let target_hashes: Vec<ContentHash> = target_order.iter().map(|x| x.content_hash()).collect();
        let current_hashes: Vec<ContentHash> = self.items.iter().map(|x| x.content_hash()).collect();

        if target_hashes.len() != current_hashes.len() {
            return Err(OrderChangeError::LengthMismatch {
                current: current_hashes.len(),
                target: target_hashes.len(),
            });
        }

        let target_set: HashSet<ContentHash> = target_hashes.iter().cloned().collect();
        let current_set: HashSet<ContentHash> = current_hashes.iter().cloned().collect();

        if target_set != current_set {
            return Err(OrderChangeError::ContentMismatch);
        }

        let mut new_order = Vec::with_capacity(self.items.len());
        let mut item_map: HashMap<ContentHash, T> = self
            .items
            .drain(..)
            .map(|item| (item.content_hash(), item))
            .collect();

        for target_hash in target_hashes {
            if let Some(item) = item_map.remove(&target_hash) {
                new_order.push(item);
            }
        }

        self.items = new_order;
        Ok(())
    }

    pub fn apply_order_change(&mut self, report: &OrderChangeReport<T>) -> Result<()> {
        let mut new_items = Vec::with_capacity(self.items.len());
        let mut item_map: HashMap<ContentHash, T> = self
            .items
            .drain(..)
            .map(|item| (item.content_hash(), item))
            .collect();

        for change_item in &report.items {
            let hash = change_item.item.content_hash();
            if let Some(item) = item_map.remove(&hash) {
                new_items.push(item);
            }
        }

        self.items = new_items;
        Ok(())
    }

    pub fn into_inner(self) -> Vec<T> {
        self.items
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }
}
