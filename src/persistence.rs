use std::collections::{HashMap, HashSet};
use log::debug;


// Define a struct for a simplex entry
#[derive(Clone, Debug)]
struct Entry<'a> {
    simplex: &'a Simplex,
    filtration_level: f64,
    is_marked: bool, // Is cycle to be retained in next dimension
    co_bounds: HashSet<usize>, // Elements of pivot column
}


/// Persistence interval
#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceInterval {
    birth: f64,
    death: f64,
}

/// Simplex (as defined via its vertices)
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Simplex {
    pub vertices: Vec<usize>,
}

impl Simplex {
    fn dim(&self) -> usize {
        self.vertices.len() - 1
    }
}


fn remove_pivot_rows(
    simplex_ix: usize,
    boundary_op: &[Vec<i32>],
    table: &[Entry],
) -> HashSet<usize> {
    // Get boundary indices of given simplex
    let simplex = &table[simplex_ix].simplex;
    let mut boundary: HashSet<usize> = boundary_op
        .iter()
        .enumerate()
        .filter_map(|(bx, row)| if row[simplex_ix] != 0 {
            Some(bx)
        } else {
            None
        })
        .collect();
    debug!(
        "Removing pivot from {:?}, full-boundary={:?}",
        simplex,
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
        simplex,
        boundary
    );
    boundary
}


/// Compute persistence intervals for simplicial complex
pub fn compute_intervals(
    simplices: &[Simplex],
    simplices_map: &HashMap<Simplex, f64>,
    boundary_op: &[Vec<i32>],
) -> Vec<Vec<PersistenceInterval>> {
    let mut table: Vec<Entry> = simplices
        .iter()
        .map(|s| {
            Entry {
                simplex: s,
                filtration_level: *simplices_map.get(s).unwrap(),
                is_marked: false,
                co_bounds: HashSet::new(),
            }
        })
        .collect();

    let max_dim = simplices.iter().map(|s| s.dim()).max().unwrap();
    let mut intervals: Vec<Vec<PersistenceInterval>> = vec![Vec::new(); max_dim + 1];

    for sx in 0..table.len() {
        let boundary = remove_pivot_rows(sx, boundary_op, &table);

        if boundary.is_empty() {
            table[sx].is_marked = true;
        } else if let Some(b) = boundary.iter().max() {
            debug!("Storing {:?} in {:?}", &boundary, b);
            table[*b].co_bounds = boundary.clone();

            let dim = table[*b].simplex.dim();
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
            let dim = entry.simplex.dim();
            // Note: usize::MAX for infinity
            intervals[dim].push(PersistenceInterval {
                birth: entry.filtration_level,
                death: f64::INFINITY,
            });
        }
    }
    intervals
}


/// Compute boundary operator for given simplicial complex
pub fn compute_boundary_op(simplices: &[Simplex]) -> Vec<Vec<i32>> {
    let n = simplices.len();
    let ordering: HashMap<Simplex, usize> = simplices
        .iter()
        .enumerate()
        .map(|(i, v)| (v.clone(), i))
        .collect();

    let mut boundary_op = vec![vec![0; n]; n];
    for s in simplices {
        for &v in &s.vertices {
            let mut ds_vertices = s.vertices.clone();
            ds_vertices.retain(|&x| x != v); // Remove vertex
            let ds = Simplex { vertices: ds_vertices };

            if let Some(&row) = ordering.get(&ds) {
                if let Some(&col) = ordering.get(s) {
                    boundary_op[row][col] = 1;
                }
            }
        }
    }

    boundary_op
}


#[cfg(test)]
mod tests {
    use core::f64;

    use super::*;

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

        let simplices_map: HashMap<Simplex, f64> = simplices
            .clone()
            .into_iter()
            .zip(levels.into_iter())
            .collect();
        debug!("Simplices {:?}", simplices_map);

        let boundary_op: Vec<Vec<i32>> = compute_boundary_op(&simplices);
        debug!("Boundary {:?}", boundary_op);

        // When
        let result = compute_intervals(&simplices, &simplices_map, &boundary_op);
        debug!("Result {:?}", result);

        // Then
        // TODO make ordering-agnostic
        let expected = vec![
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
            vec![],
        ];
        assert_eq!(result, expected)
    }
}
