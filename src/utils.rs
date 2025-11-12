/// Split a slice of items into consecutive batches of size `batches_count`.
/// If `batches_count` is 0, returns an empty vector.
pub(crate) fn create_batches<T: Clone>(items: &[T], batches_count: usize) -> Vec<Vec<T>> {
    if batches_count == 0 {
        return Vec::new();
    }

    items
        .chunks(batches_count)
        .map(|chunk| chunk.to_vec())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::create_batches;

    #[test]
    fn test_create_batches_zero_size_returns_empty() {
        let items = vec![1, 2, 3, 4, 5];
        let batches: Vec<Vec<i32>> = create_batches(&items, 0);
        assert!(
            batches.is_empty(),
            "Expected empty vector when batch size is 0"
        );
    }

    #[test]
    fn test_create_batches_last_batch_not_full() {
        // 5 items with batch size 2 -> [[1,2],[3,4],[5]]
        let items = vec![1, 2, 3, 4, 5];
        let batches = create_batches(&items, 2);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0], vec![1, 2]);
        assert_eq!(batches[1], vec![3, 4]);
        assert_eq!(batches[2], vec![5]);
    }

    #[test]
    fn test_create_batches_exact_multiples() {
        // 6 items with batch size 2 -> [[1,2],[3,4],[5,6]]
        let items = vec![1, 2, 3, 4, 5, 6];
        let batches = create_batches(&items, 2);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0], vec![1, 2]);
        assert_eq!(batches[1], vec![3, 4]);
        assert_eq!(batches[2], vec![5, 6]);
    }
}
