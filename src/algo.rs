use ahash::HashMap;

use cascade_core::prelude::*;
use geo::Point;
use pyo3::prelude::*;

use crate::graph::PyTransitGraph;

///  Finds the shortest paths from source node in a time-dependent graph using Dijkstra's algorithm.
///
/// # Arguments
/// * `graph` - A reference to a `TransitGraph` object.
/// * `start` - The source node index.
/// * `start_time` - The starting time in seconds since midnight.
/// * `x` - The x coordinate of the source point in 4326.
/// * `y` - The y coordinate of the source point in 4326.
/// # Returns
/// A `HashMap` with the shortest path weight in seconds to each node from the source node.
#[pyfunction]
#[pyo3(name = "single_source_shortest_path")]
pub fn single_source_shortest_path_rs(
    graph: &PyTransitGraph,
    start_time: u32,
    x: f64,
    y: f64,
) -> PyResult<HashMap<usize, f64>> {
    let graph = &graph.graph;
    let source = SnappedPoint::init(Point::new(x, y), graph).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to snap point: {e:?}"))
    })?;

    let hmap = cascade_core::algo::single_source_shortest_path(graph, &source, start_time)
        .into_iter()
        .map(|(k, v)| (k.index(), v))
        .collect();

    Ok(hmap)
}

/// Formats the sum of two numbers as string.
#[pyfunction]
#[pyo3(name = "shortest_path")]
pub fn shortest_path_rs(
    graph: &PyTransitGraph,
    start_time: u32,
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
) -> PyResult<f64> {
    let graph = &graph.graph;

    let source = SnappedPoint::init(Point::new(source_x, source_y), graph).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to snap point: {e:?}"))
    })?;
    let target = SnappedPoint::init(Point::new(target_x, target_y), graph).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to snap point: {e:?}"))
    })?;

    let result = cascade_core::algo::shortest_path(graph, &source, &target, start_time)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{e:?}")))?;

    Ok(result)
}
