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
struct Simplex {
    vertices: Vec<usize>,
}

impl Simplex {
    fn dim(&self) -> usize {
        self.vertices.len() - 1
    }
}

fn remove_pivot_rows(
    simplex_ix: usize,
    boundary_op: &Vec<Vec<i32>>,
    table: &Vec<Entry>,
) -> Vec<usize> {
    // Get boundary indices where boundary_op[:, simplex_ix] is nonzero
    let mut boundary: Vec<usize> = boundary_op
        .iter()
        .enumerate()
        .filter_map(|(bx, row)| if row[simplex_ix] != 0 { Some(bx) } else { None })
        .collect();

    // Keep only marked entries
    boundary.retain(|&b| table[b].is_marked);

    println!(
        "Removing pivot from {:?}, full-boundary={:?}",
        table[simplex_ix].simplex, boundary
    );

    while let Some(&max_bounding_chain) = boundary.iter().max_by_key(|&&b| table[b].level) {
        if table[max_bounding_chain].chain.is_empty() {
            break;
        }
        boundary.retain(|&b| b != max_bounding_chain);
    }

    println!(
        "Removing pivot from {:?}: reduced-boundary={:?}",
        table[simplex_ix].simplex, boundary
    );

    boundary
}


fn compute_intervals(
    simplices: &HashMap<Simplex, usize>, 
    boundary_op: &Vec<Vec<i32>>
) -> (Vec<HashSet<(usize, usize)>>, Vec<Entry>) {
    let mut table: Vec<Entry> = simplices
        .iter()
        .map(|(s, &v)| Entry {
            simplex: s.clone(),
            level: v,
            is_marked: false,
            chain: Vec::new(),
        })
        .collect();

    let mut max_dim = 0;
    for simplex in simplices.keys() {
        max_dim = max_dim.max(simplex.dim());
    }
    let mut intervals: Vec<HashSet<(usize, usize)>> = vec![HashSet::new(); max_dim + 1];

    for sx in 0..table.len() {
        let boundary = remove_pivot_rows(sx, boundary_op, &table);

        if boundary.is_empty() {
            table[sx].is_marked = true;
        } else {
            if let Some(&max_bounding_chain) = boundary.iter().max_by_key(|&&b| table[b].level) {
                table[max_bounding_chain].chain = boundary.clone();
                let dim = table[sx].simplex.dim();
                intervals[dim].insert((table[max_bounding_chain].level, table[sx].level));
            }
        }
    }

    for sx in 0..table.len() {
        if table[sx].is_marked && table[sx].chain.is_empty() {
            let dim = table[sx].simplex.dim();
            intervals[dim].insert((table[sx].level, usize::MAX)); // usize::MAX for infinity
        }
    }

    (intervals, table)
}


fn compute_boundary_op(ordering: &HashMap<Simplex, usize>, simplices: &Vec<Simplex>) -> Vec<Vec<i32>> {
    let n = ordering.len();
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
        let mut simplices: HashMap<Simplex, usize> = HashMap::new();
        simplices.insert(Simplex{ vertices: vec![0] }, 0);
        simplices.insert(Simplex{ vertices: vec![1] }, 0);
        simplices.insert(Simplex{ vertices: vec![2] }, 1);
        simplices.insert(Simplex{ vertices: vec![3] }, 1);
        simplices.insert(Simplex{ vertices: vec![0,1] }, 1);
        simplices.insert(Simplex{ vertices: vec![1,2] }, 1);
        simplices.insert(Simplex{ vertices: vec![2,3] }, 2);
        simplices.insert(Simplex{ vertices: vec![0,3] }, 2);
        simplices.insert(Simplex{ vertices: vec![0,2] }, 3);
        simplices.insert(Simplex{ vertices: vec![0,1,2] }, 4);
        simplices.insert(Simplex{ vertices: vec![0,2,3] }, 5);
        println!("Simplices {:?}", simplices);

        let boundary_op: Vec<Vec<i32>> = compute_boundary_op(&simplices, &Vec::from_iter(simplices.clone().into_keys().into_iter()));
        println!("Boundary {:?}", boundary_op);
        
        let result = compute_intervals(&simplices, &boundary_op);
        println!("Result {:?}", result);
    }
}
