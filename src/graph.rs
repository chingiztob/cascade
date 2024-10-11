use cascade_core::prelude::*;

use pyo3::prelude::*;

/// Creates a graph from GTFS and OSM data.
#[pyfunction]
#[pyo3(name = "create_graph")]
pub fn create_graph(
    gtfs_path: &str,
    pbf_path: &str,
    departure: u32,
    duration: u32,
    weekday: &str,
) -> PyResult<PyTransitGraph> {
    let feed_args = FeedArgs {
        gtfs_path,
        pbf_path,
        departure,
        duration,
        weekday,
    };
    let instant = std::time::Instant::now();
    let graph = TransitGraph::from(&feed_args).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Graph creation failed: {e:?}"))
    })?;
    println!("Graph creation time: {:?}", instant.elapsed());

    Ok(PyTransitGraph { graph })
}

#[pyclass]
pub struct PyTransitGraph {
    pub graph: TransitGraph,
}
