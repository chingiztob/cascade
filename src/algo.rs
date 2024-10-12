use ahash::{HashMap, HashSet};

use cascade_core::prelude::*;
use geo::Point;
use rayon::prelude::*;
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
    let source = snap_point(x, y, graph)?;

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
    dep_time: u32,
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
) -> PyResult<f64> {
    let graph = &graph.graph;

    let source = snap_point(source_x, source_y, graph)?;
    let target = snap_point(target_x, target_y, graph)?;

    let result = cascade_core::algo::shortest_path(graph, &source, &target, dep_time)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{e:?}")))?;

    Ok(result)
}

/// Formats the sum of two numbers as string.
#[pyfunction]
pub fn calculate_od_matrix(
    graph: &PyTransitGraph,
    nodes: Vec<(f64, f64)>,
    dep_time: u32,
) -> PyResult<HashMap<usize, HashMap<usize, f64>>> {
    let graph = &graph.graph;
    let nodes = nodes
        .into_iter()
        .map(|(x, y)| snap_point(x, y, graph))
        .collect::<Result<Vec<_>, _>>()?;

    let od_matrix = nodes
        .par_iter()
        .map(|node| {
            let shortest_paths =
                cascade_core::algo::single_source_shortest_path(graph, node, dep_time)
                    .into_iter()
                    .map(|(k, v)| (k.index(), v))
                    .collect::<HashMap<usize, f64>>();
            (node.index().index(), shortest_paths) // This is cringe, but it works
        })
        .collect::<HashMap<usize, HashMap<usize, f64>>>();

    Ok(od_matrix)
}

fn snap_point(x: f64, y: f64, graph: &TransitGraph) -> PyResult<SnappedPoint> {
    SnappedPoint::init(Point::new(x, y), graph).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to snap point: {e:?}"))
    })
}
