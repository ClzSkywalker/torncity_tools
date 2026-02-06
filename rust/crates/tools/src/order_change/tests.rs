use super::*;
use super::hash::ContentHashable;

#[derive(Debug, Clone, PartialEq)]
struct TestItem {
    id: i32,
    name: String,
    value: f32,
}

impl ContentHashable for TestItem {
    fn content_hash(&self) -> ContentHash {
        let mut hasher = super::hash::StableHasher::new();
        hasher.write_i32(self.id);
        hasher.write_str(&self.name);
        hasher.write_f32(self.value);
        hasher.finish()
    }
}

#[test]
fn test_no_change() {
    let items = vec![
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
        TestItem { id: 2, name: "B".to_string(), value: 20.0 },
        TestItem { id: 3, name: "C".to_string(), value: 30.0 },
    ];

    let detector = OrderChangeDetector::new(items.clone(), items.clone());
    let report = detector.detect().unwrap();

    assert!(!report.has_changes);
    assert_eq!(report.added_count, 0);
    assert_eq!(report.removed_count, 0);
    assert_eq!(report.order_changed_count, 0);
    assert_eq!(report.unchanged_count, 3);
}

#[test]
fn test_order_only_changed() {
    let old_items = vec![
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
        TestItem { id: 2, name: "B".to_string(), value: 20.0 },
        TestItem { id: 3, name: "C".to_string(), value: 30.0 },
    ];

    let new_items = vec![
        TestItem { id: 3, name: "C".to_string(), value: 30.0 },
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
        TestItem { id: 2, name: "B".to_string(), value: 20.0 },
    ];

    let detector = OrderChangeDetector::new(old_items, new_items);
    let report = detector.detect().unwrap();

    eprintln!("Debug report: has_changes={}, added={}, removed={}, order_changed={}, unchanged={}",
        report.has_changes, report.added_count, report.removed_count, report.order_changed_count, report.unchanged_count);

    assert!(report.has_changes);
    assert_eq!(report.added_count, 0);
    assert_eq!(report.removed_count, 0);
    assert_eq!(report.order_changed_count, 3);
    assert_eq!(report.unchanged_count, 0);
}

#[test]
fn test_content_only_changed() {
    let old_items = vec![
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
        TestItem { id: 2, name: "B".to_string(), value: 20.0 },
    ];

    let new_items = vec![
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
        TestItem { id: 2, name: "B".to_string(), value: 20.0 },
        TestItem { id: 3, name: "C".to_string(), value: 30.0 },
    ];

    let detector = OrderChangeDetector::new(old_items, new_items);
    let report = detector.detect().unwrap();

    assert!(report.has_changes);
    assert_eq!(report.added_count, 1);
    assert_eq!(report.removed_count, 0);
}

#[test]
fn test_mixed_changes() {
    let old_items = vec![
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
        TestItem { id: 2, name: "B".to_string(), value: 20.0 },
        TestItem { id: 3, name: "C".to_string(), value: 30.0 },
    ];

    let new_items = vec![
        TestItem { id: 3, name: "C".to_string(), value: 30.0 },
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
        TestItem { id: 4, name: "D".to_string(), value: 40.0 },
    ];

    let detector = OrderChangeDetector::new(old_items, new_items);
    let report = detector.detect().unwrap();

    assert!(report.has_changes);
    assert_eq!(report.added_count, 1);
    assert_eq!(report.removed_count, 1);
}

#[test]
fn test_reorder() {
    let items = vec![
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
        TestItem { id: 2, name: "B".to_string(), value: 20.0 },
        TestItem { id: 3, name: "C".to_string(), value: 30.0 },
    ];

    let target_order = vec![
        TestItem { id: 3, name: "C".to_string(), value: 30.0 },
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
        TestItem { id: 2, name: "B".to_string(), value: 20.0 },
    ];

    let mut reorderer = DataReorderer::new(items);
    reorderer.reorder(&target_order).unwrap();

    let result = reorderer.into_inner();
    assert_eq!(result[0].id, 3);
    assert_eq!(result[1].id, 1);
    assert_eq!(result[2].id, 2);
}

#[test]
fn test_reorder_length_mismatch() {
    let items = vec![
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
        TestItem { id: 2, name: "B".to_string(), value: 20.0 },
    ];

    let target_order = vec![
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
    ];

    let mut reorderer = DataReorderer::new(items);
    let result = reorderer.reorder(&target_order);

    assert!(result.is_err());
    assert!(matches!(result, Err(OrderChangeError::LengthMismatch { .. })));
}

#[test]
fn test_reorder_content_mismatch() {
    let items = vec![
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
        TestItem { id: 2, name: "B".to_string(), value: 20.0 },
    ];

    let target_order = vec![
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
        TestItem { id: 3, name: "C".to_string(), value: 30.0 },
    ];

    let mut reorderer = DataReorderer::new(items);
    let result = reorderer.reorder(&target_order);

    assert!(result.is_err());
    assert!(matches!(result, Err(OrderChangeError::ContentMismatch)));
}

#[test]
fn test_content_hashable_string() {
    let s1 = "test";
    let s2 = "test";
    let s3 = "different";

    assert_eq!(s1.content_hash(), s2.content_hash());
    assert_ne!(s1.content_hash(), s3.content_hash());
}

#[test]
fn test_content_hashable_primitives() {
    assert_eq!(42i32.content_hash(), 42i32.content_hash());
    assert_ne!(42i32.content_hash(), 43i32.content_hash());

    assert_eq!(3.14f32.content_hash(), 3.14f32.content_hash());
    assert_ne!(3.14f32.content_hash(), 2.71f32.content_hash());
}

#[test]
fn test_change_summary_order_only() {
    let old_items = vec![
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
        TestItem { id: 2, name: "B".to_string(), value: 20.0 },
    ];

    let new_items = vec![
        TestItem { id: 2, name: "B".to_string(), value: 20.0 },
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
    ];

    let detector = OrderChangeDetector::new(old_items, new_items);
    let summary = detector.get_change_summary();

    assert_eq!(summary.change_type, OverallChangeType::OrderOnly);
    assert!(detector.needs_reorder());
    assert!(!detector.needs_full_update());
}

#[test]
fn test_change_summary_no_change() {
    let items = vec![
        TestItem { id: 1, name: "A".to_string(), value: 10.0 },
    ];

    let detector = OrderChangeDetector::new(items.clone(), items);
    let summary = detector.get_change_summary();

    assert_eq!(summary.change_type, OverallChangeType::NoChange);
    assert!(!detector.needs_reorder());
    assert!(!detector.needs_full_update());
}

#[test]
fn test_performance_large_dataset() {
    let old_items: Vec<TestItem> = (0..1000)
        .map(|i| TestItem {
            id: i,
            name: format!("Item{}", i),
            value: i as f32,
        })
        .collect();

    let mut new_items = old_items.clone();
    new_items.reverse();

    let detector = OrderChangeDetector::new(old_items, new_items);
    let report = detector.detect().unwrap();

    assert_eq!(report.items.len(), 1000);
    assert!(report.detection_time_us > 0);
}

#[test]
fn test_hash_stability() {
    let item = TestItem { id: 1, name: "A".to_string(), value: 10.0 };
    let hash1 = item.content_hash();
    let hash2 = item.content_hash();

    assert_eq!(hash1, hash2, "Hash should be stable across multiple calls");
}
