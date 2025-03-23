pub mod point_cloud;
pub mod simplicial_complex;
pub mod homology;

mod combinatorics;

// Python packaging
#[cfg(feature = "python")]
pub mod pylib;
#[cfg(feature = "python")]
pub use pylib::persistence;
