use pyo3::prelude::*;

use crate::algo::snap_point;
use crate::graph::PyTransitGraph;

#[pyfunction]
pub fn calculate_isochrone(
    graph: &PyTransitGraph,
    source_x: f64,
    source_y: f64,
    dep_time: u32,
    cutoff: f64,
    buffer_radius: f64,
) -> PyResult<String> {
    let graph = &graph.graph;

    let source = snap_point(source_x, source_y, graph)?;

    let result =
        cascade_core::algo::isochrone::isochrone(graph, &source, dep_time, cutoff, buffer_radius)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{e:?}")))?;

    Ok(result)
}
