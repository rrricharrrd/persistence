use std::collections::HashSet;


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
    pub fn vietoris_rips_complex(&self, threshold: f64) -> Vec<HashSet<usize>> {
        let mut simplices = Vec::new();
        let n = self.points.len();
        let dist_matrix = self.pairwise_distances();

        // Add 0-simplices (individual points)
        for i in 0..n {
            simplices.push(HashSet::from([i]));
        }

        // Add 1-simplices (edges)
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            for j in i + 1..n {
                if dist_matrix[i][j] <= threshold {
                    simplices.push(HashSet::from([i, j]));
                }
            }
        }

        // Add 2-simplices (triangles)
        for i in 0..n {
            for j in i + 1..n {
                for k in j + 1..n {
                    if dist_matrix[i][j] <= threshold && dist_matrix[i][k] <= threshold &&
                        dist_matrix[j][k] <= threshold
                    {
                        simplices.push(HashSet::from([i, j, k]));
                    }
                }
            }
        }

        // Add 3-simplices (tetrahedra)
        for i in 0..n {
            for j in i + 1..n {
                for k in j + 1..n {
                    for l in k + 1..n {
                        if dist_matrix[i][j] <= threshold && dist_matrix[i][k] <= threshold &&
                            dist_matrix[i][l] <= threshold &&
                            dist_matrix[j][k] <= threshold &&
                            dist_matrix[j][l] <= threshold &&
                            dist_matrix[k][l] <= threshold
                        {
                            simplices.push(HashSet::from([i, j, k, l]));
                        }
                    }
                }
            }
        }

        simplices
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        let point_cloud = PointCloud {
            points: vec![
                Point { coords: vec![0.0, 0.0] },
                Point { coords: vec![1.0, 0.0] },
                Point { coords: vec![1.0, 1.0] },
            ],
        };

        let dist_matrix = point_cloud.pairwise_distances();
        let sqrt2 = (2.0 as f64).sqrt();
        let expected = vec![
            vec![0.0, 1.0, sqrt2],
            vec![1.0, 0.0, 1.0],
            vec![sqrt2, 1.0, 0.0],
        ];
        assert_eq!(dist_matrix, expected);

        let _complex = point_cloud.vietoris_rips_complex(1.0); // TODO more
    }
}
