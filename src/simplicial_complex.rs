use super::homology::{Chain, ChainComplex};
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Error types for simplicial complex operations
#[derive(Error, Debug)]
pub enum SimplicialComplexError {
    #[error("Mismatch between number of simplices ({n_simplices}) and filtration levels ({n_levels})")]
    LengthMismatch { n_simplices: usize, n_levels: usize },
    #[error("Invalid simplex: {0}")]
    InvalidSimplex(String),
}

/// A simplex defined by its vertices.
///
/// A simplex is a generalization of a triangle to arbitrary dimensions.
/// For example:
/// - 0-simplex: point (vertex)
/// - 1-simplex: line segment (edge)
/// - 2-simplex: triangle
/// - 3-simplex: tetrahedron
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Simplex {
    /// Vertices that define the simplex, stored in ascending order
    pub vertices: Vec<usize>,
}

impl Simplex {
    /// Creates a new simplex from a list of vertices.
    ///
    /// The vertices will be sorted in ascending order to ensure consistent representation.
    ///
    /// # Arguments
    ///
    /// * `vertices` - List of vertex indices that define the simplex
    pub fn new(mut vertices: Vec<usize>) -> Self {
        vertices.sort_unstable();
        Self { vertices }
    }

    /// Validates that the simplex is well-formed.
    ///
    /// A simplex is well-formed if:
    /// - It has at least one vertex
    /// - All vertices are unique
    fn validate(&self) -> Result<(), SimplicialComplexError> {
        if self.vertices.is_empty() {
            return Err(SimplicialComplexError::InvalidSimplex("Simplex must have at least one vertex".to_string()));
        }

        // Check for duplicate vertices
        let mut seen = HashSet::new();
        for &v in &self.vertices {
            if !seen.insert(v) {
                return Err(SimplicialComplexError::InvalidSimplex(format!("Duplicate vertex {} in simplex", v)));
            }
        }
        Ok(())
    }
}

impl Chain for Simplex {
    fn dim(&self) -> usize {
        self.vertices.len() - 1
    }
}

/// A simplicial complex with an associated filtration.
///
/// A simplicial complex is a collection of simplices that satisfy the closure property:
/// if a simplex is in the complex, all of its faces must also be in the complex.
///
/// The filtration assigns a "time of appearance" to each simplex, creating a sequence
/// of nested simplicial complexes.
#[derive(Debug, Clone)]
pub struct SimplicialComplex {
    /// Simplices in the complex, ordered by filtration level
    pub simplices: Vec<Simplex>,
    /// Filtration levels corresponding to each simplex
    pub levels: Vec<f64>,
    /// Map from simplex to its index in the complex
    indexes: HashMap<Simplex, usize>,
}

impl SimplicialComplex {
    /// Creates a new simplicial complex with the given simplices and filtration levels.
    ///
    /// # Arguments
    ///
    /// * `simplices` - Vector of simplices in the complex
    /// * `levels` - Vector of filtration levels, one for each simplex
    ///
    /// # Returns
    ///
    /// A Result containing either the new SimplicialComplex or an error if:
    /// - The number of simplices doesn't match the number of levels
    /// - Any simplex is invalid
    pub fn new(simplices: Vec<Simplex>, levels: Vec<f64>) -> Result<Self, SimplicialComplexError> {
        if simplices.len() != levels.len() {
            return Err(SimplicialComplexError::LengthMismatch {
                n_simplices: simplices.len(),
                n_levels: levels.len(),
            });
        }

        // Validate all simplices
        for simplex in &simplices {
            simplex.validate()?;
        }

        // Order simplices by filtration level
        let mut paired: Vec<_> = simplices.into_iter().zip(levels).collect();
        paired.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let (sorted_simplices, sorted_levels): (Vec<_>, Vec<_>) = paired.into_iter().unzip();

        // Construct mapping to be able to access "ith simplex"
        let indexes: HashMap<Simplex, usize> =
            sorted_simplices.iter().enumerate().map(|(i, s)| (s.clone(), i)).collect();

        Ok(Self { simplices: sorted_simplices, levels: sorted_levels, indexes })
    }

    /// Returns the index of a simplex in the complex, if it exists.
    pub fn index_of(&self, simplex: &Simplex) -> Option<usize> {
        self.indexes.get(simplex).copied()
    }
}

impl ChainComplex<Simplex> for SimplicialComplex {
    fn chain(&self, index: usize) -> &Simplex {
        &self.simplices[index]
    }

    fn chains(&self) -> &Vec<Simplex> {
        &self.simplices
    }

    fn filtration_level(&self, index: usize) -> OrderedFloat<f64> {
        OrderedFloat(self.levels[index])
    }

    fn boundary(&self, index: usize) -> HashSet<usize> {
        let s = &self.simplices[index];
        let mut bounds = HashSet::new();

        // For each vertex in the simplex, create a face by removing that vertex
        for &v in &s.vertices {
            let mut face_vertices = s.vertices.clone();
            face_vertices.retain(|x| *x != v);

            if !face_vertices.is_empty() {
                let face = Simplex::new(face_vertices);
                if let Some(face_index) = self.index_of(&face) {
                    bounds.insert(face_index);
                }
            }
        }
        bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;

    #[test]
    fn test_simplex_validation() {
        assert!(Simplex::new(vec![0]).validate().is_ok());
        assert!(Simplex::new(vec![]).validate().is_err());
        assert!(Simplex::new(vec![0, 0]).validate().is_err());
    }

    #[test]
    fn test_simplicial_complex_creation() {
        let result = SimplicialComplex::new(vec![Simplex::new(vec![])], vec![0.0]);
        assert!(result.is_err());

        let result = SimplicialComplex::new(vec![Simplex::new(vec![0])], vec![0.0, 1.0]);
        assert!(result.is_err());

        let result = SimplicialComplex::new(vec![Simplex::new(vec![0]), Simplex::new(vec![0])], vec![0.0]);
        assert!(result.is_err());
    }

    #[derive(Debug, PartialEq)]
    struct Interval {
        birth: f64,
        birth_chain: HashSet<Vec<usize>>,
        death: f64,
        death_chain: HashSet<Vec<usize>>,
    }

    #[test]
    fn test_simplicial_complex() {
        let _ = env_logger::try_init();

        let simplex0 = Simplex::new(vec![0]);
        let simplex1 = Simplex::new(vec![1]);
        let simplex01 = Simplex::new(vec![0, 1]);
        let levels = vec![0.0, 1.0, 1.0];
        let complex = SimplicialComplex::new(vec![simplex0, simplex1, simplex01.clone()], levels).unwrap();

        assert_eq!(simplex01.dim(), 1);
        assert_eq!(complex.len(), 3);
        debug!("{:?}", complex);

        for i in 0..complex.len() {
            let boundary = complex.boundary(i);
            debug!("d.{:?} is {:?}", complex.chain(i), boundary);
        }
        debug!("Boundary: {:?}", complex.boundary_matrix());
    }

    #[test]
    fn test_persistence_intervals_paper_example() {
        let _ = env_logger::try_init();

        // Given
        let simplices = vec![
            Simplex { vertices: vec![0] },
            Simplex { vertices: vec![1] },
            Simplex { vertices: vec![2] },
            Simplex { vertices: vec![3] },
            Simplex { vertices: vec![0, 1] },
            Simplex { vertices: vec![1, 2] },
            Simplex { vertices: vec![2, 3] },
            Simplex { vertices: vec![0, 3] },
            Simplex { vertices: vec![0, 2] },
            Simplex { vertices: vec![0, 1, 2] },
            Simplex { vertices: vec![0, 2, 3] },
        ];
        let levels = vec![0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(simplices.len(), levels.len());

        let complex = SimplicialComplex::new(simplices, levels).unwrap();

        // When
        let all_intervals = complex.persistence_intervals();
        debug!("Result {:?}", all_intervals);

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
                        birth: 1.0,
                        birth_chain: HashSet::from([vec![2]]),
                        death: 1.0,
                        death_chain: HashSet::from([vec![1, 2]]),
                    },
                    Interval {
                        birth: 1.0,
                        birth_chain: HashSet::from([vec![3]]),
                        death: 2.0,
                        death_chain: HashSet::from([vec![2, 3]]),
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
                        birth: 3.0,
                        birth_chain: HashSet::from([vec![0, 2], vec![0, 1], vec![1, 2]]),
                        death: 4.0,
                        death_chain: HashSet::from([vec![0, 1, 2]]),
                    },
                    Interval {
                        birth: 2.0,
                        birth_chain: HashSet::from([vec![0, 3], vec![0, 1], vec![1, 2], vec![2, 3]]),
                        death: 5.0,
                        death_chain: HashSet::from([vec![0, 1, 2], vec![0, 2, 3]]),
                    },
                ],
            ),
            (2, vec![]),
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
