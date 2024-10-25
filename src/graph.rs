/*!
Wrappers for main cascade-core types
and graph functions
*/

use ahash::{HashMap, HashMapExt};

use cascade_core::graph::GraphNode;
use cascade_core::prelude::*;

use geo::Point;
use pyo3::prelude::*;
use pyo3::types::PyString;

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
    let mut id_mapping = HashMap::with_capacity(graph.node_count());
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
    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let class_name: Bound<'_, PyString> = slf.get_type().qualname()?;
        Ok(format!(
            "{class_name} with {} nodes and {} edges",
            slf.borrow().graph.node_count(),
            slf.borrow().graph.edge_count()
        ))
    }

    #[must_use]
    pub fn get_mapping(&self) -> HashMap<usize, PyGraphNode> {
        self.id_mapping.clone()
    }

    #[warn(unstable_features)]
    pub fn extend_with_transit(
        &mut self,
        gtfs_path: &str,
        departure: u32,
        duration: u32,
        weekday: &str,
    ) -> PyResult<()> {
        let gtfs_path = std::path::PathBuf::from(gtfs_path);
        // dummy pbf path, not used in this case
        // but required by the FeedArgs struct
        let pbf_path = gtfs_path.clone();

        let feed_args = FeedArgs {
            gtfs_path,
            pbf_path,
            departure,
            duration,
            weekday,
        };

        self.graph.extend_with_transit(&feed_args).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Graph creation failed: {e:?}"))
        })?;

        Ok(())
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyGraphNode {
    pub node_type: String,
    pub id: String,
    pub geometry: Point,
}

#[pymethods]
impl PyGraphNode {
    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let class_name: Bound<'_, PyString> = slf.get_type().qualname()?;
        Ok(format!("{class_name}"))
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }

    #[must_use]
    pub fn get_node_type(&self) -> String {
        self.node_type.clone()
    }

    #[getter(id)]
    #[must_use]
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    #[getter(geometry)]
    #[must_use]
    pub fn get_geometry(&self) -> (f64, f64) {
        (self.geometry.x(), self.geometry.y())
    }

    #[getter(x)]
    #[must_use]
    pub fn get_x(&self) -> f64 {
        self.geometry.x()
    }

    #[getter(y)]
    #[must_use]
    pub fn get_y(&self) -> f64 {
        self.geometry.y()
    }
}
