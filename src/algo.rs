/*!
This module provides algorithms for finding shortest paths in time-dependent transit graphs. It includes functions to:

- Compute the shortest paths from a source node to all other nodes using Dijkstra's algorithm ([`single_source_shortest_path()`]).
- Find the shortest path weight between a source and target node ([`shortest_path_weight()`]).
- Retrieve the actual shortest path between a source and target node as a sequence of node indices ([`shortest_path()`]).
- Calculate an origin-destination (OD) matrix for a set of points, providing the shortest path weights between all pairs of points ([`calculate_od_matrix()`]).

The module also defines a [`PyPoint`] class, a Python wrapper for passing coordinates with an ID to the Rust backend, facilitating seamless integration between Rust and Python components.

# Examples
```python
from cascade import create_graph, single_source_shortest_path, shortest_path_weight, shortest_path, PyPoint

gtfs_path = "path/to/City_GTFS"
pbf_path = "path/to/City.pbf"
departure = 0
duration = 86400
weekday = "monday"

graph = create_graph(gtfs_path, pbf_path, departure, duration, weekday)

(source_x, source_y) = (59.851960, 30.221418)
(target_x, target_y) = (59.978989, 30.502047)]

print(
    cascade.shortest_path_weight(
        graph=graph,
        dep_time=43200,
        source_x=source_x,
        source_y=source_y,
        target_x=target_x,
        target_y=target_y,
    )
)
```
*/

use ahash::{HashMap, HashMapExt};

use cascade_core::prelude::*;
use geo::Point;
use pyo3::prelude::*;
use pyo3::types::PyString;
use rayon::prelude::*;

use crate::graph::PyTransitGraph;

///  Finds the shortest paths from source node in a time-dependent graph using Dijkstra's algorithm.
///
/// # Arguments
/// * `graph` - A reference to a `TransitGraph` object.
/// * `start` - The source node index.
/// * `dep_time` - The departure time in seconds since midnight.
/// * `x` - The x coordinate of the source point in 4326.
/// * `y` - The y coordinate of the source point in 4326.
/// # Returns
/// A `HashMap` with the shortest path weight in seconds to each node from the source node.
#[pyfunction]
#[pyo3(name = "single_source_shortest_path_weight")]
pub fn single_source_shortest_path_weight(
    graph: &PyTransitGraph,
    dep_time: u32,
    x: f64,
    y: f64,
) -> PyResult<HashMap<usize, f64>> {
    let graph = &graph.graph;
    let source = snap_point(x, y, graph)?;

    let hmap = cascade_core::algo::single_source_shortest_path(graph, &source, dep_time)
        .into_iter()
        .map(|(k, v)| (k.index(), v))
        .collect();

    Ok(hmap)
}

///  Finds the shortest paths from source node in a time-dependent graph using Dijkstra's algorithm.
///
/// # Arguments
/// * `graph` - A reference to a `TransitGraph` object.
/// * `start` - The source node index.
/// * `dep_time` - The departure time in seconds since midnight.
/// * `source_x` - The x coordinate of the source point in 4326.
/// * `source_y` - The y coordinate of the source point in 4326.
/// * `target_x` - The x coordinate of the target point in 4326.
/// * `target_y` - The y coordinate of the target point in 4326.
/// # Returns
/// weight of the shortest path in seconds.
#[pyfunction]
#[pyo3(name = "shortest_path_weight")]
pub fn shortest_path_weight(
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

    let result = cascade_core::algo::shortest_path_weight(graph, &source, &target, dep_time)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{e:?}")))?;

    Ok(result)
}

/// Retrieve the actual shortest path between a source and target node as a sequence of node indices
#[pyfunction]
#[pyo3(name = "shortest_path")]
pub fn shortest_path(
    graph: &PyTransitGraph,
    dep_time: u32,
    source_x: f64,
    source_y: f64,
    target_x: f64,
    target_y: f64,
) -> PyResult<Vec<usize>> {
    let graph = &graph.graph;

    let source = snap_point(source_x, source_y, graph)?;
    let target = snap_point(target_x, target_y, graph)?;

    let result = cascade_core::algo::shortest_path(graph, &source, &target, dep_time);

    Ok(result)
}

/// Calculate an origin-destination (OD) matrix for a set of points, providing the shortest path weights between all pairs of points
#[pyfunction]
pub fn calculate_od_matrix(
    graph: &PyTransitGraph,
    nodes: Vec<PyPoint>,
    dep_time: u32,
) -> PyResult<HashMap<String, HashMap<String, f64>>> {
    let graph = &graph.graph;

    let snapped_points: Vec<(String, SnappedPoint)> = nodes
        .into_iter()
        .map(|py_point| {
            snap_point(py_point.x, py_point.y, graph).map(|snapped| (py_point.id, snapped))
        })
        .collect::<Result<_, _>>()?;

    // Map of node indices to original PyPoint IDs for lookup
    let id_map: HashMap<usize, &String> = snapped_points
        .iter()
        .map(|(id, sn)| (sn.index().index(), id))
        .collect();

    // Collect the OD matrix with PyPoint IDs as keys
    let od_matrix: HashMap<String, HashMap<String, f64>> = snapped_points
        .par_iter()
        .map(|(id, node)| {
            let mut shortest_paths = HashMap::with_capacity(snapped_points.len());
            for (k, v) in cascade_core::algo::single_source_shortest_path(graph, node, dep_time) {
                if let Some(&dest_id) = id_map.get(&k.index()) {
                    shortest_paths.insert(dest_id.clone(), v); // Directly insert into pre-allocated map
                }
            }
            (id.clone(), shortest_paths)
        })
        .collect();

    Ok(od_matrix)
}

fn snap_point(x: f64, y: f64, graph: &TransitGraph) -> PyResult<SnappedPoint> {
    SnappedPoint::init(Point::new(x, y), graph).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to snap point: {e:?}"))
    })
}

/// A Python wrapper to pass coordinates with an ID to Rust backend.
#[pyclass(get_all)]
#[derive(Clone, Debug)] // This allow backwards conversion from python PyPoint
pub struct PyPoint {
    pub x: f64,
    pub y: f64,
    pub id: String,
}

#[pymethods]
impl PyPoint {
    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let class_name: Bound<'_, PyString> = slf.get_type().qualname()?;
        Ok(format!("{class_name}"))
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }

    #[new]
    #[must_use]
    pub fn new(x: f64, y: f64, id: String) -> Self {
        Self { x, y, id }
    }
}
