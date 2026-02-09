use super::OrderChangeDetector;

#[derive(Debug, Clone)]
pub struct ChangeSummary {
    pub total_items: usize,
    pub added_count: usize,
    pub removed_count: usize,
    pub content_changed_count: usize,
    pub order_changed_count: usize,
    pub unchanged_count: usize,
    pub change_type: OverallChangeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverallChangeType {
    NoChange,
    OrderOnly,
    ContentOnly,
    Mixed,
}

impl<T: ContentHashable + PartialEq + Clone> OrderChangeDetector<T> {
    pub fn get_change_summary(&self) -> ChangeSummary {
        let report = match self.detect() {
            Ok(r) => r,
            Err(_) => {
                return ChangeSummary {
                    total_items: self.new_data.len(),
                    added_count: 0,
                    removed_count: 0,
                    content_changed_count: 0,
                    order_changed_count: 0,
                    unchanged_count: 0,
                    change_type: OverallChangeType::ContentOnly,
                }
            }
        };

        let change_type = if report.added_count == 0
            && report.removed_count == 0
            && report.content_changed_count == 0
            && report.order_changed_count == 0
        {
            OverallChangeType::NoChange
        } else if report.added_count == 0 && report.removed_count == 0 && report.content_changed_count == 0 {
            OverallChangeType::OrderOnly
        } else if report.order_changed_count == 0 {
            OverallChangeType::ContentOnly
        } else {
            OverallChangeType::Mixed
        };

        ChangeSummary {
            total_items: report.items.len(),
            added_count: report.added_count,
            removed_count: report.removed_count,
            content_changed_count: report.content_changed_count,
            order_changed_count: report.order_changed_count,
            unchanged_count: report.unchanged_count,
            change_type,
        }
    }

    pub fn needs_reorder(&self) -> bool {
        self.is_order_only_changed()
    }

    pub fn needs_full_update(&self) -> bool {
        !self.is_content_only_changed()
    }
}

use super::ContentHashable;
