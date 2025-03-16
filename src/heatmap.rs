use super::union_find::UnionFind;


#[derive(Debug, Clone)]
pub struct Heatmap {
    pub pixels: Vec<Vec<f64>>,
}

impl Heatmap {
    fn height(&self) -> usize {
        self.pixels.len()
    }

    fn width(&self) -> usize {
        self.pixels[0].len()
    }

    #[allow(dead_code)] // TODO
    fn sorted_pixel_indices(&self) -> Vec<(f64, usize, usize)> {
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
        // pixels.into_iter().map(|(_, i, j)| (i, j)).collect()
        pixels
    }

    #[allow(dead_code)] // TODO
    fn find_maxima(&self) -> UnionFind {
        let pixels = self.sorted_pixel_indices();
        let uf = UnionFind::new(pixels.len());
        for (v, i, j) in pixels {
            // let index = i * self.width() + j;

            let up = if i > 0 {
                self.pixels[i - 1][j]
            } else {
                -f64::INFINITY
            };
            let down = if i < self.height() - 1 {
                self.pixels[i + 1][j]
            } else {
                -f64::INFINITY
            };
            let is_ymin = up > v && down > v;
            let is_ymax = up < v && down < v;

            let left = if j > 0 {
                self.pixels[i][j - 1]
            } else {
                -f64::INFINITY
            };
            let right = if j < self.width() - 1 {
                self.pixels[i][j + 1]
            } else {
                -f64::INFINITY
            };
            let is_xmin = left > v && right > v;
            let is_xmax = left < v && right < v;

            let _is_max = is_xmax && is_ymax;
            let _is_min = is_xmin && is_ymin;
            let _is_saddle = (is_xmax && is_ymin) || (is_xmin && is_ymax);
            // TODO
        }
        uf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;

    #[test]
    fn test_union_find() {
        let _ = env_logger::try_init();

        let heatmap = Heatmap { pixels: vec![vec![0.2, 0.8, 0.5], vec![0.4, 0.1, 0.9]] };
        assert_eq!(heatmap.height(), 2);
        assert_eq!(heatmap.width(), 3);

        let sorted_indices = heatmap.sorted_pixel_indices();
        assert_eq!(sorted_indices[0], (0.9, 1, 2));
        debug!("{:?}", sorted_indices);
    }
}
