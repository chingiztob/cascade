use geo::{prelude::*, Point};
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use rstar::primitives::GeomWithData;
use rstar::RTree;

use crate::graph::{GraphEdge, GraphNode, TransitGraph, WalkEdge};
use crate::{Error, WALK_SPEED};

/// Any object that can be snapped to the nearest node in the graph.
/// The object should have a geometry method that returns its representation as `geo::Point`.
pub(crate) trait Snappable {
    fn geometry(&self) -> &Point;
}

impl Snappable for Point {
    fn geometry(&self) -> &Point {
        self
    }
}

/// Point representing an external point that needs to be snapped to the nearest walk node.
/// Point itself is not stored in the graph, but internally it contains
/// the nearest walk node index and the distance to it. So any graph calculations
/// can be done using the nearest walk node index minus the distance to the snapped point.
#[derive(Debug)]
pub struct SnappedPoint {
    pub geometry: Point,
    index: NodeIndex,
    distance: f64,
}

impl SnappedPoint {
    pub fn init(geometry: Point, graph: &TransitGraph) -> Result<SnappedPoint, Error> {
        let rtree = graph
            .rtree_ref()
            .ok_or_else(|| Error::MissingValue("Graph spatial index is not set".to_string()))?;

        let point = snap_single_point(&geometry, rtree)?;
        Ok(point)
    }

    #[must_use]
    pub fn index(&self) -> &NodeIndex {
        &self.index
    }

    #[must_use]
    pub const fn distance(&self) -> &f64 {
        &self.distance
    }
}

/// Structure representing a graph node in the `RTree`.
/// Required to store the node index to be able to connect the transit nodes to the nearest walk nodes.
pub type IndexedPoint = GeomWithData<Point, Option<NodeIndex>>;

/// Helper function to find the nearest point in the `RTree` and calculate the distance.
/// Returns a tuple of the nearest node index and the calculated distance.
fn find_nearest_point_and_calculate_distance(
    point: &Point,
    tree: &RTree<IndexedPoint>,
) -> Result<(NodeIndex, f64), Error> {
    if let Some(nearest_point) = tree.nearest_neighbor(point) {
        let distance = point.haversine_distance(nearest_point.geom()) / WALK_SPEED;
        let node = nearest_point.data.ok_or_else(|| {
            Error::NodeNotFound(format!("Nearest node not found for point {point:?}"))
        })?;
        Ok((node, distance))
    } else {
        Err(Error::NodeNotFound(format!(
            "Nearest node not found for point {point:?}"
        )))
    }
}

/// Snaps a single point to the nearest node in the `RTree`.
pub(crate) fn snap_single_point<T: Snappable>(
    point: &T,
    tree: &RTree<IndexedPoint>,
) -> Result<SnappedPoint, Error> {
    let (index, distance) = find_nearest_point_and_calculate_distance(point.geometry(), tree)?;

    Ok(SnappedPoint {
        geometry: *point.geometry(),
        index,
        distance,
    })
}

pub(crate) fn build_rtree(graph: &DiGraph<GraphNode, GraphEdge>) -> RTree<IndexedPoint> {
    let index_geo_vec: Vec<IndexedPoint> = graph
        .node_indices()
        .map(|node| {
            let node_data = graph.node_weight(node).unwrap();
            let node_point: Point = *node_data.geometry();
            IndexedPoint::new(node_point, Some(node))
        })
        .collect();

    RTree::bulk_load(index_geo_vec)
}

/// Connects Transit nodes (stops) to the nearest walk nodes.
pub(crate) fn connect_stops_to_streets(graph: &mut TransitGraph) -> Result<(), Error> {
    let rtree: RTree<IndexedPoint> = graph.rtree_ref().unwrap().clone();

    for node in graph.node_indices() {
        // check if there is already a transfer edge
        // This is required to avoid creating duplicate transfer edges
        // when merging multiple transit graphs on top of main street graph
        // (Work in progress)
        if graph
            .edges(node)
            .any(|edge| matches!(edge.weight(), GraphEdge::Transfer(_)))
        {
            continue;
        }

        let weight = graph
            .node_weight(node)
            .ok_or_else(|| Error::MissingValue("Node weight not found".to_string()))?;

        if let GraphNode::Transit(_) = weight {
            if let Ok((nearest_point_index, distance)) =
                find_nearest_point_and_calculate_distance(weight.geometry(), &rtree)
            {
                let edge = GraphEdge::Transfer(WalkEdge {
                    edge_weight: distance,
                });

                graph.add_edge(node, nearest_point_index, edge.clone());
                graph.add_edge(nearest_point_index, node, edge);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use osm4routing::NodeId;

    use crate::graph::WalkNode;

    use super::*;

    #[cfg(test)]
    impl GraphNode {
        pub(crate) fn new(geometry: Point) -> Self {
            GraphNode::Walk(WalkNode {
                id: NodeId(0),
                geometry,
            })
        }
    }

    #[test]
    fn test_build_rtree() {
        let mut graph = DiGraph::<GraphNode, GraphEdge>::new();
        let node1 = graph.add_node(GraphNode::new(Point::new(0.0, 0.0)));
        let node2 = graph.add_node(GraphNode::new(Point::new(1.0, 1.0)));
        let node3 = graph.add_node(GraphNode::new(Point::new(2.0, 2.0)));

        let rtree = build_rtree(&graph);

        assert_eq!(
            rtree.nearest_neighbor(&Point::new(0.4, 0.4)),
            Some(&IndexedPoint::new(Point::new(0.0, 0.0), Some(node1)))
        );

        assert_eq!(
            rtree.nearest_neighbor(&Point::new(1.4, 1.4)),
            Some(&IndexedPoint::new(Point::new(1.0, 1.0), Some(node2)))
        );

        assert_eq!(
            rtree.nearest_neighbor(&Point::new(2.5, 2.5)),
            Some(&IndexedPoint::new(Point::new(2.0, 2.0), Some(node3)))
        );
    }

    #[test]
    #[allow(unused)]
    fn test_snap_single_point() {
        let mut graph = DiGraph::<GraphNode, GraphEdge>::new();

        let node1 = graph.add_node(GraphNode::new(Point::new(0.0, 0.0)));
        let node2 = graph.add_node(GraphNode::new(Point::new(1.0, 1.0)));
        let node3 = graph.add_node(GraphNode::new(Point::new(2.0, 2.0)));

        let rtree = build_rtree(&graph);

        let point = Point::new(0.4, 0.4);
        let snapped_point = snap_single_point(&point, &rtree).unwrap();
        assert_eq!(snapped_point.geometry, point);
        assert_eq!(*snapped_point.index(), node1);
        assert!(
            (*snapped_point.distance()
                - point.haversine_distance(&Point::new(0.0, 0.0)) / WALK_SPEED)
                .abs()
                < f64::EPSILON
        );
    }
}
