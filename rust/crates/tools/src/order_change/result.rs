use super::{ChangeType, ContentHashable};

#[derive(Debug, Clone)]
pub struct OrderChangeReport<T> {
    pub items: Vec<OrderChangeItem<T>>,
    pub added_count: usize,
    pub removed_count: usize,
    pub content_changed_count: usize,
    pub order_changed_count: usize,
    pub unchanged_count: usize,
    pub has_changes: bool,
    pub detection_time_us: u128,
}

impl<T> Default for OrderChangeReport<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            added_count: 0,
            removed_count: 0,
            content_changed_count: 0,
            order_changed_count: 0,
            unchanged_count: 0,
            has_changes: false,
            detection_time_us: 0,
        }
    }
}

impl<T: ContentHashable> OrderChangeReport<T> {
    pub fn get_added_items(&self) -> Vec<&T> {
        self.items
            .iter()
            .filter(|x| x.change_type == ChangeType::Added)
            .map(|x| &x.item)
            .collect()
    }

    pub fn get_removed_items(&self) -> Vec<&T> {
        self.items
            .iter()
            .filter(|x| x.change_type == ChangeType::Removed)
            .map(|x| &x.item)
            .collect()
    }

    pub fn get_content_changed_items(&self) -> Vec<&OrderChangeItem<T>> {
        self.items
            .iter()
            .filter(|x| x.change_type == ChangeType::ContentChanged)
            .collect()
    }

    pub fn get_order_changed_items(&self) -> Vec<&OrderChangeItem<T>> {
        self.items
            .iter()
            .filter(|x| x.change_type == ChangeType::OrderChanged)
            .collect()
    }

    pub fn get_unchanged_items(&self) -> Vec<&T> {
        self.items
            .iter()
            .filter(|x| x.change_type == ChangeType::NoChange)
            .map(|x| &x.item)
            .collect()
    }

    pub fn has_only_order_changes(&self) -> bool {
        self.added_count == 0
            && self.removed_count == 0
            && self.order_changed_count > 0
    }

    pub fn has_only_content_changes(&self) -> bool {
        self.added_count > 0 || self.removed_count > 0 || self.content_changed_count > 0
    }

    pub fn has_mixed_changes(&self) -> bool {
        (self.added_count > 0 || self.removed_count > 0 || self.content_changed_count > 0) && self.order_changed_count > 0
    }

    pub fn summary(&self) -> String {
        format!(
            "OrderChangeReport: added={}, removed={}, content_changed={}, order_changed={}, unchanged={}, has_changes={}, detection_time_us={}",
            self.added_count,
            self.removed_count,
            self.content_changed_count,
            self.order_changed_count,
            self.unchanged_count,
            self.has_changes,
            self.detection_time_us
        )
    }
}

#[derive(Debug, Clone)]
pub struct OrderChangeItem<T> {
    pub item: T,
    pub old_position: Option<usize>,
    pub new_position: Option<usize>,
    pub change_type: ChangeType,
}
