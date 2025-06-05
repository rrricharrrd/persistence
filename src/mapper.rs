use std::collections::{HashMap, HashSet};
use std::fmt;

use super::combinatorics::generate_products;
use super::dbscan::dbscan;
use ndarray::{stack, Array2, Axis};
use thiserror::Error;

/// Error types for Mapper
#[derive(Error, Debug)]
pub enum MapperError {
    #[error("Empty points")]
    EmptyPoints,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Graph<T> {
    pub adjacency_list: HashMap<T, HashSet<T>>,
}
impl<T: Eq + std::hash::Hash + Clone> Default for Graph<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + std::hash::Hash + Clone> Graph<T> {
    pub fn new() -> Self {
        Self { adjacency_list: HashMap::new() }
    }

    pub fn add_node(&mut self, node: T) {
        self.adjacency_list.entry(node).or_default();
    }

    pub fn add_edge(&mut self, from: T, to: T) {
        self.adjacency_list.entry(from.clone()).or_default().insert(to);
        self.adjacency_list.entry(from).or_default();
    }

    pub fn neighbors(&self, node: &T) -> Option<&HashSet<T>> {
        self.adjacency_list.get(node)
    }
}

impl<T: Eq + std::hash::Hash + Clone + std::fmt::Display> Graph<T> {
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph G {\n");

        for (node, neighbors) in &self.adjacency_list {
            for neighbor in neighbors {
                dot.push_str(&format!("    \"{}\" -> \"{}\";\n", node, neighbor));
            }

            // Ensure isolated nodes appear in the output
            if neighbors.is_empty() {
                dot.push_str(&format!("    \"{}\";\n", node));
            }
        }

        dot.push_str("}\n");
        dot
    }
}

fn select_rows(data: &Array2<f64>, indices: &[usize]) -> Array2<f64> {
    let views: Vec<_> = indices.iter().map(|&i| data.slice(ndarray::s![i, ..])).collect();

    stack(Axis(0), &views).expect("Failed to stack selected rows")
}

// Represents a node in the structure Mapper discovers
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Node {
    segment: Vec<usize>,
    cluster_label: usize,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write your desired output format
        write!(f, "Node({:?}, {})", self.segment, self.cluster_label)
    }
}

pub fn mapper(
    points: Array2<f64>,
    n_divisions: usize,
    epsilon: f64,
    min_points: usize,
) -> Result<Graph<Node>, MapperError> {
    if points.is_empty() {
        return Err(MapperError::EmptyPoints);
    }

    let mins: Vec<f64> = points
        .axis_iter(Axis(1)) // iterate over columns
        .map(|col| col.iter().cloned().fold(f64::INFINITY, f64::min))
        .collect();
    let maxs: Vec<f64> = points
        .axis_iter(Axis(1)) // iterate over columns
        .map(|col| col.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
        .collect();
    println!("Minmaxes {:?} {:?}", mins, maxs);

    let overlap = 0.1; // TODO make configurable
    let mut graph = Graph::new();

    let slots: Vec<usize> = (0..n_divisions).collect();
    let box_indices = generate_products(&slots, points.ncols());
    let mut overlaps: HashMap<usize, Vec<Node>> = HashMap::new();
    for indices in box_indices.unwrap() {
        // TODO
        let box_mins: Vec<f64> = mins
            .iter()
            .zip(maxs.iter())
            .zip(indices.iter())
            .map(|((&lo, &hi), &ix)| lo - overlap / 2. + ((ix as f64) / (n_divisions as f64)) * (hi - lo))
            .collect();
        let box_maxs: Vec<f64> = mins
            .iter()
            .zip(maxs.iter())
            .zip(indices.iter())
            .map(|((&lo, &hi), &ix)| lo + overlap / 2. + (((ix + 1) as f64) / (n_divisions as f64)) * (hi - lo))
            .collect();

        let box_points: Vec<usize> = points
            .axis_iter(Axis(0))
            .enumerate()
            .filter(|(_, pt)| {
                box_mins.iter().zip(box_maxs.iter()).zip(pt.iter()).all(|((&lo, &hi), &x)| lo <= x && x <= hi)
            })
            .map(|(ix, _)| ix)
            .collect();
        println!("Box indices {:?} {:?} {:?} {:?}", indices.clone(), box_mins, box_maxs, box_points);
        if box_points.is_empty() {
            continue;
        }

        let clustering = dbscan(select_rows(&points, &box_points), epsilon, min_points).unwrap();
        let mut labels_seen: HashSet<usize> = HashSet::new();
        println!("Box clustering {:?}", clustering);
        for (ix, &label) in clustering.iter().enumerate() {
            if label == 0 {
                // TODO ignore noise?
                continue;
            }
            let node = Node { segment: indices.clone(), cluster_label: label };
            if labels_seen.insert(label) {
                graph.add_node(node.clone()); // TODO clone
            }

            let is_new = match overlaps.entry(ix) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().push(node.clone()); // TODO clone
                    false
                },
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(vec![node.clone()]);
                    true
                },
            };
            if !is_new {
                for n in overlaps.get(&ix).unwrap() {
                    if *n != node {
                        // TODO error
                        graph.add_edge(node.clone(), n.clone());
                        graph.add_edge(n.clone(), node.clone()); // TODO make undirected
                    }
                }
            }
        }
        //println!("Overlaps {:?}", overlaps);
    }
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    // use log::debug;
    use ndarray::{array, Array2};

    #[test]
    fn test_empty() {
        let points = Array2::<f64>::zeros((0, 0));
        let result = mapper(points, 2, 0.1, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_basic() {
        //  xxx
        //  x x
        //  xxx--x--x
        let points = array![
            [0.0, 0.0],
            [0.5, 0.0],
            [1.0, 0.0],
            [1.0, 0.5],
            [1.0, 1.0],
            [1.0, 1.5],
            [0.5, 1.5],
            [0.0, 1.5],
            [0.0, 1.0],
            [0.0, 0.5],
            [1.5, 0.0],
            [2.0, 0.0],
            [2.5, 0.0],
            [3.0, 0.0]
        ];
        let result = mapper(points, 3, 0.51, 1).unwrap();
        println!("{:?}", result);
        println!("{}", result.to_dot());
        // assert_eq!(result, expected);
    }
}
