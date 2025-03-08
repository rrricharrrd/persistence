use std::collections::{HashMap, HashSet};
use log::debug;


// Define a struct for a simplex entry
#[derive(Clone, Debug)]
struct Entry<'a> {
    simplex: &'a Simplex,
    is_marked: bool, // Is cycle to be retained in next dimension
    co_bounds: HashSet<usize>, // Elements of pivot column
}


/// Simplex (as defined via its vertices)
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Simplex {
    vertices: Vec<usize>,
    filtration_level: usize,
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
    boundary_op: &[Vec<i32>],
) -> Vec<HashSet<(usize, usize)>> {
    let mut table: Vec<Entry> = simplices
        .iter()
        .map(|s| {
            Entry {
                simplex: s,
                is_marked: false,
                co_bounds: HashSet::new(),
            }
        })
        .collect();

    let max_dim = simplices.iter().map(|s| s.dim()).max().unwrap();
    let mut intervals: Vec<HashSet<(usize, usize)>> = vec![HashSet::new(); max_dim + 1];

    for sx in 0..table.len() {
        let boundary = remove_pivot_rows(sx, boundary_op, &table);

        if boundary.is_empty() {
            table[sx].is_marked = true;
        } else if let Some(b) = boundary.iter().max() {
            debug!("Storing {:?} in {:?}", &boundary, b);
            table[*b].co_bounds = boundary.clone();

            let dim = table[*b].simplex.dim();
            intervals[dim].insert((
                table[*b].simplex.filtration_level,
                table[sx].simplex.filtration_level,
            ));
        }
    }

    debug!("Table");
    for entry in table {
        debug!("{:?}", entry);
        if entry.is_marked && entry.co_bounds.is_empty() {
            let dim = entry.simplex.dim();
            // Note: usize::MAX for infinity
            intervals[dim].insert((entry.simplex.filtration_level, usize::MAX));
        }
    }
    intervals
}


/// Compute boundary operator for given simplicial complex
pub fn compute_boundary_op(simplices: &[Simplex]) -> Vec<Vec<i32>> {
    let n = simplices.len();
    // TODO throughout using dummy filtration level, as not needed to define boundary matrix
    let ordering: HashMap<Simplex, usize> = simplices
        .iter()
        .enumerate()
        .map(|(i, v)| {
            (
                Simplex {
                    vertices: v.vertices.clone(),
                    filtration_level: 0,
                },
                i,
            )
        })
        .collect();

    let mut boundary_op = vec![vec![0; n]; n];
    for s in simplices {
        let ss = Simplex {
            vertices: s.vertices.clone(),
            filtration_level: 0,
        }; // TODO
        for &v in &s.vertices {
            let mut ds_vertices = s.vertices.clone();
            ds_vertices.retain(|&x| x != v); // Remove vertex
            let ds = Simplex {
                vertices: ds_vertices,
                filtration_level: 0,
            };

            if let Some(&row) = ordering.get(&ds) {
                if let Some(&col) = ordering.get(&ss) {
                    boundary_op[row][col] = 1;
                }
            }
        }
    }

    boundary_op
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persistence_intervals() {
        let _ = env_logger::try_init();

        // Given
        let simplices = vec![
            Simplex {
                vertices: vec![0],
                filtration_level: 0,
            },
            Simplex {
                vertices: vec![1],
                filtration_level: 0,
            },
            Simplex {
                vertices: vec![2],
                filtration_level: 1,
            },
            Simplex {
                vertices: vec![3],
                filtration_level: 1,
            },
            Simplex {
                vertices: vec![0, 1],
                filtration_level: 1,
            },
            Simplex {
                vertices: vec![1, 2],
                filtration_level: 1,
            },
            Simplex {
                vertices: vec![2, 3],
                filtration_level: 2,
            },
            Simplex {
                vertices: vec![0, 3],
                filtration_level: 2,
            },
            Simplex {
                vertices: vec![0, 2],
                filtration_level: 3,
            },
            Simplex {
                vertices: vec![0, 1, 2],
                filtration_level: 4,
            },
            Simplex {
                vertices: vec![0, 2, 3],
                filtration_level: 5,
            },
        ];
        let boundary_op: Vec<Vec<i32>> = compute_boundary_op(&simplices);
        debug!("Boundary {:?}", boundary_op);

        // When
        let result = compute_intervals(&simplices, &boundary_op);
        debug!("Result {:?}", result);

        // Then
        let expected = vec![
            HashSet::from([(0, 1), (1, 1), (1, 2), (0, usize::MAX)]),
            HashSet::from([(2, 5), (3, 4)]),
            HashSet::new(),
        ];
        assert_eq!(result, expected)
    }
}
