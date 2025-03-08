use std::collections::{HashMap, HashSet};


// Define a struct for a simplex entry
#[derive(Clone, Debug)]
struct Entry {
    simplex: Simplex,
    filtration_level: usize,
    is_marked: bool,
    co_bounds: HashSet<usize>,
    //bounding_chain: Some(usize),
}

/// Simplex (as defined via its vertices)
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Simplex {
    vertices: Vec<usize>,
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
    let mut boundary: HashSet<usize> = boundary_op
        .iter()
        .enumerate()
        .filter_map(|(bx, row)| if row[simplex_ix] != 0 {
            Some(bx)
        } else {
            None
        })
        .collect();

    // Remove any boundary chains that don't generate the cycles in that dimension
    boundary.retain(|&bx| table[bx].is_marked);
    println!(
        "Removing pivot from {:?}, full-boundary={:?}",
        table[simplex_ix].simplex,
        boundary
    );

    // Simulate conversion to echelon form
    while let Some(b) = boundary.clone().into_iter().max() {
        if table[b].co_bounds.is_empty() {
            break;
        } else {
            for cb in &table[b].co_bounds {
                if boundary.contains(cb) {
                    boundary.remove(cb); // Working over Z/2
                } else {
                    boundary.insert(*cb);
                }
            }
        }
    }

    println!(
        "Removing pivot from {:?}: reduced-boundary={:?}",
        table[simplex_ix].simplex,
        boundary
    );

    boundary
}


/// Compute persistence intervals for simplicial complex
pub fn compute_intervals(
    simplices: &[Simplex],
    simplices_map: &HashMap<Simplex, usize>,
    boundary_op: &[Vec<i32>],
) -> Vec<HashSet<(usize, usize)>> {
    let mut table: Vec<Entry> = simplices
        .iter()
        .map(|s| {
            Entry {
                simplex: s.clone(),
                filtration_level: *simplices_map.get(s).unwrap(),
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
        } else if let Some(b) = boundary.clone().into_iter().max() {
            println!("Storing {:?} in {:?}", boundary.clone(), b);
            table[b].co_bounds = boundary.clone();

            let dim = table[b].simplex.dim();
            intervals[dim].insert((table[b].filtration_level, table[sx].filtration_level));
        }
    }

    println!("Table");
    for entry in table {
        println!("{:?}", entry);
        if entry.is_marked && entry.co_bounds.is_empty() {
            let dim = entry.simplex.dim();
            intervals[dim].insert((entry.filtration_level, usize::MAX)); // usize::MAX for infinity
        }
    }
    intervals
}


/// Compute boundary operator for given simplicial complex
pub fn compute_boundary_op(simplices: &Vec<Simplex>) -> Vec<Vec<i32>> {
    let n = simplices.len();
    let ordering: HashMap<Simplex, usize> = simplices
        .iter()
        .enumerate()
        .map(|(i, v)| (v.clone(), i))
        .collect();

    let mut boundary_op = vec![vec![0; n]; n]; // Initialize zero matrix

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
    use super::*;

    #[test]
    fn test_persistence_intervals() {
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
        let levels = vec![0, 0, 1, 1, 1, 1, 2, 2, 3, 4, 5];
        assert_eq!(simplices.len(), levels.len());

        let simplices_map: HashMap<Simplex, usize> = simplices
            .clone()
            .into_iter()
            .zip(levels.into_iter())
            .collect();
        println!("Simplices {:?}", simplices_map);

        let boundary_op: Vec<Vec<i32>> = compute_boundary_op(&simplices);
        println!("Boundary {:?}", boundary_op);

        let result = compute_intervals(&simplices, &simplices_map, &boundary_op);
        println!("Result {:?}", result);

        let expected = vec![
            HashSet::from([(0, 1), (1, 1), (1, 2), (0, usize::MAX)]),
            HashSet::from([(2, 5), (3, 4)]),
            HashSet::new(),
        ];
        assert_eq!(result, expected)
    }
}
