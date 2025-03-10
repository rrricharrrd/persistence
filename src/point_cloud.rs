use super::persistence::Simplex;
use ordered_float::OrderedFloat;
use std::collections::HashMap;


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
    pub fn vietoris_rips_complex(&self, threshold: f64) -> (Vec<Simplex>, HashMap<Simplex, f64>) {
        let threshold = OrderedFloat(threshold);
        let mut simplices = Vec::new();
        let mut filtration = HashMap::new();
        let n = self.points.len();
        let dist_matrix = self.pairwise_distances();

        // Add 0-simplices (individual points)
        for i in 0..n {
            let simplex = Simplex { vertices: vec![i] };
            simplices.push(simplex.clone());
            filtration.insert(simplex, 0.0);
        }

        // Add 1-simplices (edges)
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            for j in i + 1..n {
                if OrderedFloat(dist_matrix[i][j]) <= threshold {
                    let simplex = Simplex{vertices: vec![i,j] };
                    simplices.push(simplex.clone());
                    filtration.insert(simplex,  dist_matrix[i][j]);
                }
            }
        }

        // Add 2-simplices (triangles)
        for i in 0..n {
            for j in i + 1..n {
                for k in j + 1..n {
                    let d = [dist_matrix[i][j], dist_matrix[i][k], dist_matrix[j][k]]
                        .iter()
                        .map(|x| OrderedFloat(*x))
                        .max()
                        .unwrap();
                    if d <= threshold {
                        let simplex = Simplex { vertices: vec![i, j, k] };
                        simplices.push(simplex.clone());
                        filtration.insert(simplex, *d);
                    }
                }
            }
        }

        // Add 3-simplices (tetrahedra)
        for i in 0..n {
            for j in i + 1..n {
                for k in j + 1..n {
                    for l in k + 1..n {
                        let d = [
                            dist_matrix[i][j],
                            dist_matrix[i][k],
                            dist_matrix[i][l],
                            dist_matrix[j][k],
                            dist_matrix[j][l],
                            dist_matrix[k][l],
                        ].iter()
                            .map(|x| OrderedFloat(*x))
                            .max()
                            .unwrap();
                        if d <= threshold {
                            let simplex = Simplex { vertices: vec![i, j, k, l] };
                            simplices.push(simplex.clone());
                            filtration.insert(simplex, *d);
                        }
                    }
                }
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
    fn test_distance() {
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

        let (complex, filtration) = point_cloud.vietoris_rips_complex(10.0);
        debug!("Simplicial complex is {:?}", complex); // TODO more
        debug!("Filtered complex is {:?}", filtration); // TODO more
        // TODO make ordering-agnostic
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
