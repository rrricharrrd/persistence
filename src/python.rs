use pyo3::prelude::*;
use numpy::{PyArray2, ToPyArray, PyReadonlyArray2};
use ndarray::Array2;
use super::point_cloud::PointCloud;

#[pyfunction]
pub fn pairwise_distances_py(py: Python, points: PyReadonlyArray2<f64>) -> Py<PyArray2<f64>> {
    let points: Array2<f64> = points.as_array().into_owned();
    let point_cloud = PointCloud { points };
    let distance_matrix = point_cloud.pairwise_distances();
    distance_matrix.to_pyarray_bound(py).into()
}
