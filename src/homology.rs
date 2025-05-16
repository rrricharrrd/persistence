use log::debug;
use ndarray::Array2;
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};

/// Performs symmetric difference (XOR) operation between two sets over Z/2.
/// Modifies the first set in place.
fn xor(a: &mut HashSet<usize>, b: &HashSet<usize>) {
    for &y in b {
        if !a.insert(y) {
            a.remove(&y);
        }
    }
}

/// Represents a persistence interval in the filtration.
///
/// A persistence interval tracks when a topological feature appears (birth)
/// and when it disappears (death), along with the chains that create and destroy it.
#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceInterval {
    /// Value at which the feature appears
    pub birth: f64,
    /// Chain representing the birth of the feature
    pub birth_chain: HashSet<usize>,
    /// Value at which the feature disappears (infinity if the feature persists)
    pub death: f64,
    /// Chain representing the death of the feature (None if the feature persists)
    pub death_chain: Option<HashSet<usize>>,
}

/// Internal representation of a complex entry used during persistence computation.
#[derive(Clone, Debug)]
struct TableEntry {
    /// Index of the parent chain that kills this cycle
    parent: usize,
    /// Indicates if this represents a cycle that persists to the next dimension
    represents_cycle: bool,
    /// Elements in the pivot column (chains that this bounds)
    co_bounds: HashSet<usize>,
    /// Chains that this element bounds
    bound: HashSet<usize>,
    /// Chains collected with this basis element during reduction
    chain: HashSet<usize>,
}

impl TableEntry {
    /// Creates a new TableEntry initialized with the given index in its chain.
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

/// Represents a chain in a chain complex.
///
/// A chain is a formal sum of basis elements in a given dimension.
/// This trait should be implemented by any type that can be part of a chain complex.
pub trait Chain {
    /// Returns the dimension of this chain.
    fn dim(&self) -> usize;
}

/// Represents a chain complex with a boundary operator.
///
/// A chain complex is a sequence of vector spaces connected by boundary operators
/// that satisfy the property that the composition of any two consecutive boundary
/// operators is zero.
pub trait ChainComplex<T: Chain + std::fmt::Debug> {
    /// Returns a reference to the chain at the given index.
    fn chain(&self, index: usize) -> &T;

    /// Returns a reference to all chains in the complex.
    fn chains(&self) -> &Vec<T>;

    /// Returns the boundary of the chain at the given index.
    ///
    /// The boundary is represented as a set of indices of the chains that form
    /// the boundary. This implementation works over Z/2, so multiplicities are
    /// not tracked.
    fn boundary(&self, index: usize) -> HashSet<usize>;

    /// Returns the number of chains in the complex.
    fn len(&self) -> usize {
        self.chains().len()
    }

    /// Returns true if the complex contains no chains.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the filtration level of the chain at the given index.
    ///
    /// The filtration level determines when this chain appears in the filtration.
    /// By default, all chains appear at level 0.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    fn filtration_level(&self, index: usize) -> OrderedFloat<f64> {
        if index >= self.len() {
            panic!("Index {} out of bounds for chain complex of length {}", index, self.len());
        }
        OrderedFloat(0.0)
    }

    /// Computes the boundary matrix of the chain complex.
    ///
    /// The boundary matrix represents the boundary operator as a matrix where
    /// entry (i,j) is 1 if chain i appears in the boundary of chain j (working over Z/2).
    ///
    /// Note: This does not include filtration information.
    fn boundary_matrix(&self) -> Array2<usize> {
        let n = self.len();
        let mut matrix = Array2::<usize>::zeros((n, n));
        for (ix, _) in self.chains().iter().enumerate() {
            for jx in self.boundary(ix) {
                matrix[[jx, ix]] = 1;
            }
        }
        matrix
    }

    /// Removes pivot rows from the boundary of a chain.
    ///
    /// This is a key step in the persistence algorithm that performs the matrix reduction
    /// to compute persistent homology. It modifies the table entries in place.
    ///
    /// # Arguments
    ///
    /// * `chain_ix` - Index of the chain whose boundary we're processing
    /// * `table` - Mutable reference to the table of entries being built
    ///
    /// # Returns
    ///
    /// The reduced boundary after removing pivot rows
    #[allow(private_interfaces)] // TODO
    fn remove_pivot_rows(&self, chain_ix: usize, table: &mut Vec<TableEntry>) -> HashSet<usize> {
        let mut boundary = self.boundary(chain_ix);

        // Only keep boundary chains that generate cycles
        boundary.retain(|&bx| table[bx].represents_cycle);

        // Reduce the boundary by performing column operations
        while let Some(&max_index) = boundary.iter().max() {
            if table[max_index].co_bounds.is_empty() {
                // This row is unclaimed - use as pivot
                break;
            } else {
                // Perform column operation
                xor(&mut boundary, &table[max_index].co_bounds);
                // Add chains from the parent entry
                let other = table[table[max_index].parent].chain.clone();
                table[chain_ix].chain.extend(&other);
            }
        }

        boundary
    }

    /// Compute persistence intervals for the chain complex.
    ///
    /// This implements the standard persistence algorithm to compute the intervals
    /// where homological features (connected components, loops, voids, etc.) exist.
    ///
    /// # Returns
    ///
    /// A HashMap where keys are dimensions and values are vectors of persistence intervals
    /// in that dimension.
    fn persistence_intervals(&self) -> HashMap<usize, Vec<PersistenceInterval>> {
        let mut table: Vec<TableEntry> = (0..self.len()).map(TableEntry::new).collect();

        // Initialize intervals map for each dimension up to max dimension
        let max_dim = self.chains().iter().map(|s| s.dim()).max().unwrap_or(0);
        let mut intervals: HashMap<usize, Vec<PersistenceInterval>> = (0..=max_dim).map(|i| (i, Vec::new())).collect();

        // Process each chain in the complex
        for chain_ix in 0..self.len() {
            let boundary = self.remove_pivot_rows(chain_ix, &mut table);

            if boundary.is_empty() {
                // This chain creates a new cycle
                table[chain_ix].represents_cycle = true;
            } else if let Some(&max_boundary_index) = boundary.iter().max() {
                // This chain kills an existing cycle
                table[max_boundary_index].co_bounds = boundary;
                table[max_boundary_index].parent = chain_ix;
                table[max_boundary_index].bound.insert(chain_ix);

                let dim = self.chain(max_boundary_index).dim();
                debug!(
                    "Creating interval: birth={}, death={}",
                    self.filtration_level(max_boundary_index),
                    self.filtration_level(chain_ix)
                );

                intervals.get_mut(&dim).unwrap().push(PersistenceInterval {
                    birth: self.filtration_level(max_boundary_index).into_inner(),
                    birth_chain: table[max_boundary_index].chain.clone(),
                    death: self.filtration_level(chain_ix).into_inner(),
                    death_chain: Some(table[chain_ix].chain.clone()),
                });
            }
        }

        // Process remaining cycles (those that never die)
        for (ix, entry) in table.iter().enumerate() {
            if entry.represents_cycle && entry.co_bounds.is_empty() {
                let dim = self.chain(ix).dim();
                debug!("Creating infinite interval: birth={}", self.filtration_level(ix));

                intervals.get_mut(&dim).unwrap().push(PersistenceInterval {
                    birth: self.filtration_level(ix).into_inner(),
                    birth_chain: entry.chain.clone(),
                    death: f64::INFINITY,
                    death_chain: None,
                });
            }
        }

        intervals
    }
}
