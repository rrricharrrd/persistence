pub mod point_cloud;
pub mod simplicial_complex;
mod combinatorics;
mod homology;
pub mod python;
use pyo3::prelude::*;


/// Create a Python module
#[pymodule]
fn persistence(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(
        wrap_pyfunction!(python::pairwise_distances_py, m)?,
    )?;
    Ok(())
}
