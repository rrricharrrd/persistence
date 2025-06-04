use std::collections::{HashSet, VecDeque};

use super::point_cloud::PointCloud;
use log::debug;
use ndarray::Array2;

pub fn dbscan(points: Array2<f64>, epsilon: f64, min_points: usize) -> Vec<usize> {
    let unhandled_label: usize = usize::MAX;
    let noise_label: usize = 0;
    let mut label = noise_label + 1;
    let mut labels: Vec<usize> = vec![unhandled_label; points.nrows()];
    let point_cloud = PointCloud { points: points.clone() }; // TODO: remove clone
    let distances = point_cloud.pairwise_distances();

    for ix in 0..points.nrows() {
        if labels[ix] != unhandled_label {
            continue; // already processed
        }

        let neighbours: Vec<usize> =
            distances.row(ix).iter().enumerate().filter(|(i, d)| **d < epsilon && *i != ix).map(|(i, _)| i).collect();
        debug!("Neighbours for {} are {:?}", ix, neighbours);
        if neighbours.len() < min_points {
            labels[ix] = noise_label;
            continue;
        }

        labels[ix] = label;

        let mut queue = VecDeque::<usize>::from(neighbours);
        let mut seen = HashSet::<usize>::new();
        while let Some(jx) = queue.pop_front() {
            if labels[jx] == noise_label {
                labels[jx] = label; // convert noise point to boundary point
            }
            if labels[jx] != unhandled_label {
                continue;
            }
            labels[jx] = labels[ix];
            let new_neighbours: Vec<usize> = distances
                .row(jx)
                .iter()
                .enumerate()
                .filter(|(j, d)| **d < epsilon && *j != jx)
                .map(|(j, _)| j)
                .collect();
            debug!("Neighbours for {} from {} are {:?}", jx, ix, new_neighbours);
            if new_neighbours.len() >= min_points {
                for kx in new_neighbours {
                    if seen.insert(kx) {
                        queue.push_back(kx);
                    }
                }
            }
        }
        label += 1;
    }
    labels
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn test_basic() {
        let points =
            array![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [0.0, 2.0], [0.0, 10.0], [1.0, 10.0], [0.0, 11.0], [10.0, 0.0]];
        let result = dbscan(points, 1.5, 2);
        let expected = vec![1, 1, 1, 1, 2, 2, 2, 0];
        debug!("{:?}, {:?}", result, expected);
        assert_eq!(result, expected);
    }
}
