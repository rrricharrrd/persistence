use ndarray::Array2;
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};
use log::debug;

/// Persistence interval
#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceInterval {
    pub birth: f64,
    pub death: f64,
}

// Define a struct for a complex entry, used to help track persistence
#[derive(Clone, Debug)]
struct TableEntry {
    // chain_extra: Vec<usize>,  // TODO Chains collected with basis element during reduction
    represents_cycle: bool, // Is cycle to be retained in next dimension
    co_bounds: HashSet<usize>, // Elements of pivot column
}

impl TableEntry {
    fn new() -> Self {
        Self {
            // chain_extra: vec![],
            represents_cycle: false,
            co_bounds: HashSet::new(),
        }
    }
}

pub trait Chain {
    fn dim(&self) -> usize; // Dimension of chain
}

pub trait ChainComplex<T: Chain + std::fmt::Debug> {
    fn chain(&self, index: usize) -> &T;
    fn chains(&self) -> &Vec<T>;
    fn boundary(&self, index: usize) -> HashSet<usize>; // Note working over Z/2

    fn len(&self) -> usize {
        self.chains().len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn filtration_level(&self, index: usize) -> OrderedFloat<f64> {
        if index >= self.len() {
            panic!("Invalid index");
        }
        OrderedFloat(0.0) // Default is no separate filtration levels
    }

    fn boundary_matrix(&self) -> Array2<usize> {
        // TODO this does not include any filtration information
        let n = self.chains().len();
        let mut matrix = Array2::<usize>::zeros((n, n));
        for (ix, _) in self.chains().iter().enumerate() {
            for jx in self.boundary(ix) {
                matrix[[jx, ix]] = 1;
            }
        }
        matrix
    }

    #[allow(private_interfaces)] // TODO
    fn remove_pivot_rows(&self, chain_ix: usize, table: &[TableEntry]) -> HashSet<usize> {
        // Get boundary indices of given simplex
        let chain = self.chain(chain_ix);
        let mut boundary: HashSet<usize> = self.boundary(chain_ix);
        debug!(
            "Removing pivot from {:?}, full-boundary={:?}",
            chain,
            boundary
        );

        // Remove any boundary chains that don't generate the cycles in that dimension
        boundary.retain(|&bx| table[bx].represents_cycle);

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

        debug!("After removing pivot: boundary={:?}", boundary);
        boundary
    }

    /// Compute persistence intervals for simplicial complex
    fn persistence_intervals(&self) -> HashMap<usize, Vec<PersistenceInterval>> {
        let mut table: Vec<TableEntry> = vec![TableEntry::new(); self.len()];

        let max_dim = self.chains().iter().map(|s| s.dim()).max().unwrap();
        let mut intervals: HashMap<usize, Vec<PersistenceInterval>> =
            (0..max_dim + 1).map(|i| (i, Vec::new())).collect();

        for chain_ix in 0..table.len() {
            let boundary = self.remove_pivot_rows(chain_ix, &table);

            if boundary.is_empty() {
                table[chain_ix].represents_cycle = true;
            } else if let Some(&b) = boundary.iter().max() {
                debug!("Storing {:?} in {:?}", &boundary, b);
                table[b].co_bounds = boundary.clone();

                let dim = self.chain(b).dim();
                intervals.entry(dim).or_default().push(PersistenceInterval {
                    birth: self.filtration_level(b).into_inner(),
                    death: self.filtration_level(chain_ix).into_inner(),
                });
            }
        }

        debug!("Table");
        for (ix, entry) in table.iter().enumerate() {
            debug!("{:?}", entry);
            if entry.represents_cycle && entry.co_bounds.is_empty() {
                let dim = self.chain(ix).dim();
                intervals.entry(dim).or_default().push(PersistenceInterval {
                    birth: self.filtration_level(ix).into_inner(),
                    death: f64::INFINITY,
                });
            }
        }
        intervals
    }
}
