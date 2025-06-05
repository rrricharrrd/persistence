use std::collections::HashMap;

use super::combinatorics::generate_products;
use super::dbscan::dbscan;
use ndarray::{stack, Array2, Axis};
use thiserror::Error;

/// Error types for Mapper
#[derive(Error, Debug)]
pub enum MapperError {
    #[error("Empty points")]
    EmptyPoints,
}

#[allow(dead_code)]
struct Node<T> {
    data: T,
    adjacencies: Vec<Node<T>>,
}

fn select_rows(data: &Array2<f64>, indices: &[usize]) -> Array2<f64> {
    let views: Vec<_> = indices.iter().map(|&i| data.slice(ndarray::s![i, ..])).collect();

    stack(Axis(0), &views).expect("Failed to stack selected rows")
}

pub fn mapper(points: Array2<f64>) -> Result<Vec<usize>, MapperError> {
    if points.is_empty() {
        return Err(MapperError::EmptyPoints);
    }

    let mins: Vec<f64> = points
        .axis_iter(Axis(1)) // iterate over columns
        .map(|col| col.iter().cloned().fold(f64::INFINITY, f64::min))
        .collect();
    let maxs: Vec<f64> = points
        .axis_iter(Axis(1)) // iterate over columns
        .map(|col| col.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
        .collect();
    println!("Minmaxes {:?} {:?}", mins, maxs);

    let n_divisions = 3; // TODO make configurable
    let overlap = 0.1; // TODO make configurable
    let epsilon = 0.1; // TODO make configurable
    let min_points = 2; // TODO make configurable

    let slots: Vec<usize> = (0..n_divisions).collect();
    let box_indices = generate_products(&slots, points.ncols());
    let mut overlaps: HashMap<usize, Vec<usize>> = HashMap::new();
    for indices in box_indices.unwrap() {
        // TODO
        let box_mins: Vec<f64> = mins
            .iter()
            .zip(maxs.iter())
            .zip(indices.iter())
            .map(|((&lo, &hi), &ix)| lo - overlap / 2. + ((ix as f64) / (n_divisions as f64)) * (hi - lo))
            .collect();
        let box_maxs: Vec<f64> = mins
            .iter()
            .zip(maxs.iter())
            .zip(indices.iter())
            .map(|((&lo, &hi), &ix)| lo + overlap / 2. + (((ix + 1) as f64) / (n_divisions as f64)) * (hi - lo))
            .collect();

        let box_points: Vec<usize> = points
            .axis_iter(Axis(0))
            .enumerate()
            .filter(|(_, pt)| {
                box_mins.iter().zip(box_maxs.iter()).zip(pt.iter()).all(|((&lo, &hi), &x)| lo <= x && x <= hi)
            })
            .map(|(ix, _)| ix)
            .collect();
        println!("Box indices {:?} {:?} {:?} {:?}", indices.clone(), box_mins, box_maxs, box_points);
        if box_points.is_empty() {
            continue;
        }
        let clustering = dbscan(select_rows(&points, &box_points), epsilon, min_points).unwrap();
        for (ix, &label) in clustering.iter().enumerate() {
            if label != 0 {
                // TODO ignore noise?
                let _is_new = match overlaps.entry(ix) {
                    std::collections::hash_map::Entry::Occupied(mut entry) => {
                        entry.get_mut().push(label);
                        false
                    },
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(vec![label]);
                        true
                    },
                };
            }
        }
    }
    Ok(vec![1, 2, 3])
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;
    use ndarray::{array, Array2};

    #[test]
    fn test_empty() {
        let points = Array2::<f64>::zeros((0, 0));
        let result = mapper(points);
        assert!(result.is_err());
    }

    #[test]
    fn test_basic() {
        let points = array![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0], [0.0, 2.0], [0.0, 3.0]];
        let result = mapper(points).unwrap();
        let expected = vec![1, 1, 1, 1, 2, 2, 2, 0];
        debug!("{:?}, {:?}", result, expected);
        // assert_eq!(result, expected);
    }
}
