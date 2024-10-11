/*!
Hello world!
*/

use pyo3::prelude::*;

use crate::algo::{shortest_path_rs, single_source_shortest_path_rs};
use crate::graph::{create_graph, PyTransitGraph};

pub mod algo;
pub mod graph;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn return_string() -> String {
    cascade_core::utils::return_string()
}

#[pymodule]
fn _cascade_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(return_string, m)?)?;
    m.add_function(wrap_pyfunction!(single_source_shortest_path_rs, m)?)?;
    m.add_function(wrap_pyfunction!(shortest_path_rs, m)?)?;
    m.add_function(wrap_pyfunction!(create_graph, m)?)?;
    m.add_class::<PyTransitGraph>()?;
    Ok(())
}
