use pyo3::prelude::*;

use crate::algo::{snap_point, PyPoint};
use crate::graph::PyTransitGraph;
use cascade_core::prelude::*;

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

    let result = cascade_core::algo::isochrone::calculate_isochrone(
        graph,
        &source,
        dep_time,
        cutoff,
        buffer_radius,
    )
    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{e:?}")))?;

    Ok(result)
}

#[pyfunction]
pub fn bulk_isochrones(
    graph: &PyTransitGraph,
    sources: Vec<PyPoint>,
    start_time: u32,
    cutoff: f64,
    buffer_radius: f64,
) -> PyResult<String> {
    let graph = &graph.graph;

    let snapped_points: Vec<(String, SnappedPoint)> = sources
        .into_iter()
        .map(|py_point| {
            snap_point(py_point.x, py_point.y, graph).map(|snapped| (py_point.id, snapped))
        })
        .collect::<Result<_, _>>()?;

    let isochrones = cascade_core::algo::bulk_isochrones(
        graph,
        &snapped_points,
        start_time,
        cutoff,
        buffer_radius,
    )
    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{e:?}")))?;

    let collection = geojson::ser::to_feature_collection_string(&isochrones)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("{e:?}")))?;

    Ok(collection)
}
