use ordered_float::OrderedFloat;
use std::collections::HashMap;

#[allow(dead_code)] // TODO
trait Chain {
    fn filtration_level(&self) -> OrderedFloat<f64>; // Filtration level
    fn dim(&self) -> usize; // Dimension of chain
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
#[allow(dead_code)] // TODO
struct Simplex {
    vertices: Vec<usize>,
}

impl Chain for Simplex {
    fn dim(&self) -> usize {
        self.vertices.len() - 1
    }

    fn filtration_level(&self) -> OrderedFloat<f64> {
        ordered_float::OrderedFloat(0.0)
    }
}

#[allow(dead_code)] // TODO
trait ChainComplex<T: Chain> {
    fn chain(&self, index: usize) -> &T;
    fn chains(&self) -> &Vec<T>;
    fn boundary(&self, index: usize) -> Vec<usize>; // Note working over Z/2
    fn len(&self) -> usize {
        self.chains().len()
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // TODO
struct SimplicialComplex {
    pub simplices: Vec<Simplex>,
    indexes: HashMap<Simplex, usize>,
}

impl SimplicialComplex {
    #[allow(dead_code)] // TODO
    fn new(simplices: Vec<Simplex>) -> Self {
        let indexes: HashMap<Simplex, usize> = simplices
            .iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), i))
            .collect();
        Self {
            simplices: simplices.clone(),
            indexes,
        }
    }
}

impl ChainComplex<Simplex> for SimplicialComplex {
    fn chain(&self, index: usize) -> &Simplex {
        &self.simplices[index]
    }

    fn chains(&self) -> &Vec<Simplex> {
        &self.simplices
    }

    fn boundary(&self, index: usize) -> Vec<usize> {
        // Note working over Z/2
        let s = &self.simplices[index];
        let mut bounds = Vec::new();
        for v in s.vertices.clone() {
            let mut ds_vertices = s.vertices.clone();
            ds_vertices.retain(|&x| x != v); // Remove vertex
            if !ds_vertices.is_empty() {
                let ds = Simplex { vertices: ds_vertices };
                bounds.push(*self.indexes.get(&ds).unwrap());
            }
        }
        bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;

    #[test]
    fn test_simplicial_complex() {
        let _ = env_logger::try_init();

        let simplex0 = Simplex { vertices: vec![0] };
        let simplex1 = Simplex { vertices: vec![1] };
        let simplex01 = Simplex { vertices: vec![0, 1] };
        let complex = SimplicialComplex::new(vec![simplex0, simplex1, simplex01.clone()]);
        assert_eq!(simplex01.clone().dim(), 1);
        assert_eq!(simplex01.filtration_level(), OrderedFloat(0.0));
        assert_eq!(complex.len(), 3);
        debug!("{:?}", complex);

        for i in 0..complex.len() {
            let boundary = complex.boundary(i);
            debug!("d.{:?} is {:?}", complex.chain(i), boundary);
        }
    }
}
