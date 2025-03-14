/// Union-Find data structure
#[derive(Debug, Clone)]
pub struct UnionFind {
    #[allow(dead_code)] // TODO
    size: usize,
    subsets: Vec<usize>,
}

impl UnionFind {
    #[allow(dead_code)] // TODO
    fn new(size: usize) -> Self {
        let subsets = (0..size).collect();
        Self { size, subsets }
    }

    #[allow(dead_code)] // TODO
    fn len(self) -> usize {
        self.subsets.len()
    }

    #[allow(dead_code)] // TODO
    fn find(&mut self, x: usize) -> usize {
        let mut y = x;
        while self.subsets[y] != y {
            y = self.subsets[y]
        }
        y
    }

    #[allow(dead_code)] // TODO
    fn merge(&mut self, x: usize, y: usize) {
        // TODO path compression
        let x_root = self.find(x);
        let y_root = self.find(y);
        self.subsets[x_root] = y_root;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;

    #[test]
    fn test_union_find() {
        let _ = env_logger::try_init();

        let mut uf = UnionFind::new(5);
        assert_eq!(uf.clone().len(), 5);
        uf.merge(0, 4);
        uf.merge(2, 3);
        uf.merge(1, 2);
        assert_eq!(uf.subsets.clone(), vec![4, 3, 3, 3, 4]);
        debug!("UnionFind: {:?}", uf.subsets.clone());
    }
}
