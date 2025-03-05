use std::collections::{HashMap, HashSet};
//use std::cmp::Ordering;


// Define a struct for a simplex entry
#[derive(Clone, Debug)]
struct Entry {
    simplex: Simplex,
    level: usize,
    is_marked: bool,
    chain: Vec<usize>,
}

// Define a struct for simplices (placeholder)
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Simplex {
    vertices: Vec<usize>,
}

impl Simplex {
    fn dim(&self) -> usize {
        self.vertices.len() - 1
    }
}

fn remove_pivot_rows(simplex_ix: usize, boundary_op: &[Vec<i32>], table: &[Entry]) -> Vec<usize> {
    // Get boundary indices where boundary_op[:, simplex_ix] is nonzero
    let mut boundary: Vec<usize> = boundary_op
        .iter()
        .enumerate()
        .filter_map(|(bx, row)| if row[simplex_ix] != 0 {
            Some(bx)
        } else {
            None
        })
        .collect();

    // Keep only marked entries
    boundary.retain(|&b| table[b].is_marked);

    println!(
        "Removing pivot from {:?}, full-boundary={:?}",
        table[simplex_ix].simplex,
        boundary
    );

    while let Some(&max_bounding_chain) = boundary.iter().max_by_key(|&&b| table[b].level) {
        if table[max_bounding_chain].chain.is_empty() {
            break;
        }
        boundary.retain(|&b| b != max_bounding_chain);
    }

    println!(
        "Removing pivot from {:?}: reduced-boundary={:?}",
        table[simplex_ix].simplex,
        boundary
    );

    boundary
}


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
                level: *simplices_map.get(s).unwrap(),
                is_marked: false,
                chain: Vec::new(),
            }
        })
        .collect();

    let mut max_dim = 0;
    for simplex in simplices_map.keys() {
        max_dim = max_dim.max(simplex.dim());
    }
    let mut intervals: Vec<HashSet<(usize, usize)>> = vec![HashSet::new(); max_dim + 1];

    for sx in 0..table.len() {
        let boundary = remove_pivot_rows(sx, boundary_op, &table);

        if boundary.is_empty() {
            table[sx].is_marked = true;
        } else if let Some(&max_bounding_chain) = boundary.iter().max_by_key(|&&b| table[b].level) {
            table[max_bounding_chain].chain = boundary.clone();
            let dim = table[sx].simplex.dim();
            intervals[dim].insert((table[max_bounding_chain].level, table[sx].level));
        }
    }

    for entry in table {
        if entry.is_marked && entry.chain.is_empty() {
            let dim = entry.simplex.dim();
            intervals[dim].insert((entry.level, usize::MAX)); // usize::MAX for infinity
        }
    }
    intervals
}


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
    fn persistence_intervals() {
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
    }
}
