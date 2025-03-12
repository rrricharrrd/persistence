use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};
use log::debug;

/// Persistence interval
#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceInterval {
    birth: OrderedFloat<f64>,
    death: OrderedFloat<f64>,
}

// Define a struct for a simplex entry
#[derive(Clone, Debug)]
struct Entry<'a, T> {
    chain: &'a T,
    filtration_level: OrderedFloat<f64>,
    is_marked: bool, // Is cycle to be retained in next dimension
    co_bounds: HashSet<usize>, // Elements of pivot column
}

#[allow(dead_code)] // TODO
trait Chain {
    // fn filtration_level(&self) -> OrderedFloat<f64> {
    //     // Default is no separate filtration levels
    //     ordered_float::OrderedFloat(0.0)
    // }
    fn dim(&self) -> usize; // Dimension of chain
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
#[allow(dead_code)] // TODO
struct Simplex {
    vertices: Vec<usize>,
}

impl Chain for Simplex {
    fn dim(&self) -> usize {
        self.vertices.len() - 1
    }
}

#[allow(dead_code)] // TODO
trait ChainComplex<T: Chain + std::fmt::Debug> {
    fn chain(&self, index: usize) -> &T;
    fn chains(&self) -> &Vec<T>;
    fn boundary(&self, index: usize) -> HashSet<usize>; // Note working over Z/2
    fn len(&self) -> usize {
        self.chains().len()
    }
    fn filtration_level(&self, index: usize) -> OrderedFloat<f64> {
        if index >= self.len() {
            panic!("Invalid index");
        }
        OrderedFloat(0.0) // Default is no separate filtration levels
    }

    fn remove_pivot_rows(&self, chain_ix: usize, table: &[Entry<T>]) -> HashSet<usize> {
        // Get boundary indices of given simplex
        let chain = &table[chain_ix].chain;
        let mut boundary: HashSet<usize> = self.boundary(chain_ix);
        debug!(
            "Removing pivot from {:?}, full-boundary={:?}",
            chain,
            boundary
        );

        // Remove any boundary chains that don't generate the cycles in that dimension
        boundary.retain(|&bx| table[bx].is_marked);

        // Simulate conversion to echelon form
        while let Some(b) = boundary.iter().max() {
            if table[*b].co_bounds.is_empty() {
                // This row is unclaimed - use as pivot
                break;
            } else {
                for cb in &table[*b].co_bounds {
                    // Simulate subtracting pivot column (working over Z/2)
                    if boundary.contains(cb) {
                        boundary.remove(cb);
                    } else {
                        boundary.insert(*cb);
                    }
                }
            }
        }

        debug!(
            "Removing pivot from {:?}: reduced-boundary={:?}",
            chain,
            boundary
        );
        boundary
    }


    /// Compute persistence intervals for simplicial complex
    fn compute_intervals(&self) -> Vec<Vec<PersistenceInterval>> {
        let mut table: Vec<Entry<T>> = self.chains()
            .iter()
            .enumerate()
            .map(|(i, s)| {
                Entry {
                    chain: s,
                    filtration_level: self.filtration_level(i),
                    is_marked: false,
                    co_bounds: HashSet::new(),
                }
            })
            .collect();

        let max_dim = self.chains().iter().map(|s| s.dim()).max().unwrap();
        let mut intervals: Vec<Vec<PersistenceInterval>> = vec![Vec::new(); max_dim + 1];

        for sx in 0..table.len() {
            let boundary = self.remove_pivot_rows(sx, &table);

            if boundary.is_empty() {
                table[sx].is_marked = true;
            } else if let Some(b) = boundary.iter().max() {
                debug!("Storing {:?} in {:?}", &boundary, b);
                table[*b].co_bounds = boundary.clone();

                let dim = table[*b].chain.dim();
                intervals[dim].push(PersistenceInterval {
                    birth: table[*b].filtration_level,
                    death: table[sx].filtration_level,
                });
            }
        }

        debug!("Table");
        for entry in table {
            debug!("{:?}", entry);
            if entry.is_marked && entry.co_bounds.is_empty() {
                let dim = entry.chain.dim();
                // Note: usize::MAX for infinity
                intervals[dim].push(PersistenceInterval {
                    birth: entry.filtration_level,
                    death: OrderedFloat(f64::INFINITY),
                });
            }
        }
        intervals
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // TODO
struct SimplicialComplex {
    simplices: Vec<Simplex>,
    levels: Vec<f64>,
    indexes: HashMap<Simplex, usize>,
}

impl SimplicialComplex {
    #[allow(dead_code)] // TODO
    fn new(simplices: Vec<Simplex>, levels: Vec<f64>) -> Self {
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
        let result = complex.compute_intervals();
        debug!("Result {:?}", result);

        // Then
        // TODO make ordering-agnostic
        let expected = vec![
            vec![
                PersistenceInterval {
                    birth: OrderedFloat(0.0),
                    death: OrderedFloat(1.0),
                },
                PersistenceInterval {
                    birth: OrderedFloat(1.0),
                    death: OrderedFloat(1.0),
                },
                PersistenceInterval {
                    birth: OrderedFloat(1.0),
                    death: OrderedFloat(2.0),
                },
                PersistenceInterval {
                    birth: OrderedFloat(0.0),
                    death: OrderedFloat(f64::INFINITY),
                },
            ],
            vec![
                PersistenceInterval {
                    birth: OrderedFloat(3.0),
                    death: OrderedFloat(4.0),
                },
                PersistenceInterval {
                    birth: OrderedFloat(2.0),
                    death: OrderedFloat(5.0),
                },
            ],
            vec![],
        ];
        assert_eq!(result, expected)
    }
}
