/*!
Wrappers for main routing functions of
underlying cascade-core crate
*/

use ahash::HashMap;

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
#[pyo3(name = "single_source_shortest_path")]
pub fn single_source_shortest_path_rs(
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
            snap_point(py_point.x, py_point.y, graph)
                .map(|snapped| (py_point.id, snapped))
                .map_err(PyErr::from) // Convert the error to PyErr for PyO3
        })
        .collect::<Result<Vec<_>, PyErr>>()?;

    // Map of node indices to original PyPoint IDs for lookup
    let id_map: HashMap<usize, &String> = snapped_points
        .iter()
        .map(|(id, sn)| (sn.index().index(), id))
        .collect();

    // Collect the OD matrix with PyPoint IDs as keys
    let od_matrix: HashMap<String, HashMap<String, f64>> = snapped_points
        .par_iter()
        .with_min_len(100) // do not split arrays smaller than 100 elements
        .map(|(id, node)| {
            let shortest_paths =
                cascade_core::algo::single_source_shortest_path(graph, node, dep_time)
                    .into_iter()
                    // For each (k: NodeIndex), find the `id` of the destination point in the id_map
                    .filter_map(|(k, v)| {
                        id_map.get(&k.index()).map(|&dest_id| (dest_id.clone(), v))
                    })
                    .collect::<HashMap<String, f64>>();
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
