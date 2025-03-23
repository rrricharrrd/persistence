use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};
use super::homology::{Chain, ChainComplex};

/// Simplex defined via collection of vertices
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Simplex {
    pub vertices: Vec<usize>,
}

impl Chain for Simplex {
    fn dim(&self) -> usize {
        self.vertices.len() - 1
    }
}

/// Simplicial chain complex
#[derive(Debug, Clone)]
pub struct SimplicialComplex {
    pub simplices: Vec<Simplex>,
    pub levels: Vec<f64>,
    indexes: HashMap<Simplex, usize>,
}

impl SimplicialComplex {
    pub fn new(simplices: Vec<Simplex>, levels: Vec<f64>) -> Self {
        // TODO check lengths match
        let indexes: HashMap<Simplex, usize> = simplices
            .iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), i))
            .collect();
        Self {
            simplices: simplices.clone(),
            levels,
            indexes,
        }
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
        // Note working over Z/2
        let s = &self.simplices[index];
        let mut bounds = HashSet::new();
        for v in s.vertices.clone() {
            let mut ds_vertices = s.vertices.clone();
            ds_vertices.retain(|&x| x != v); // Remove vertex
            if !ds_vertices.is_empty() {
                let ds = Simplex { vertices: ds_vertices };
                bounds.insert(*self.indexes.get(&ds).unwrap());
            }
        }
        bounds
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::homology::PersistenceInterval;
    use log::debug;

    #[test]
    fn test_simplicial_complex() {
        let _ = env_logger::try_init();

        let simplex0 = Simplex { vertices: vec![0] };
        let simplex1 = Simplex { vertices: vec![1] };
        let simplex01 = Simplex { vertices: vec![0, 1] };
        let levels = vec![0.0, 1.0, 1.0];
        let complex = SimplicialComplex::new(vec![simplex0, simplex1, simplex01.clone()], levels);
        assert_eq!(simplex01.clone().dim(), 1);
        assert_eq!(complex.len(), 3);
        debug!("{:?}", complex);

        for i in 0..complex.len() {
            let boundary = complex.boundary(i);
            debug!("d.{:?} is {:?}", complex.chain(i), boundary);
        }
        debug!("Boundary: {:?}", complex.boundary_matrix());
    }

    #[test]
    fn test_persistence_intervals() {
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

        let complex = SimplicialComplex::new(simplices, levels);

        // When
        let result = complex.persistence_intervals();
        debug!("Result {:?}", result);

        // Then
        // TODO make ordering-agnostic
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
                            birth: 1.0,
                            death: 1.0,
                        },
                        PersistenceInterval {
                            birth: 1.0,
                            death: 2.0,
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
                            birth: 3.0,
                            death: 4.0,
                        },
                        PersistenceInterval {
                            birth: 2.0,
                            death: 5.0,
                        },
                    ],
                ),
                (2, vec![]),
            ],
        );
        assert_eq!(result, expected)
    }
}
