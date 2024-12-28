use pyo3::prelude::*;

use crate::algo::snap_point;
use crate::graph::PyTransitGraph;

/// Retrieve the actual shortest path between a source and target node as a sequence of node indices
#[pyfunction]
pub fn detailed_itinerary(
    graph: &PyTransitGraph,
    dep_time: u32,
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
) -> PyResult<String> {
    let graph = &graph.graph;

    let source = snap_point(source_x, source_y, graph)?;
    let target = snap_point(target_x, target_y, graph)?;

    let itinerary = cascade_core::algo::detailed_itinerary(graph, &source, &target, dep_time);

    Ok(itinerary.to_geojson().to_string())
}
