/*!
# Cascade (in Development)

**Cascade** is a blazingly-fast™ Rust-based library built using `PyO3`,
designed to provide the same core functionality as `NxTransit`,
a Python library for creating and analyzing
multimodal graphs of urban transit systems using GTFS data.

See the original [NxTransit documentation](https://nxtransit.readthedocs.io/en/latest/)
for an overview of the features being ported and enhanced in this version.
*/

use pyo3::prelude::*;

use crate::algo::{calculate_od_matrix, shortest_path_rs, single_source_shortest_path_rs, PyPoint};
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
    m.add_class::<PyPoint>()?;
    Ok(())
}
