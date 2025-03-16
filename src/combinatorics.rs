fn generate_combinations<T: Clone + std::fmt::Debug>(
    v: &[T],
    size: usize,
    start: usize,
    current: &mut Vec<T>,
    result: &mut Vec<Vec<T>>,
) {
    if current.len() == size {
        result.push(current.clone());
        return;
    }

    for i in start..v.len() {
        current.push(v[i].clone());
        generate_combinations(v, size, i + 1, current, result);
        current.pop();
    }
}


pub fn generate_subsets<T: Clone + std::fmt::Debug>(v: &[T], n: usize) -> Vec<Vec<T>> {
    let mut subsets = vec![vec![]]; // Start with the empty subset

    for size in 1..n + 1 {
        let mut new_subsets = Vec::new();
        generate_combinations(v, size, 0, &mut vec![], &mut new_subsets);
        subsets.extend(new_subsets);
    }

    subsets
}


#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;

    #[test]
    fn test_generate_subsets() {
        let _ = env_logger::try_init();

        let v = vec![1, 2, 3, 4, 5];
        let n = 3;
        let subsets = generate_subsets(&v, n);
        assert_eq!(subsets.len(), 1 + 5 + 5 * 4 / 2 + 5 * 4 * 3 / (3 * 2));
        for subset in subsets {
            debug!("{:?}", subset);
        }
    }
}
