/*!
Hello world!
*/

use pyo3::prelude::*;

use crate::algo::{shortest_path_rs, single_source_shortest_path_rs, calculate_od_matrix};
use crate::graph::{create_graph, PyTransitGraph};

pub mod algo;
pub mod graph;

#[pymodule]
fn _cascade_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(single_source_shortest_path_rs, m)?)?;
    m.add_function(wrap_pyfunction!(shortest_path_rs, m)?)?;
    m.add_function(wrap_pyfunction!(create_graph, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_od_matrix, m)?)?;
    m.add_class::<PyTransitGraph>()?;
    Ok(())
}
