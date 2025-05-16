#[cfg(feature = "python")]
use numpy::{PyArray2, PyReadonlyArray2, ToPyArray};
#[cfg(feature = "python")]
use pyo3::exceptions::{PyRuntimeError, PyValueError};
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::{PyDict, PyTuple};

use super::homology::ChainComplex;
use super::point_cloud::{PointCloud, PointCloudError};
use ndarray::Array2;

/// Python module providing persistent homology computation.
///
/// This module provides functions for computing persistent homology of point clouds
/// using the Vietoris-Rips complex construction.
#[cfg(feature = "python")]
#[pymodule]
pub fn persistence(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pairwise_distances, m)?)?;
    m.add_function(wrap_pyfunction!(persistence_intervals, m)?)?;
    Ok(())
}

/// Compute pairwise Euclidean distances between points.
///
/// # Arguments
///
/// * `points` - 2D numpy array where each row is a point and each column is a dimension
///
/// # Returns
///
/// A 2D numpy array containing the pairwise distances
///
/// # Raises
///
/// * `ValueError` if the input array is empty or has invalid dimensions
#[cfg(feature = "python")]
#[pyfunction]
pub fn pairwise_distances(py: Python, points: PyReadonlyArray2<f64>) -> PyResult<Py<PyArray2<f64>>> {
    let points: Array2<f64> = points.as_array().into_owned();
    let point_cloud = PointCloud::new(points).map_err(|e| match e {
        PointCloudError::EmptyCloud => PyValueError::new_err("Empty point cloud"),
        PointCloudError::DimensionMismatch { expected, got } => PyValueError::new_err(format!(
            "Inconsistent point dimensions: expected {}, got {}",
            expected, got
        )),
        _ => PyRuntimeError::new_err(e.to_string()),
    })?;

    Ok(point_cloud.pairwise_distances().to_pyarray_bound(py).into())
}

/// Compute persistence intervals for a point cloud.
///
/// This function constructs a Vietoris-Rips complex from the input points
/// and computes its persistent homology.
///
/// # Arguments
///
/// * `points` - 2D numpy array where each row is a point and each column is a dimension
/// * `max_dimension` - Maximum homology dimension to compute
/// * `threshold` - Distance threshold for the Vietoris-Rips complex
///
/// # Returns
///
/// A dictionary mapping dimensions to lists of persistence intervals.
/// Each interval is a tuple (birth, birth_chain, death, death_chain).
///
/// # Raises
///
/// * `ValueError` if:
///   - The input array is empty or has invalid dimensions
///   - The maximum dimension is invalid
///   - The threshold is negative
#[cfg(feature = "python")]
#[pyfunction]
pub fn persistence_intervals(
    py: Python,
    points: PyReadonlyArray2<f64>,
    max_dimension: usize,
    threshold: f64,
) -> PyResult<Py<PyDict>> {
    // Validate inputs
    if threshold < 0.0 {
        return Err(PyValueError::new_err("Threshold must be non-negative"));
    }

    let points: Array2<f64> = points.as_array().into_owned();
    let point_cloud = PointCloud::new(points).map_err(|e| match e {
        PointCloudError::EmptyCloud => PyValueError::new_err("Empty point cloud"),
        PointCloudError::DimensionMismatch { expected, got } => PyValueError::new_err(format!(
            "Inconsistent point dimensions: expected {}, got {}",
            expected, got
        )),
        _ => PyRuntimeError::new_err(e.to_string()),
    })?;

    let complex = point_cloud
        .vietoris_rips_complex(max_dimension, threshold)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    let intervals = complex.persistence_intervals();

    // Convert to Python objects
    let py_intervals = PyDict::new_bound(py);
    for (dim, ints) in intervals {
        let py_list: Vec<_> = ints
            .iter()
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
        py_intervals.set_item(dim, py_list)?;
    }

    Ok(py_intervals.into())
}
