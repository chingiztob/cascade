use pyo3::prelude::*;
use router_core::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn return_string() -> String {
    router_core::utils::return_string()
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn create_graph() -> TransitGraphRs {
    let graph = router_core::example::create_graph();

    TransitGraphRs { graph }
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn demo(graph: &TransitGraphRs) -> f64 {
    let graph = &graph.graph;
    router_core::example::demo(graph)
}

/// Formats the sum of two numbers as string.
#[pyfunction]
#[allow(clippy::unnecessary_wraps)]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyclass]
struct TransitGraphRs {
    graph: router_core::graph::TransitGraph,
}

/// A Python module implemented in Rust.
#[pymodule]
fn _cascade_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(return_string, m)?)?;
    m.add_function(wrap_pyfunction!(demo, m)?)?;
    m.add_function(wrap_pyfunction!(create_graph, m)?)?;
    m.add_class::<TransitGraphRs>()?;
    Ok(())
}
