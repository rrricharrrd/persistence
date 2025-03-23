use ordered_float::OrderedFloat;
use std::f64;
use super::combinatorics::generate_subsets;
use super::simplicial_complex::{SimplicialComplex, Simplex};
use ndarray::{Array2, ArrayView1};


/// Represents a point in d-dimensional space
#[derive(Debug, Clone)]
pub struct Point {
    pub coords: Vec<f64>,
}


/// Compute Euclidean distance between two points
fn euclidean_distance(point1: ArrayView1<f64>, point2: ArrayView1<f64>) -> f64 {
    point1
        .iter()
        .zip(&point2)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}


/// Represents a collection of points
#[derive(Debug, Clone)]
pub struct PointCloud {
    pub points: Array2<f64>,
}

impl PointCloud {
    /// Number of points in cloud
    pub fn n_points(&self) -> usize {
        self.points.nrows()
    }

    /// Dimensionality of cloud
    pub fn dim(&self) -> usize {
        self.points.ncols()
    }

    /// Compute pairwise Euclidean distances between points
    pub fn pairwise_distances(&self) -> Array2<f64> {
        let n = self.n_points();
        let mut dist_matrix = Array2::<f64>::zeros((n, n));

        for i in 0..n {
            for j in i + 1..n {
                let dist = euclidean_distance(self.points.row(i), self.points.row(j));
                dist_matrix[(i, j)] = dist;
                dist_matrix[(j, i)] = dist;
            }
        }

        dist_matrix
    }

    /// Construct a Vietoris-Rips complex up to a given distance threshold
    pub fn vietoris_rips_complex(&self, max_dimension: usize, threshold: f64) -> SimplicialComplex {
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
                let pairs = generate_subsets(&subset, 2);
                max_dist = pairs
                    .iter()
                    .filter(|p| p.len() == 2)
                    .map(|p| OrderedFloat(dist_matrix[[p[0], p[1]]]))
                    .max()
                    .unwrap();
            }
            if max_dist <= OrderedFloat(threshold) {
                let simplex = Simplex { vertices: subset };
                simplices.push(simplex.clone());
                filtration.push(*max_dist);
            }
        }

        SimplicialComplex::new(simplices, filtration)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::f64::consts::SQRT_2;
    use log::debug;
    use ndarray::array;
    use super::super::homology::PersistenceInterval;
    use super::super::homology::ChainComplex;

    #[test]
    fn test_triangle() {
        let _ = env_logger::try_init();

        let point_cloud = PointCloud {
            points: array![
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 2.0],
            ],
        };

        let dist_matrix = point_cloud.pairwise_distances();
        let sqrt5 = (5.0 as f64).sqrt();
        let expected =
            array![
            [0.0, 1.0, sqrt5],
            [1.0, 0.0, 2.0],
            [sqrt5, 2.0, 0.0],
        ];
        assert_eq!(dist_matrix, expected);

        let complex = point_cloud.vietoris_rips_complex(2, 10.0);
        debug!("Simplicial complex of {:?}", complex.simplices);
        debug!("Filtration {:?}", complex.levels);
        let expected = vec![
            Simplex { vertices: vec![0] },
            Simplex { vertices: vec![1] },
            Simplex { vertices: vec![2] },
            Simplex { vertices: vec![0, 1] },
            Simplex { vertices: vec![1, 2] },
            Simplex { vertices: vec![0, 2] },
            Simplex { vertices: vec![0, 1, 2] },
        ];
        assert_eq!(complex.simplices, expected);

        let expected = vec![0.0, 0.0, 0.0, 1.0, 2.0, sqrt5, sqrt5];
        assert_eq!(complex.levels, expected);
    }

    #[test]
    fn test_square() {
        let _ = env_logger::try_init();

        // Given
        let point_cloud = PointCloud {
            points: array![
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
            ],
        };

        // When
        let complex = point_cloud.vietoris_rips_complex(2, 10.0);
        debug!("Simplicial complex {:?}", complex);
        let intervals = complex.persistence_intervals();
        debug!("Persistence intervals {:?}", intervals);

        // Then
        // TODO This is not correct!!!
        let expected: HashMap<usize, Vec<PersistenceInterval>> = HashMap::from(
            [
                (
                    0,
                    vec![
                        PersistenceInterval {
                            birth: 0.0,
                            death: 1.0,
                        },
                        PersistenceInterval {
                            birth: 0.0,
                            death: 1.0,
                        },
                        PersistenceInterval {
                            birth: 0.0,
                            death: 1.0,
                        },
                        PersistenceInterval {
                            birth: 0.0,
                            death: f64::INFINITY,
                        },
                    ],
                ),
                (
                    1,
                    vec![
                        PersistenceInterval {
                            birth: SQRT_2,
                            death: SQRT_2,
                        },
                        PersistenceInterval {
                            birth: SQRT_2,
                            death: SQRT_2,
                        },
                        PersistenceInterval {
                            birth: 1.0,
                            death: SQRT_2,
                        },
                    ],
                ),
                (
                    2,
                    vec![
                        PersistenceInterval {
                            birth: SQRT_2,
                            death: f64::INFINITY,
                        },
                    ],
                ),
            ],
        );
        assert_eq!(intervals, expected)
    }
}
