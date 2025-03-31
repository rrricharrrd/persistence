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

        // Order simplices by filtration level
        let mut paired: Vec<_> = simplices.into_iter().zip(levels).collect();
        paired.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let (sorted_simplices, sorted_levels): (Vec<_>, Vec<_>) = paired.into_iter().unzip();

        // Construct mapping to be able to access "ith simplex"
        let indexes: HashMap<Simplex, usize> = sorted_simplices
            .iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), i))
            .collect();
        Self {
            simplices: sorted_simplices,
            levels: sorted_levels,
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
    use log::debug;

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

        let complex = SimplicialComplex::new(simplices, levels);

        // When
        let all_intervals = complex.persistence_intervals();
        debug!("Result {:?}", all_intervals);

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
                            birth_chain: HashSet::from(
                                [vec![0, 3], vec![0, 1], vec![1, 2], vec![2, 3]]
                            ),
                            death: 5.0,
                            death_chain: HashSet::from([vec![0, 1, 2], vec![0, 2, 3]]),
                        },
                    ],
                ),
                (2, vec![]),
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
