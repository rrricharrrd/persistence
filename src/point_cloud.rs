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
    use std::collections::{HashMap, HashSet};
    use std::f64::consts::SQRT_2;
    use log::debug;
    use ndarray::array;
    use super::super::homology::ChainComplex;

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
        let all_intervals = complex.persistence_intervals();
        debug!("Persistence intervals {:?}", all_intervals);

        // Then
        // TODO make ordering-agnostic
        let expected: HashMap<usize, Vec<Interval>> = HashMap::from(
            [
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
                    vec![
                        Interval {
                            birth: SQRT_2,
                            birth_chain: HashSet::from([vec![0, 2, 3], vec![0, 1, 2], vec![1, 2, 3], vec![0, 1, 3]]),
                            death: f64::INFINITY,
                            death_chain: HashSet::new(),
                        },
                    ],
                ),
            ],
        );
        for (dim, intervals) in &all_intervals {
            debug!("=== {:?} ===", dim);
            let mut intervals_dim = Vec::new();
            for interval in intervals {
                let birth_chains: HashSet<Vec<usize>> = interval
                    .birth_chain
                    .iter()
                    .map(|c| complex.chain(*c).vertices.clone())
                    .collect();

                let death_chains: HashSet<Vec<usize>>;
                if let Some(death_chain) = &interval.death_chain {
                    death_chains = death_chain
                        .iter()
                        .map(|c| complex.chain(*c).vertices.clone())
                        .collect();
                } else {
                    death_chains = HashSet::new();
                }
                debug!(
                    "Birth: {:?} (level {:?} --- Death: {:?} (level {:?})",
                    birth_chains,
                    interval.birth,
                    death_chains,
                    interval.death
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
