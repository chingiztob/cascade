use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn return_string() -> String {
    router_core::utils::return_string()
}

/// Formats the sum of two numbers as string.
#[pyfunction]
#[allow(clippy::unnecessary_wraps)]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn cascade(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(return_string, m)?)?;
    Ok(())
}
