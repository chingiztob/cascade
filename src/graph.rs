use std::collections::HashMap;

use cascade_core::graph::GraphNode;
use cascade_core::prelude::*;

use geo::Point;
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
    // create pathbufs from the input strings
    let gtfs_path = std::path::PathBuf::from(gtfs_path);
    let pbf_path = std::path::PathBuf::from(pbf_path);

    let feed_args = FeedArgs {
        gtfs_path,
        pbf_path,
        departure,
        duration,
        weekday,
    };
    let instant = std::time::Instant::now();
    let graph = TransitGraph::from(feed_args).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Graph creation failed: {e:?}"))
    })?;

    let id_mapping = create_mapping(&graph);
    println!("Graph creation time: {:?}", instant.elapsed());

    Ok(PyTransitGraph { graph, id_mapping })
}

fn create_mapping(graph: &TransitGraph) -> HashMap<usize, PyGraphNode> {
    let mut id_mapping = HashMap::new();
    for node in graph.into_inner_graph().node_indices() {
        let node_data = graph.into_inner_graph().node_weight(node).unwrap();

        match node_data {
            GraphNode::Transit(transit_node) => {
                let graph_node = PyGraphNode {
                    node_type: "transit".to_string(),
                    id: transit_node.stop_id.clone(),
                    geometry: transit_node.geometry,
                };
                id_mapping.insert(node.index(), graph_node);
            }
            GraphNode::Walk(street_node) => {
                let graph_node = PyGraphNode {
                    node_type: "street".to_string(),
                    id: format!("{}", street_node.id.0),
                    geometry: street_node.geometry,
                };
                id_mapping.insert(node.index(), graph_node);
            }
        };
    }
    id_mapping
}

#[pyclass]
pub struct PyTransitGraph {
    pub graph: TransitGraph,
    id_mapping: HashMap<usize, PyGraphNode>,
}

#[pymethods]
impl PyTransitGraph {
    #[must_use]
    pub fn get_mapping(&self) -> HashMap<usize, PyGraphNode> {
        self.id_mapping.clone()
    }
}

#[pyclass]
#[derive(Clone)]
pub struct PyGraphNode {
    pub node_type: String,
    pub id: String,
    pub geometry: Point,
}

#[pymethods]
impl PyGraphNode {
    #[must_use]
    pub fn get_node_type(&self) -> String {
        self.node_type.clone()
    }

    #[must_use]
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    #[must_use]
    pub fn get_geometry(&self) -> (f64, f64) {
        (self.geometry.x(), self.geometry.y())
    }
}
