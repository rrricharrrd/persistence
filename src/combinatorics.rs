use thiserror::Error;

/// Error types for combinatorial operations
#[derive(Error, Debug)]
pub enum CombinatoricsError {
    #[error("Size {size} exceeds input length {len}")]
    InvalidSize { size: usize, len: usize },
    #[error("Input too large: would generate {0} combinations")]
    TooLarge(usize),
}

/// Iterator over combinations of elements.
pub struct Combinations<T> {
    elements: Vec<T>,
    indices: Vec<usize>,
    first: bool,
    size: usize,
}

impl<T: Clone> Combinations<T> {
    /// Creates a new combinations iterator.
    ///
    /// # Arguments
    ///
    /// * `elements` - Vector of elements to generate combinations from
    /// * `size` - Size of each combination
    ///
    /// # Returns
    ///
    /// A Result containing either the iterator or an error if:
    /// - The size exceeds the number of elements
    /// - The number of possible combinations is too large
    pub fn new(elements: Vec<T>, size: usize) -> Result<Self, CombinatoricsError> {
        let len = elements.len();
        if size > len {
            return Err(CombinatoricsError::InvalidSize { size, len });
        }

        // Check if the number of combinations would be too large
        let n_combinations = (1..=size).fold(1, |acc, i| acc * (len - size + i) / i);
        if n_combinations > 10_000_000 {
            return Err(CombinatoricsError::TooLarge(n_combinations));
        }

        Ok(Self { elements, indices: (0..size).collect(), first: true, size })
    }
}

impl<T: Clone> Iterator for Combinations<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            return Some(self.indices.iter().map(|&i| self.elements[i].clone()).collect());
        }

        let len = self.elements.len();
        let mut i = self.size;

        while i > 0 {
            i -= 1;
            if self.indices[i] != i + len - self.size {
                self.indices[i] += 1;
                for j in i + 1..self.size {
                    self.indices[j] = self.indices[j - 1] + 1;
                }
                return Some(self.indices.iter().map(|&i| self.elements[i].clone()).collect());
            }
        }

        None
    }
}

/// Generates all combinations of a given size from a slice of elements.
///
/// This is a helper function that collects the combinations into a vector.
/// For large inputs, consider using the `Combinations` iterator directly.
///
/// # Arguments
///
/// * `elements` - Slice of elements to generate combinations from
/// * `size` - Size of each combination
///
/// # Returns
///
/// A Result containing either a vector of combinations or an error if:
/// - The size exceeds the number of elements
/// - The number of possible combinations is too large
pub fn generate_combinations<T: Clone>(elements: &[T], size: usize) -> Result<Vec<Vec<T>>, CombinatoricsError> {
    Combinations::new(elements.to_vec(), size).map(|iter| iter.collect())
}

/// Generates all subsets up to a given size from a slice of elements.
///
/// # Arguments
///
/// * `elements` - Slice of elements to generate subsets from
/// * `max_size` - Maximum size of subsets to generate (inclusive)
///
/// # Returns
///
/// A vector of all subsets, including the empty set
pub fn generate_subsets<T: Clone>(elements: &[T], max_size: usize) -> Vec<Vec<T>> {
    let mut subsets = vec![vec![]]; // Start with the empty subset
    let max_size = max_size.min(elements.len());

    for size in 1..=max_size {
        if let Ok(combinations) = generate_combinations(elements, size) {
            subsets.extend(combinations);
        } else {
            break; // Stop if we hit the size limit
        }
    }

    subsets
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;

    #[test]
    fn test_combinations_iterator() {
        let elements = vec![1, 2, 3, 4];
        let combinations = Combinations::new(elements.clone(), 2).unwrap();
        let result: Vec<_> = combinations.collect();
        assert_eq!(result.len(), 6); // C(4,2) = 6
        assert!(result.contains(&vec![1, 2]));
        assert!(result.contains(&vec![3, 4]));
    }

    #[test]
    fn test_invalid_combinations() {
        let elements = vec![1, 2, 3];
        assert!(Combinations::new(elements.clone(), 4).is_err());
    }

    #[test]
    fn test_generate_subsets() {
        let _ = env_logger::try_init();

        let v = vec![1, 2, 3];
        let n = 2;
        let subsets = generate_subsets(&v, n);

        // Expected: [], [1], [2], [3], [1,2], [1,3], [2,3]
        assert_eq!(subsets.len(), 7);

        for subset in &subsets {
            debug!("{:?}", subset);
            assert!(subset.len() <= n);
            assert!(subset.iter().all(|x| v.contains(x)));
        }
    }

    #[test]
    fn test_large_input() {
        let v: Vec<_> = (0..100).collect();
        assert!(generate_combinations(&v, 10).is_err());
    }
}
