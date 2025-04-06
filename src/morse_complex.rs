use super::homology::{Chain, ChainComplex};
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};

/// Critical point from Morse function
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct MorseCriticalPoint {
    pub point: Vec<usize>,
    value: OrderedFloat<f64>,
    index: usize,
}

impl Chain for MorseCriticalPoint {
    fn dim(&self) -> usize {
        self.index
    }
}

/// Morse complex
#[derive(Debug, Clone)]
pub struct MorseComplex {
    pub critical_points: Vec<MorseCriticalPoint>,
    _indexes: HashMap<MorseCriticalPoint, usize>,
}

impl MorseComplex {
    pub fn new(critical_points: Vec<MorseCriticalPoint>) -> Self {
        let _indexes: HashMap<MorseCriticalPoint, usize> = critical_points
            .iter()
            .enumerate()
            .map(|(i, p)| (p.clone(), i))
            .collect();
        Self {
            critical_points: critical_points.clone(),
            _indexes,
        }
    }
}

impl ChainComplex<MorseCriticalPoint> for MorseComplex {
    fn chain(&self, index: usize) -> &MorseCriticalPoint {
        &self.critical_points[index]
    }

    fn chains(&self) -> &Vec<MorseCriticalPoint> {
        &self.critical_points
    }

    fn filtration_level(&self, index: usize) -> OrderedFloat<f64> {
        self.critical_points[index].value
    }

    fn boundary(&self, index: usize) -> HashSet<usize> {
        // Note working over Z/2
        HashSet::from([index]) // TODO Do this properly
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::debug;

    #[test]
    fn test_morse_complex() {
        let _ = env_logger::try_init();

        let point0 = MorseCriticalPoint {
            point: vec![0, 0],
            value: OrderedFloat(0.0),
            index: 0,
        };
        let point1 = MorseCriticalPoint {
            point: vec![0, 2],
            value: OrderedFloat(2.0),
            index: 1,
        };
        let complex = MorseComplex::new(vec![point0, point1.clone()]);
        assert_eq!(point1.dim(), 1);
        assert_eq!(complex.len(), 2);
        debug!("{:?}", complex);
        // TODO more
    }
}
