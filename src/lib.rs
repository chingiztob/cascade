use std::collections::HashMap;

use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn return_string() -> String {
    router_core::utils::return_string()
}

/// Creates a graph from GTFS and OSM data.
#[pyfunction]
fn create_graph(
    gtfs_path: &str,
    pbf_path: &str,
    departure: u32,
    duration: u32,
    weekday: &str,
) -> TransitGraphRs {
    let graph =
        router_core::example::create_graph(gtfs_path, pbf_path, departure, duration, weekday);

    TransitGraphRs { graph }
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn single_source_shortest_path(
    graph: &TransitGraphRs,
    start_time: u32,
    x: f64,
    y: f64,
) -> HashMap<usize, f64> {
    let graph = &graph.graph;
    router_core::example::shortest_path_wrapper(graph, start_time, x, y)
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn shortest_path(graph: &TransitGraphRs, start_time: u32) -> f64 {
    let graph = &graph.graph;
    router_core::example::single_shortest_path_wrapper(graph, start_time)
}

#[pyclass]
struct TransitGraphRs {
    graph: router_core::graph::TransitGraph,
}

/// A Python module implemented in Rust.
#[pymodule]
fn _cascade_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(return_string, m)?)?;
    m.add_function(wrap_pyfunction!(single_source_shortest_path, m)?)?;
    m.add_function(wrap_pyfunction!(shortest_path, m)?)?;
    m.add_function(wrap_pyfunction!(create_graph, m)?)?;
    m.add_class::<TransitGraphRs>()?;
    Ok(())
}
