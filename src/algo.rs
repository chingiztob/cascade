use std::collections::HashMap;

use cascade_core::prelude::*;
use geo::Point;
use pyo3::prelude::*;

use crate::graph::TransitGraphRs;

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
pub fn single_source_shortest_path_rs(
    graph: &TransitGraphRs,
    start_time: u32,
    x: f64,
    y: f64,
) -> PyResult<HashMap<usize, f64>> {
    let graph = &graph.graph;
    let source = SnappedPoint::init(Point::new(x, y), graph).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to snap point: {e:?}"))
    })?;

    let hmap = cascade_core::algo::single_source_shortest_path(graph, &source, start_time);

    Ok(hmap.into_iter().map(|(k, v)| (k.index(), v)).collect())
}

/// Formats the sum of two numbers as string.
#[pyfunction]
pub fn shortest_path_rs(graph: &TransitGraphRs, start_time: u32) -> PyResult<f64> {
    let graph = &graph.graph;

    let source = SnappedPoint::init(Point::new(30.320234, 59.875912), graph).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to snap point: {e:?}"))
    })?;
    let target = SnappedPoint::init(Point::new(30.309416, 60.066852), graph).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to snap point: {e:?}"))
    })?;

    let result = cascade_core::algo::shortest_path(graph, &source, &target, start_time)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{e:?}")))?;

    Ok(result)
}
