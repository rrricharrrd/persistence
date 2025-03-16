use ordered_float::OrderedFloat;
use std::collections::HashSet;
use log::debug;

/// Persistence interval
#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceInterval {
    pub birth: OrderedFloat<f64>,
    pub death: OrderedFloat<f64>,
}

// Define a struct for a simplex entry, used to help track persistence
#[derive(Clone, Debug)]
struct TableEntry<'a, T> {
    chain: &'a T,
    filtration_level: OrderedFloat<f64>,
    is_marked: bool, // Is cycle to be retained in next dimension
    co_bounds: HashSet<usize>, // Elements of pivot column
}

#[allow(dead_code)] // TODO
pub trait Chain {
    fn dim(&self) -> usize; // Dimension of chain
}

#[allow(dead_code)] // TODO
pub trait ChainComplex<T: Chain + std::fmt::Debug> {
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

    #[allow(private_interfaces)] // TODO
    fn remove_pivot_rows(&self, chain_ix: usize, table: &[TableEntry<T>]) -> HashSet<usize> {
        // Get boundary indices of given simplex
        let chain = table[chain_ix].chain;
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
        let mut table: Vec<TableEntry<T>> = self.chains()
            .iter()
            .enumerate()
            .map(|(i, s)| {
                TableEntry {
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
                intervals[dim].push(PersistenceInterval {
                    birth: entry.filtration_level,
                    death: OrderedFloat(f64::INFINITY),
                });
            }
        }
        intervals
    }
}
