use super::morse_complex::MorseComplex;

use super::union_find::UnionFind;
use ndarray::{Array2, ArrayView1};


#[derive(Debug, Clone)]
pub struct Heatmap {
    pub pixels: Array2<f64>,
}

fn flat_index(rx: usize, cx: usize, width: usize) -> usize {
    rx * width + cx
}

impl Heatmap {
    fn height(&self) -> usize {
        self.pixels.nrows()
    }

    fn width(&self) -> usize {
        self.pixels.ncols()
    }

    #[allow(dead_code)] // TODO
    fn sorted_pixel_indices(&self) -> Vec<(f64, usize, usize)> {
        let mut pixels_flat: Vec<(f64, usize, usize)> = Vec::new();

        for rx in (0..self.height()) {
            for cx in (0..self.width()) {
                let value = self.pixels[[rx, cx]];
                pixels_flat.push((value, rx, cx));
            }
        }

        // Sort by descending value, then ascending i, then ascending j
        pixels_flat.sort_by(|a, b|
            b.0.partial_cmp(&a.0) // Descending value
            .unwrap()
            .then(a.1.cmp(&b.1)) // Ascending row index
            .then(a.2.cmp(&b.2)) // Ascending column index
        );

        // Extract just (i, j) pairs
        // pixels.into_iter().map(|(_, i, j)| (i, j)).collect()
        pixels_flat
    }

    #[allow(dead_code)] // TODO
    fn find_maxima(&self) -> UnionFind {
        let uf = UnionFind::new(pixels.len());

        let pixels_flat = self.sorted_pixel_indices();
        for (value, rx, cx) in pixels {
            let index = flat_index(rx, cx, self.width());

            let value_up = if rx > 0 {
                self.pixels[[rx - 1, cx]]
            } else {
                -f64::INFINITY
            };
            let value_down = if rx < self.height() - 1 {
                self.pixels[[rx + 1, cx]]
            } else {
                -f64::INFINITY
            };
            let is_ymin = value_up > value && value_down > value;
            let is_ymax = value_up < value && value_down < value;
            // otherwise just normal non-critical point

            let value_left = if cx > 0 {
                self.pixels[[rx, cx - 1]]
            } else {
                -f64::INFINITY
            };
            let value_right = if cx < self.width() - 1 {
                self.pixels[[rx, cx + 1]]
            } else {
                -f64::INFINITY
            };
            let is_xmin = value_left > value && value_right > value;
            let is_xmax = value_left < value && value_right < value;
            // otherwise just normal non-critical point

            let is_max = is_xmax && is_ymax;
            let is_min = is_xmin && is_ymin;
            let is_saddle = (is_xmax && is_ymin) || (is_xmin && is_ymax);
            // TODO
            if !(is_max || is_min || is_saddle) {
                // Find direction of greatest increase
                let uphill = [value_up, value_down, value_left, value_right]; // arg
                if uphill == 0 {
                    let index_up = flat_index(rx - 1, cx, self.width());
                    uf.merge(index_up, index);
                } else if uphill == 1 {
                    let index_down = flat_index(rx + 1, cx, self.width());
                    uf.merge(index_down, index);
                } else if uphill == 2 {
                    let index_left = flat_index(rx, cx - 1, self.width());
                    uf.merge(index_left, index);
                } else if uphill == 3 {
                    let index_right = flat_index(rx, cx + 1, self.width());
                    uf.merge(index_right, index);
                } else {
                    panic!("Couldn't merge uphill");
                }
            }
        }
        uf
    }

    /// Create Morse complex
    fn morse_complex(&self) -> MorseComplex {
        let uf = self.find_maxima();
        MorseComplex::new(vec![])  // TODO
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
