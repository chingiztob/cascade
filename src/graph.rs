/*!
Module for creating and manipulating transit graphs using GTFS and OSM data, exposed to Python via `PyO3` bindings.
Main logic of the module implemented in [`cascade_core`] crate, with Python bindings provided by this module.

This module provides functionality to:

- **Create a transit graph** from GTFS (General Transit Feed Specification) data and OpenStreetMap (OSM) PBF files using the `create_graph` function.
- **Extend an existing transit graph** with additional GTFS data using the `extend_with_transit` method of `PyTransitGraph`.
- **Access graph nodes and their properties** through the `PyTransitGraph` and `PyGraphNode` classes.

### Key Components

- [`create_graph()`]: A function to initialize a `PyTransitGraph` by providing paths to GTFS and PBF files along with departure time, duration, and weekday.
- [`PyTransitGraph`]: A class representing the transit graph, offering methods to interact with the graph such as `get_mapping` and `extend_with_transit`.
- [`PyGraphNode`]: A class representing individual nodes in the graph, containing information like node type (transit or street), identifier, and geometry.
*/

use ahash::{HashMap, HashMapExt};

use cascade_core::graph::GraphNode;
use cascade_core::prelude::*;

use geo::Point;
use pyo3::prelude::*;
use pyo3::types::PyString;

/// Creates a `PyTransitGraph` based on GTFS and OpenStreetMap data.
///
/// Parameters
/// ----------
/// gtfs_path : str
///     Path to the GTFS files.
/// pbf_path : str
///     Path to the OSM dump in .pbf format.
/// departure : int
///     Departure time in seconds.
/// duration : int
///     Time period from departure for which the graph will be loaded.
/// weekday : str
///     Day of the week in lowercase (e.g., 'monday').
///
/// Returns
/// -------
/// PyTransitGraph
///     Combined multimodal graph representing transit network.
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

    println!("Graph creation time: {:?}", instant.elapsed());

    Ok(PyTransitGraph { graph })
}

/// Multimodal graph of transit system, implemented with `PetGraph`
#[pyclass]
pub struct PyTransitGraph {
    pub graph: TransitGraph,
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

    /// Get mapping of graph raw node ids to `PyGraphNode` objects.
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // panic impossible
    pub fn get_mapping(&self) -> HashMap<usize, PyGraphNode> {
        let graph = &self.graph;
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

    /// Extends the graph with transit data from GTFS feed.
    ///
    /// Parameters
    /// ----------
    /// gtfs_path : str
    ///     Path to the GTFS files.
    /// departure : int
    ///     Departure time in seconds.
    /// duration : int
    ///     Time period from departure for which the graph will be loaded.
    /// weekday : str
    ///     Day of the week in lowercase (e.g., 'monday').
    ///
    /// Returns
    /// -------
    /// None
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

/// Node of transit graph. Contains information about node type, id and geometry
#[pymethods]
impl PyGraphNode {
    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let class_name: Bound<'_, PyString> = slf.get_type().qualname()?;
        Ok(format!("{class_name}"))
    }

    fn __str__(&self) -> String {
        format!("{self:?}")
    }

    /// Returns the node type.
    #[must_use]
    pub fn get_node_type(&self) -> String {
        self.node_type.clone()
    }

    /// Returns the node ID.
    #[getter(id)]
    #[must_use]
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    /// Returns the node geometry as a tuple (x, y).
    #[getter(geometry)]
    #[must_use]
    pub fn get_geometry(&self) -> (f64, f64) {
        (self.geometry.x(), self.geometry.y())
    }

    /// Returns the x coordinate of the node geometry.
    #[getter(x)]
    #[must_use]
    pub fn get_x(&self) -> f64 {
        self.geometry.x()
    }

    /// Returns the y coordinate of the node geometry.
    #[getter(y)]
    #[must_use]
    pub fn get_y(&self) -> f64 {
        self.geometry.y()
    }
}
