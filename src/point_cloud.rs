use super::combinatorics::generate_subsets;
use super::simplicial_complex::{Simplex, SimplicialComplex};
use ndarray::{Array2, ArrayView1};
use ordered_float::OrderedFloat;
use rayon::prelude::*;
use thiserror::Error;

/// Error types for point cloud operations
#[derive(Error, Debug)]
pub enum PointCloudError {
    #[error("Empty point cloud")]
    EmptyCloud,
    #[error("Inconsistent point dimensions: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
    #[error("Invalid dimension: {0}")]
    InvalidDimension(String),
}

/// Represents a point in d-dimensional space.
///
/// Points are represented by their coordinates in Euclidean space.
#[derive(Debug, Clone)]
pub struct Point {
    /// Coordinates of the point in d-dimensional space
    pub coords: Vec<f64>,
}

impl Point {
    /// Creates a new point from a vector of coordinates.
    ///
    /// # Arguments
    ///
    /// * `coords` - Vector of coordinates in d-dimensional space
    pub fn new(coords: Vec<f64>) -> Self {
        Self { coords }
    }

    /// Returns the dimension of the point.
    pub fn dim(&self) -> usize {
        self.coords.len()
    }
}

/// Compute Euclidean distance between two points.
///
/// # Arguments
///
/// * `point1` - First point
/// * `point2` - Second point
///
/// # Returns
///
/// The Euclidean distance between the points
fn euclidean_distance(point1: ArrayView1<f64>, point2: ArrayView1<f64>) -> f64 {
    point1.iter().zip(point2.iter()).map(|(a, b)| (a - b).powi(2)).sum::<f64>().sqrt()
}

/// Represents a collection of points in d-dimensional space.
///
/// The points are stored in a matrix where each row represents a point
/// and each column represents a dimension.
#[derive(Debug, Clone)]
pub struct PointCloud {
    /// Matrix of points where each row is a point and each column is a dimension
    pub points: Array2<f64>,
}

impl PointCloud {
    /// Creates a new point cloud from a matrix of points.
    ///
    /// # Arguments
    ///
    /// * `points` - Matrix where each row is a point and each column is a dimension
    ///
    /// # Returns
    ///
    /// A Result containing either the new PointCloud or an error if:
    /// - The point cloud is empty
    /// - The points have inconsistent dimensions
    pub fn new(points: Array2<f64>) -> Result<Self, PointCloudError> {
        if points.is_empty() {
            return Err(PointCloudError::EmptyCloud);
        }
        Ok(Self { points })
    }

    /// Returns the number of points in the cloud.
    pub fn n_points(&self) -> usize {
        self.points.nrows()
    }

    /// Returns the dimensionality of the points.
    pub fn dim(&self) -> usize {
        self.points.ncols()
    }

    /// Compute pairwise Euclidean distances between points.
    ///
    /// This implementation uses parallel processing for large point clouds
    /// and exploits symmetry to compute only half of the distances.
    ///
    /// # Returns
    ///
    /// A symmetric matrix of pairwise distances
    pub fn pairwise_distances(&self) -> Array2<f64> {
        let n = self.n_points();
        let dist_matrix = std::sync::Arc::new(std::sync::Mutex::new(Array2::<f64>::zeros((n, n))));

        (0..n).into_par_iter().for_each(|i| {
            let local_dists: Vec<(usize, usize, f64)> = (i + 1..n)
                .map(|j| {
                    let dist = euclidean_distance(self.points.row(i), self.points.row(j));
                    (i, j, dist)
                })
                .collect();

            let mut matrix = dist_matrix.lock().unwrap();
            for (i, j, dist) in local_dists {
                matrix[[i, j]] = dist;
                matrix[[j, i]] = dist;
            }
        });

        std::sync::Arc::try_unwrap(dist_matrix).unwrap().into_inner().unwrap()
    }

    /// Construct a Vietoris-Rips complex up to a given distance threshold.
    ///
    /// The Vietoris-Rips complex is a simplicial complex built from a point cloud
    /// where simplices are formed whenever all pairwise distances between points
    /// are less than or equal to the threshold.
    ///
    /// # Arguments
    ///
    /// * `max_dimension` - Maximum dimension of simplices to include
    /// * `threshold` - Distance threshold for forming simplices
    ///
    /// # Returns
    ///
    /// A Result containing either the SimplicialComplex or an error if:
    /// - The maximum dimension is invalid
    /// - Any simplex is invalid
    pub fn vietoris_rips_complex(
        &self,
        max_dimension: usize,
        threshold: f64,
    ) -> Result<SimplicialComplex, PointCloudError> {
        if max_dimension >= self.n_points() {
            return Err(PointCloudError::InvalidDimension(format!(
                "Maximum dimension {} exceeds number of points {}",
                max_dimension,
                self.n_points()
            )));
        }

        let mut simplices = Vec::new();
        let mut filtration = Vec::new();
        let dist_matrix = self.pairwise_distances();

        let points: Vec<usize> = (0..self.n_points()).collect();
        let subsets = generate_subsets(&points, max_dimension + 1);

        for subset in subsets {
            let max_dist: OrderedFloat<f64>;
            if subset.is_empty() {
                continue;
            } else if subset.len() == 1 {
                max_dist = OrderedFloat(0.0);
            } else {
                // Get all pairs of vertices in the simplex
                let pairs = generate_subsets(&subset, 2);
                max_dist = pairs
                    .iter()
                    .filter(|p| p.len() == 2)
                    .map(|p| OrderedFloat(dist_matrix[[p[0], p[1]]]))
                    .max()
                    .unwrap();
            }

            if max_dist <= OrderedFloat(threshold) {
                simplices.push(Simplex::new(subset));
                filtration.push(*max_dist);
            }
        }

        SimplicialComplex::new(simplices, filtration).map_err(|e| PointCloudError::InvalidDimension(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::homology::ChainComplex;
    use log::debug;
    use ndarray::array;
    use std::collections::{HashMap, HashSet};
    use std::f64::consts::SQRT_2;

    #[test]
    fn test_point_cloud_creation() {
        let result = PointCloud::new(Array2::<f64>::zeros((0, 3)));
        assert!(result.is_err());

        let points = array![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
        let cloud = PointCloud::new(points).unwrap();
        assert_eq!(cloud.n_points(), 3);
        assert_eq!(cloud.dim(), 2);
    }

    #[derive(Debug, PartialEq)]
    struct Interval {
        birth: f64,
        birth_chain: HashSet<Vec<usize>>,
        death: f64,
        death_chain: HashSet<Vec<usize>>,
    }

    #[test]
    fn test_triangle() {
        let _ = env_logger::try_init();

        let point_cloud = PointCloud::new(array![[0.0, 0.0], [1.0, 0.0], [1.0, 2.0],]).unwrap();

        let dist_matrix = point_cloud.pairwise_distances();
        let sqrt5 = (5.0 as f64).sqrt();
        let expected = array![[0.0, 1.0, sqrt5], [1.0, 0.0, 2.0], [sqrt5, 2.0, 0.0],];
        assert_eq!(dist_matrix, expected);

        let complex = point_cloud.vietoris_rips_complex(2, 10.0).unwrap();
        debug!("Simplicial complex of {:?}", complex.simplices);
        debug!("Filtration {:?}", complex.levels);
        let expected = vec![
            Simplex::new(vec![0]),
            Simplex::new(vec![1]),
            Simplex::new(vec![2]),
            Simplex::new(vec![0, 1]),
            Simplex::new(vec![1, 2]),
            Simplex::new(vec![0, 2]),
            Simplex::new(vec![0, 1, 2]),
        ];
        assert_eq!(complex.simplices, expected);

        let expected = vec![0.0, 0.0, 0.0, 1.0, 2.0, sqrt5, sqrt5];
        assert_eq!(complex.levels, expected);
    }

    #[test]
    fn test_square() {
        let _ = env_logger::try_init();

        // Given
        let point_cloud = PointCloud { points: array![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],] };

        // When
        let complex = point_cloud.vietoris_rips_complex(2, 10.0).unwrap();
        debug!("Simplicial complex {:?}", complex);
        let all_intervals = complex.persistence_intervals();
        debug!("Persistence intervals {:?}", all_intervals);

        // Then
        // TODO make ordering-agnostic
        let expected: HashMap<usize, Vec<Interval>> = HashMap::from([
            (
                0,
                vec![
                    Interval {
                        birth: 0.0,
                        birth_chain: HashSet::from([vec![1]]),
                        death: 1.0,
                        death_chain: HashSet::from([vec![0, 1]]),
                    },
                    Interval {
                        birth: 0.0,
                        birth_chain: HashSet::from([vec![3]]),
                        death: 1.0,
                        death_chain: HashSet::from([vec![0, 3]]),
                    },
                    Interval {
                        birth: 0.0,
                        birth_chain: HashSet::from([vec![2]]),
                        death: 1.0,
                        death_chain: HashSet::from([vec![1, 2]]),
                    },
                    Interval {
                        birth: 0.0,
                        birth_chain: HashSet::from([vec![0]]),
                        death: f64::INFINITY,
                        death_chain: HashSet::new(),
                    },
                ],
            ),
            (
                1,
                vec![
                    Interval {
                        birth: SQRT_2,
                        birth_chain: HashSet::from([vec![0, 2], vec![0, 1], vec![1, 2]]),
                        death: SQRT_2,
                        death_chain: HashSet::from([vec![0, 1, 2]]),
                    },
                    Interval {
                        birth: SQRT_2,
                        birth_chain: HashSet::from([vec![0, 1], vec![1, 3], vec![0, 3]]),
                        death: SQRT_2,
                        death_chain: HashSet::from([vec![0, 1, 3]]),
                    },
                    Interval {
                        birth: 1.0,
                        birth_chain: HashSet::from([vec![2, 3], vec![0, 1], vec![1, 2], vec![0, 3]]),
                        death: SQRT_2,
                        death_chain: HashSet::from([vec![0, 1, 2], vec![0, 2, 3]]),
                    },
                ],
            ),
            (
                2,
                vec![Interval {
                    birth: SQRT_2,
                    birth_chain: HashSet::from([vec![0, 2, 3], vec![0, 1, 2], vec![1, 2, 3], vec![0, 1, 3]]),
                    death: f64::INFINITY,
                    death_chain: HashSet::new(),
                }],
            ),
        ]);
        for (dim, intervals) in &all_intervals {
            debug!("=== {:?} ===", dim);
            let mut intervals_dim = Vec::new();
            for interval in intervals {
                let birth_chains: HashSet<Vec<usize>> =
                    interval.birth_chain.iter().map(|c| complex.chain(*c).vertices.clone()).collect();

                let death_chains: HashSet<Vec<usize>>;
                if let Some(death_chain) = &interval.death_chain {
                    death_chains = death_chain.iter().map(|c| complex.chain(*c).vertices.clone()).collect();
                } else {
                    death_chains = HashSet::new();
                }
                debug!(
                    "Birth: {:?} (level {:?} --- Death: {:?} (level {:?})",
                    birth_chains, interval.birth, death_chains, interval.death
                );
                intervals_dim.push(Interval {
                    birth: interval.birth,
                    birth_chain: birth_chains,
                    death: interval.death,
                    death_chain: death_chains,
                });
            }
            assert_eq!(intervals_dim, expected[dim]);
        }
    }
}
