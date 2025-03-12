use super::persistence::Simplex;
use ordered_float::OrderedFloat;
use std::collections::HashMap;
use std::f64;
use super::combinatorics::generate_subsets;


/// Represents a point in d-dimensional space
#[derive(Debug, Clone)]
pub struct Point {
    pub coords: Vec<f64>,
}


/// Compute Euclidean distance between two points
fn euclidean_distance(p1: &Point, p2: &Point) -> f64 {
    p1.coords
        .iter()
        .zip(&p2.coords)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}


/// Represents a collection of points
#[derive(Debug, Clone)]
pub struct PointCloud {
    pub points: Vec<Point>,
}

impl PointCloud {
    /// Compute pairwise Euclidean distances between points
    pub fn pairwise_distances(&self) -> Vec<Vec<f64>> {
        let n = self.points.len();
        let mut dist_matrix = vec![vec![0.0; n]; n];

        for i in 0..n {
            for j in i + 1..n {
                let dist = euclidean_distance(&self.points[i], &self.points[j]);
                dist_matrix[i][j] = dist;
                dist_matrix[j][i] = dist;
            }
        }

        dist_matrix
    }

    /// Construct a Vietoris-Rips complex up to a given distance threshold
    pub fn vietoris_rips_complex(
        &self,
        max_dimension: usize,
        threshold: f64,
    ) -> (Vec<Simplex>, HashMap<Simplex, f64>) {
        let mut simplices = Vec::new();
        let mut filtration = HashMap::new();
        let dist_matrix = self.pairwise_distances();

        let points: Vec<usize> = (0..self.points.len()).collect();
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
                    .map(|p| OrderedFloat(dist_matrix[p[0]][p[1]]))
                    .max()
                    .unwrap();
            }
            if max_dist <= OrderedFloat(threshold) {
                let simplex = Simplex { vertices: subset };
                simplices.push(simplex.clone());
                filtration.insert(simplex, *max_dist);
            }
        }

        (simplices, filtration)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;

    #[test]
    fn test_triangle() {
        let _ = env_logger::try_init();

        let point_cloud = PointCloud {
            points: vec![
                Point { coords: vec![0.0, 0.0] },
                Point { coords: vec![1.0, 0.0] },
                Point { coords: vec![1.0, 2.0] },
            ],
        };

        let dist_matrix = point_cloud.pairwise_distances();
        let sqrt5 = (5.0 as f64).sqrt();
        let expected = vec![
            vec![0.0, 1.0, sqrt5],
            vec![1.0, 0.0, 2.0],
            vec![sqrt5, 2.0, 0.0],
        ];
        assert_eq!(dist_matrix, expected);

        let (complex, filtration) = point_cloud.vietoris_rips_complex(2, 10.0);
        debug!("Simplicial complex is {:?}", complex);
        debug!("Filtered complex is {:?}", filtration);
        let expected = vec![
            Simplex { vertices: vec![0] },
            Simplex { vertices: vec![1] },
            Simplex { vertices: vec![2] },
            Simplex { vertices: vec![0, 1] },
            Simplex { vertices: vec![0, 2] },
            Simplex { vertices: vec![1, 2] },
            Simplex { vertices: vec![0, 1, 2] },
        ];
        assert_eq!(complex, expected);

        let expected = HashMap::from(
            [
                (Simplex { vertices: vec![0] }, 0.0),
                (Simplex { vertices: vec![1] }, 0.0),
                (Simplex { vertices: vec![2] }, 0.0),
                (Simplex { vertices: vec![0, 1] }, 1.0),
                (Simplex { vertices: vec![0, 2] }, sqrt5),
                (Simplex { vertices: vec![1, 2] }, 2.0),
                (Simplex { vertices: vec![0, 1, 2] }, sqrt5),
            ],
        );
        assert_eq!(filtration, expected);
    }
}
