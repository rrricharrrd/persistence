#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::{PyTuple, PyDict};
#[cfg(feature = "python")]
use numpy::{PyArray2, ToPyArray, PyReadonlyArray2};

use ndarray::Array2;
use super::point_cloud::PointCloud;
use super::homology::ChainComplex;

#[cfg(feature = "python")]
#[pyfunction]
pub fn pairwise_distances(py: Python, points: PyReadonlyArray2<f64>) -> Py<PyArray2<f64>> {
    let points: Array2<f64> = points.as_array().into_owned();
    let point_cloud = PointCloud { points };
    let distance_matrix = point_cloud.pairwise_distances();
    distance_matrix.to_pyarray_bound(py).into()
}

#[cfg(feature = "python")]
#[pyfunction]
pub fn persistence_intervals(
    py: Python,
    points: PyReadonlyArray2<f64>,
    max_dimension: usize,
    threshold: f64,
) -> Py<PyDict> {

    let points: Array2<f64> = points.as_array().into_owned();
    let point_cloud = PointCloud { points };
    let complex = point_cloud.vietoris_rips_complex(max_dimension, threshold);
    let intervals = complex.persistence_intervals();

    let py_intervals = PyDict::new_bound(py);
    for (dim, ints) in intervals {
        let py_list: Vec<_> = ints.iter()
            .map(|s| {
                PyTuple::new_bound(
                    py,
                    [
                        s.birth.into_py(py),
                        s.birth_chain.clone().into_py(py),
                        s.death.into_py(py),
                        s.death_chain.clone().into_py(py),
                    ],
                )
            })
            .collect();
        py_intervals.set_item(dim, py_list).unwrap();
    }

    py_intervals.into()
}

#[cfg(feature = "python")]
#[pymodule]
pub fn persistence_rs(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pairwise_distances, m)?)?;
    m.add_function(wrap_pyfunction!(persistence_intervals, m)?)?;
    Ok(())
}
