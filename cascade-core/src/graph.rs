use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use geo::Point;
use osm4routing::NodeId;
use petgraph::graph::DiGraph;
use petgraph::prelude::{EdgeIndex, NodeIndex};
use rstar::RTree;

use crate::connectors;
use crate::connectors::IndexedPoint;
use crate::loaders;
use crate::streets;
use crate::Error;

/// Struct to store the arguments required to create a `TransitGraph` from a GTFS feed
/// # Fields
/// * `gtfs_path` - Path to the GTFS feed folder
/// * `edgelist_path` - Path to the `edgelist` file
/// * `departure` - Departure time in seconds from midnight
/// * `duration` - Duration in seconds for which the graph should be created
/// * `weekday` - Weekday for which the graph should be created
pub struct FeedArgs<'a> {
    pub gtfs_path: PathBuf,
    pub pbf_path: PathBuf,
    pub departure: u32,
    pub duration: u32,
    pub weekday: &'a str,
}

/// `TransitGraph` struct to store the graph and associated method.
/// Graph is backed by [`petgraph`] `DiGraph` with `NodeType` and `EdgeType` as node and edge types respectively.
#[derive(Debug, Clone)]
pub struct TransitGraph {
    // graph shouldnt be public but for now
    // idk how to implement all the traits
    graph: DiGraph<GraphNode, GraphEdge>,
    /// `RTree` for spatial indexing of nodes.
    /// Used for snapping points to the nearest node
    /// This tree only keeps the street nodes
    rtree: Option<RTree<IndexedPoint>>,
}

impl TransitGraph {
    /// Create a new `TransitGraph`
    pub(crate) fn new() -> Self {
        Self {
            graph: DiGraph::<GraphNode, GraphEdge>::new(),
            rtree: None,
        }
    }

    /// Create a `TransitGraph` from an edgelist file.
    #[must_use]
    pub(crate) fn from_parts(
        graph: DiGraph<GraphNode, GraphEdge>,
        rtree: RTree<IndexedPoint>,
    ) -> Self {
        // Create a graph from the edgelist data
        TransitGraph {
            graph,
            rtree: Some(rtree),
        }
    }

    /// Create a `TransitGraph` from the GTFS feed and walk graph
    /// # Arguments
    /// * `feed_args` - A [`FeedArgs`] struct containing the GTFS feed path, edgelist path, departure time, duration, and weekday
    /// # Returns
    /// A `TransitGraph` object
    /// # Panics
    /// Panics if the GTFS feed path is invalid or the required columns are missing
    /// # Example
    /// ```ignore
    /// use polars_test::graph::{FeedArgs, TransitGraph};
    ///
    /// let feed_args = FeedArgs {
    ///     gtfs_path: "path/to/City_GTFS",
    ///     pbf_path: "path/to/City.pbf",
    ///     departure: 0,
    ///     duration: 86400,
    ///     weekday: "monday",
    /// };
    /// let transit_graph = TransitGraph::from(&feed_args)?;
    /// ```
    pub fn from(feed_args: FeedArgs) -> Result<Self, Error> {
        // perform street graph creation in a separate thread
        let walk_graph_handle =
            std::thread::spawn(move || streets::create_graph(&feed_args.pbf_path));

        // Prepare the dataframes from the GTFS feed
        let (stops_df, stop_times_df) = loaders::prepare_dataframes(
            &feed_args.gtfs_path,
            feed_args.departure,
            feed_args.duration,
            feed_args.weekday,
        )?;

        // Construct transit only graph from dataframes
        let initial_graph = loaders::new_graph(&stops_df, &stop_times_df)?;
        // retrieve the street graph from the thread
        let mut walk_graph = walk_graph_handle.join().map_err(|_| {
            Error::ThreadPanicError("Failed to join street graph thread".to_string())
        })??;

        // Merge the pedestrian graph with the transit graph (without connections)
        loaders::merge_graphs(&mut walk_graph, &initial_graph);
        // Connect transit stops in graph to walk nodes
        connectors::connect_stops_to_streets(&mut walk_graph)?;

        walk_graph.shrink_to_fit();

        Ok(walk_graph)
    }

    /// Add transit data from another GTFS feed on top of existing graph.
    /// Currntly uses straightforward logic with adddition of all stops and
    /// transit edges to initial graph.
    #[warn(unstable_features)]
    pub fn extend_with_transit(&mut self, feed_args: &FeedArgs) -> Result<(), Error> {
        let (stops_df, stop_times_df) = loaders::prepare_dataframes(
            &feed_args.gtfs_path,
            feed_args.departure,
            feed_args.duration,
            feed_args.weekday,
        )?;

        let initial_graph = loaders::new_graph(&stops_df, &stop_times_df)?;
        loaders::merge_graphs(self, &initial_graph);

        connectors::connect_stops_to_streets(self)?;

        Ok(())
    }

    /// Add a node to underlying the graph
    pub(crate) fn add_node(&mut self, node: GraphNode) -> NodeIndex {
        self.graph.add_node(node)
    }

    /// Access the internal `DiGraph` object by immutable reference
    #[must_use]
    pub const fn into_inner_graph(&self) -> &DiGraph<GraphNode, GraphEdge> {
        &self.graph
    }

    /// Add an edge to the graph
    pub(crate) fn add_edge(
        &mut self,
        source: NodeIndex,
        target: NodeIndex,
        edge: GraphEdge,
    ) -> EdgeIndex {
        self.graph.add_edge(source, target, edge)
    }

    pub(crate) fn rtree_ref(&self) -> Option<&RTree<IndexedPoint>> {
        self.rtree.as_ref()
    }

    pub(crate) fn sort_trips(&mut self) {
        for edge in self.edge_weights_mut() {
            if let GraphEdge::Transit(transit_edge) = edge {
                transit_edge.edge_trips.sort();
            }
        }
    }
}

/// Implementing `Deref` and `DerefMut` for `TransitGraph` to allow access to the internal `DiGraph`
/// This allows for the use of all the methods available for `DiGraph` on `TransitGraph`
impl Deref for TransitGraph {
    type Target = DiGraph<GraphNode, GraphEdge>;

    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}

impl DerefMut for TransitGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graph
    }
}

/// Node representing a transit stop
/// Contains the GTFS feed `stop_id` and `geometry` of the stop
/// `geometry` is a `geo::Geometry` object representing the location of the stop
#[derive(Debug, Clone, PartialEq)]
pub struct TransitNode {
    pub stop_id: String,
    pub geometry: Point,
}

/// Node representing a walkable location
/// Contains the `id` and `geometry` of the location
/// `geometry` is a `geo::Geometry` object representing the location
#[derive(Debug, Clone, PartialEq)]
pub struct WalkNode {
    pub id: NodeId,
    pub geometry: Point,
}

/// Enum representing the type of node in the graph
/// `Transit` for transit stops and `Walk` for walkable locations
#[derive(Debug, Clone, PartialEq)]
pub enum GraphNode {
    Transit(TransitNode),
    Walk(WalkNode),
}

impl GraphNode {
    pub(crate) const fn geometry(&self) -> &Point {
        match self {
            Self::Transit(transit_node) => &transit_node.geometry,
            Self::Walk(walk_node) => &walk_node.geometry,
        }
    }
}

impl Display for GraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transit(transit_node) => write!(f, "Transit, attributes: {transit_node:?}"),
            Self::Walk(walk_node) => write!(f, "Walk, attributes: {walk_node:?}"),
        }
    }
}

/// Edge representing a transit connection between two stops
/// `edge_trips` is a vector of `Trip` objects representing the trips between the stops
/// `Trip` contains the `departure_time`, `arrival_time`, `route_id`, and `wheelchair_accessible` information
/// Vector is sorted by `departure_time` at the source stop so it can be used to find the earliest trip
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitEdge {
    pub(crate) edge_trips: Vec<Trip>,
}

/// Edge representing a pedestrian connection
/// `edge_weight` is the weight of the edge in seconds
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WalkEdge {
    pub(crate) edge_weight: f64,
}

/// Enum representing the type of edge in the graph
/// `Transit` for transit connections, `Transfer` for pedestrian connections, and `Walk` for walkable connections
/// `Transit` contains a `TransitEdge` object
/// `Transfer` and `Walk` both contain a `WalkEdge` object
#[derive(Clone, PartialEq)]
pub enum GraphEdge {
    Transit(TransitEdge),
    Transfer(WalkEdge),
    Walk(WalkEdge),
}

impl GraphEdge {
    fn fmt_common(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transit(transit_edge) => {
                write!(f, "TransitEdge: {:?} ", transit_edge.edge_trips)
            }
            Self::Transfer(transfer_edge) => write!(
                f,
                "TransferEdge {{ weight: {:?} }}",
                transfer_edge.edge_weight
            ),
            Self::Walk(walk_edge) => {
                write!(f, "WalkEdge {{ weight: {:?} }}", walk_edge.edge_weight)
            }
        }
    }

    pub(crate) fn calculate_delay(&self, current_time: u32) -> f64 {
        match self {
            Self::Transit(transit_edge) => {
                let trips = &transit_edge.edge_trips;
                match trips.binary_search_by(|trip| trip.departure_time.cmp(&current_time)) {
                    Ok(index) | Err(index) if index < trips.len() => {
                        f64::from(trips[index].arrival_time - current_time)
                    }
                    // No trip found after current time, skip this edge
                    _ => f64::INFINITY,
                }
            }
            Self::Transfer(walk_edge) | Self::Walk(walk_edge) => walk_edge.edge_weight,
        }
    }
}

impl Debug for GraphEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_common(f)
    }
}

impl Display for GraphEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_common(f)
    }
}

/// Trip is a single movement between two stops.
/// Each trip is defined by the `arrival_time`, `departure_time`, `route_id`, and `wheelchair_accessible` information
/// `departure_time` is the time the trip departs from the source stop
/// `arrival_time` is the time the trip arrives at the next stop
/// `route_id` is the identifier for the route
/// `wheelchair_accessible` is a boolean indicating if the trip is wheelchair accessible
/// `Trip` is used for pathfinding in the graph
#[derive(Debug, Clone, Eq)]
pub(crate) struct Trip {
    departure_time: u32,
    arrival_time: u32,
    route_id: String,
    wheelchair_accessible: bool,
}

impl Trip {
    pub(crate) const fn new(
        departure_time: u32,
        arrival_time: u32,
        route_id: String,
        wheelchair_accessible: bool,
    ) -> Self {
        Self {
            departure_time,
            arrival_time,
            route_id,
            wheelchair_accessible,
        }
    }
}

// Implementing `PartialEq`, `Eq`, `PartialOrd`, and `Ord` for `Trip` to allow for comparisons and sorting
impl PartialEq for Trip {
    fn eq(&self, other: &Self) -> bool {
        self.departure_time == other.departure_time
    }
}

impl PartialOrd for Trip {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Trip {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.departure_time.cmp(&other.departure_time)
    }
}

impl Display for Trip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[arrival_time: {}, departure_time: {}, route_id: {}, wheelchair_accessible: {}]",
            self.arrival_time, self.departure_time, self.route_id, self.wheelchair_accessible
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_edge() {
        let mut graph = TransitGraph::new();
        let source = graph.add_node(GraphNode::Transit(TransitNode {
            stop_id: "stop1".to_string(),
            geometry: Point::new(0.0, 0.0),
        }));
        let target = graph.add_node(GraphNode::Transit(TransitNode {
            stop_id: "stop2".to_string(),
            geometry: Point::new(1.0, 1.0),
        }));
        let edge = graph.add_edge(
            source,
            target,
            GraphEdge::Transit(TransitEdge {
                edge_trips: vec![Trip::new(0, 10, "route1".to_string(), false)],
            }),
        );

        assert_eq!(graph.edge_count(), 1);
        assert_eq!(
            graph.edge_weight(edge),
            Some(&GraphEdge::Transit(TransitEdge {
                edge_trips: vec![Trip::new(0, 10, "route1".to_string(), false)],
            }))
        );
    }

    #[test]
    fn test_into_inner_graph() {
        let graph = TransitGraph::new();
        let inner_graph = graph.into_inner_graph();

        assert_eq!(inner_graph.node_count(), 0);
        assert_eq!(inner_graph.edge_count(), 0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_calculate_delay_transit() {
        let edge = GraphEdge::Transit(TransitEdge {
            edge_trips: vec![
                Trip::new(0, 10, "route1".to_string(), false),
                Trip::new(15, 20, "route2".to_string(), true),
                Trip::new(25, 30, "route3".to_string(), false),
            ],
        });

        assert!(approx::abs_diff_eq!(edge.calculate_delay(0), 10.0));
        assert!(approx::abs_diff_eq!(edge.calculate_delay(5), 15.0));
        assert!(approx::abs_diff_eq!(edge.calculate_delay(10), 10.0));
        assert!(approx::abs_diff_eq!(edge.calculate_delay(15), 5.0));
        assert!(approx::abs_diff_eq!(edge.calculate_delay(20), 10.0));
        assert!(approx::abs_diff_eq!(edge.calculate_delay(25), 5.0));
        assert_eq!(edge.calculate_delay(30), f64::INFINITY);
        assert_eq!(edge.calculate_delay(35), f64::INFINITY);
    }

    #[test]
    fn test_calculate_delay_walk() {
        let edge = GraphEdge::Walk(WalkEdge { edge_weight: 2.5 });

        assert!(approx::abs_diff_eq!(edge.calculate_delay(0), 2.5));
        assert!(approx::abs_diff_eq!(edge.calculate_delay(10), 2.5));
        assert!(approx::abs_diff_eq!(edge.calculate_delay(100), 2.5));
    }
}
