pub mod homology;
pub mod point_cloud;
pub mod simplicial_complex;

mod combinatorics;
mod union_find;

// Python packaging
#[cfg(feature = "python")]
pub mod pylib;
#[cfg(feature = "python")]
pub use pylib::persistence;
