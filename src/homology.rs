use log::debug;
use ndarray::Array2;
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};

fn xor(a: &mut HashSet<usize>, b: &HashSet<usize>) {
    // TODO make generic
    // Simulates column operation (working over Z/2)
    for y in b {
        if a.contains(y) {
            a.remove(y);
        } else {
            a.insert(*y);
        }
    }
}

/// Persistence interval
#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceInterval {
    pub birth: f64,
    pub birth_chain: HashSet<usize>,
    pub death: f64,
    pub death_chain: Option<HashSet<usize>>,
}

// Define a struct for a complex entry, used to help track persistence
#[derive(Clone, Debug)]
struct TableEntry {
    parent: usize,
    represents_cycle: bool,    // Is cycle to be retained in next dimension
    co_bounds: HashSet<usize>, // Elements of pivot column
    bound: HashSet<usize>,     // TODO Represents (part of) what bounds this chain
    chain: HashSet<usize>,     // TODO Chains collected with basis element during reduction
}

impl TableEntry {
    fn new(index: usize) -> Self {
        Self {
            parent: 0, // TODO
            represents_cycle: false,
            co_bounds: HashSet::new(),
            bound: HashSet::new(),
            chain: HashSet::from([index]),
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
    fn remove_pivot_rows(&self, chain_ix: usize, table: &mut Vec<TableEntry>) -> HashSet<usize> {
        // Get boundary indices of given simplex
        let mut boundary: HashSet<usize> = self.boundary(chain_ix);
        // debug!(
        //     "Removing pivot from {:?}, full-boundary={:?}",
        //     self.chain(chain_ix),
        //     boundary
        // );

        // Remove any boundary chains that don't generate the cycles in that dimension
        boundary.retain(|&bx| table[bx].represents_cycle);

        // Simulate conversion to echelon form
        while let Some(b) = boundary.clone().into_iter().max() {
            if table[b].co_bounds.is_empty() {
                // This row is unclaimed - use as pivot
                break;
            } else {
                xor(&mut boundary, &table[b].co_bounds);
                let other = table[table[b].parent].chain.clone();
                table[chain_ix].chain.extend(&other);
            }
        }

        //debug!("After removing pivot: boundary={:?}", boundary);
        boundary
    }

    /// Compute persistence intervals for simplicial complex
    fn persistence_intervals(&self) -> HashMap<usize, Vec<PersistenceInterval>> {
        let mut table: Vec<TableEntry> = (0..self.len()).map(TableEntry::new).collect();

        let max_dim = self.chains().iter().map(|s| s.dim()).max().unwrap();
        let mut intervals: HashMap<usize, Vec<PersistenceInterval>> =
            (0..max_dim + 1).map(|i| (i, Vec::new())).collect();

        for chain_ix in 0..table.len() {
            let boundary = self.remove_pivot_rows(chain_ix, &mut table);

            if boundary.is_empty() {
                table[chain_ix].represents_cycle = true;
            } else if let Some(&b) = boundary.iter().max() {
                table[b].co_bounds = boundary.clone();
                table[b].parent = chain_ix;
                table[b].bound.insert(chain_ix);

                let dim = self.chain(b).dim();
                debug!(
                    "Interval created by {:?}, killed by {:?}",
                    &boundary, table[chain_ix].chain
                );
                intervals.entry(dim).or_default().push(PersistenceInterval {
                    birth: self.filtration_level(b).into_inner(),
                    birth_chain: table[b].chain.clone(), // TODO
                    death: self.filtration_level(chain_ix).into_inner(),
                    death_chain: Some(table[chain_ix].chain.clone()), // TODO
                });
            }
        }

        debug!("Table");
        for (ix, entry) in table.iter().enumerate() {
            debug!("{:?}", entry);
            if entry.represents_cycle && entry.co_bounds.is_empty() {
                let dim = self.chain(ix).dim();
                debug!("Interval created by {:?}, never killed", entry.co_bounds);
                intervals.entry(dim).or_default().push(PersistenceInterval {
                    birth: self.filtration_level(ix).into_inner(),
                    birth_chain: table[ix].chain.clone(), // TODO
                    death: f64::INFINITY,
                    death_chain: None,
                });
            }
        }
        intervals
    }
}
