use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn return_string() -> String {
    router_core::utils::return_string()
}

/// Formats the sum of two numbers as string.
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
fn demo(graph: &TransitGraphRs) -> f64 {
    let graph = &graph.graph;
    router_core::example::shortest_path(graph)
}

#[pyclass]
struct TransitGraphRs {
    graph: router_core::graph::TransitGraph,
}

/// A Python module implemented in Rust.
#[pymodule]
fn _cascade_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(return_string, m)?)?;
    m.add_function(wrap_pyfunction!(demo, m)?)?;
    m.add_function(wrap_pyfunction!(create_graph, m)?)?;
    m.add_class::<TransitGraphRs>()?;
    Ok(())
}
