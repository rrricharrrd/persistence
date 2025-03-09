use super::persistence::Simplex;


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
    pub fn vietoris_rips_complex(&self, threshold: f64) -> Vec<Simplex> {
        let mut simplices = Vec::new();
        let n = self.points.len();
        let dist_matrix = self.pairwise_distances();

        // Add 0-simplices (individual points)
        for i in 0..n {
            simplices.push(Simplex {
                vertices: vec![i],
                filtration_level: 0,
            });
        }

        // Add 1-simplices (edges)
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            for j in i + 1..n {
                if dist_matrix[i][j] <= threshold {
                    simplices.push(
                        Simplex{
                            vertices: vec![i,j],
                            filtration_level: dist_matrix[i][j] as usize // TODO float vs int
                        }
                    );
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
                        let d = [dist_matrix[i][j], dist_matrix[i][k], dist_matrix[j][k]]
                            .iter()
                            .map(|x| *x as usize)   // TODO float vs int
                            .min()
                            .unwrap();
                        simplices.push(Simplex {
                            vertices: vec![i, j, k],
                            filtration_level: d,
                        });
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
                            let d = [
                                dist_matrix[i][j],
                                dist_matrix[i][k],
                                dist_matrix[i][l],
                                dist_matrix[j][k],
                                dist_matrix[j][l],
                                dist_matrix[k][l],
                            ].iter()
                                .map(|x| *x as usize)   // TODO float vs int
                                .min()
                                .unwrap();
                            simplices.push(Simplex {
                                vertices: vec![i, j, k, l],
                                filtration_level: d,
                            });
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
    use log::debug;

    #[test]
    fn test_distance() {
        let _ = env_logger::try_init();

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

        let complex = point_cloud.vietoris_rips_complex(1.0);
        debug!("Simplicial complex is {:?}", complex); // TODO more
    }
}
