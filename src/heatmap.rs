#[derive(Debug, Clone)]
pub struct Heatmap {
    pub pixels: Vec<Vec<f64>>,
}

impl Heatmap {
    #[allow(dead_code)] // TODO
    fn sorted_pixel_indices(&self) -> Vec<(usize, usize)> {
        let mut pixels: Vec<(f64, usize, usize)> = Vec::new();

        for (i, row) in self.pixels.iter().enumerate() {
            for (j, &v) in row.iter().enumerate() {
                pixels.push((v, i, j));
            }
        }

        // Sort by descending value, then ascending i, then ascending j
        pixels.sort_by(|a, b|
            b.0.partial_cmp(&a.0) // Descending value
            .unwrap()
            .then(a.1.cmp(&b.1)) // Ascending i
            .then(a.2.cmp(&b.2)) // Ascending j
        );

        // Extract just (i, j) pairs
        pixels.into_iter().map(|(_, i, j)| (i, j)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;

    #[test]
    fn test_union_find() {
        let _ = env_logger::try_init();

        let heatmap = Heatmap {
            pixels: vec![
                vec![0.2, 0.8, 0.5],
                vec![0.4, 0.1, 0.9],
                vec![0.7, 0.3, 0.6],
            ],
        };

        let sorted_indices = heatmap.sorted_pixel_indices();
        debug!("{:?}", sorted_indices);
    }
}
