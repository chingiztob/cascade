use pyo3::prelude::*;

use crate::algo::snap_point;
use crate::graph::PyTransitGraph;

/// Computes an detailed itinerary in `GeoJSON` format,
/// containing the shortest path from the source to the target.
/// Each segment within the itinerary encapsulates detailed travel information,
/// including duration, geometry, and transit characteristics.
/// If no path is found, the returned itinerary will be empty.
///
/// # Example
/// ```python
/// from cascade import detailed_itinerary, PyTransitGraph
///
/// gtfs_path = "/Your_Feed"
/// pbf_path = "/roads.pbf"
/// departure = 0
/// duration = 86400
/// weekday = "monday"
///
/// graph = cascade.create_graph(gtfs_path, pbf_path, departure, duration, weekday)
///
/// dep_time = 1609459200  # Example departure time (Unix timestamp)
/// source_x, source_y = 37.7749, -122.4194  # source coordinates
/// target_x, target_y = 34.0522, -118.2437  # target coordinates
///
/// itinerary = detailed_itinerary(graph, dep_time, source_x, source_y, target_x, target_y)
/// print(itinerary)
/// ```
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
