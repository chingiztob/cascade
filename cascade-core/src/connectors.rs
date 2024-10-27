use geo::{prelude::*, Point};
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use rstar::Point as RstarPoint;
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

    fn new(geometry: Point, index: NodeIndex, distance: f64) -> SnappedPoint {
        SnappedPoint {
            geometry,
            index,
            distance,
        }
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

/// Helper function to find the nearest point in the `RTree` and calculate the distance.
/// Returns a tuple of the nearest node index and the calculated distance.
fn find_nearest_point_and_calculate_distance(
    point: &IndexedPoint,
    tree: &RTree<IndexedPoint>,
) -> Result<(NodeIndex, f64), Error> {
    if let Some(nearest_point) = tree.nearest_neighbor(point) {
        let distance = point.geometry.haversine_distance(&nearest_point.geometry) / WALK_SPEED;
        let node = nearest_point.index.ok_or_else(|| {
            Error::NodeNotFound(format!(
                "Nearest node not found for point {:?}",
                point.geometry
            ))
        })?;
        Ok((node, distance))
    } else {
        Err(Error::NodeNotFound(format!(
            "Nearest node not found for point {:?}",
            point.geometry
        )))
    }
}

/// Snaps a single point to the nearest node in the `RTree`.
pub(crate) fn snap_single_point<T: Snappable>(
    point: &T,
    tree: &RTree<IndexedPoint>,
) -> Result<SnappedPoint, Error> {
    let point_to_snap = IndexedPoint {
        index: None,
        geometry: *point.geometry(),
    };

    let (nearest_node, distance) = find_nearest_point_and_calculate_distance(&point_to_snap, tree)?;

    Ok(SnappedPoint::new(
        *point.geometry(),
        nearest_node,
        distance,
    ))
}

/// Structure representing a graph node in the `RTree`.
/// The `RTree` requires a structure that implements the `Point` trait.
/// Also we need to store the node index to be able to connect the transit nodes to the nearest walk nodes.
#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) struct IndexedPoint {
    pub(crate) index: Option<NodeIndex>,
    pub(crate) geometry: Point,
}

impl RstarPoint for IndexedPoint {
    type Scalar = f64;
    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        Self {
            index: None,
            geometry: Point::new(generator(0), generator(1)),
        }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        self.geometry.nth(index)
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        self.geometry.nth_mut(index)
    }
}

pub(crate) fn build_rtree(graph: &DiGraph<GraphNode, GraphEdge>) -> RTree<IndexedPoint> {
    let index_geo_vec: Vec<IndexedPoint> = graph
        .node_indices()
        .map(|node| {
            let node_data = graph.node_weight(node).unwrap();
            let node_point: Point = *node_data.geometry();
            IndexedPoint {
                index: Some(node),
                geometry: node_point,
            }
        })
        .collect();

    RTree::bulk_load(index_geo_vec)
}

/// Connects Transit nodes (stops) to the nearest walk nodes.
pub(crate) fn connect_stops_to_streets(graph: &mut TransitGraph) -> Result<(), Error> {
    let rtree = graph.rtree_ref().unwrap().clone();

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
            let node_point = IndexedPoint {
                index: Some(node),
                geometry: *weight.geometry(),
            };

            if let Ok((nearest_point_index, distance)) =
                find_nearest_point_and_calculate_distance(&node_point, &rtree)
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
            rtree.nearest_neighbor(&IndexedPoint {
                index: None,
                geometry: Point::new(0.4, 0.4),
            }),
            Some(&IndexedPoint {
                index: Some(node1),
                geometry: Point::new(0.0, 0.0),
            })
        );

        assert_eq!(
            rtree.nearest_neighbor(&IndexedPoint {
                index: None,
                geometry: Point::new(1.4, 1.4),
            }),
            Some(&IndexedPoint {
                index: Some(node2),
                geometry: Point::new(1.0, 1.0),
            })
        );

        assert_eq!(
            rtree.nearest_neighbor(&IndexedPoint {
                index: None,
                geometry: Point::new(2.5, 2.5),
            }),
            Some(&IndexedPoint {
                index: Some(node3),
                geometry: Point::new(2.0, 2.0),
            })
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
