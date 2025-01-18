use geo::LineString;
use geos::{Geom, Geometry as GeosGeometry, GeometryTypes};
use hashbrown::HashSet;
use petgraph::prelude::*;

use crate::{
    algo::single_source_shortest_path_weight, connectors::SnappedPoint, prelude::*, Error,
};

pub fn isochrone(
    graph: &TransitGraph,
    source: &SnappedPoint,
    start_time: u32,
    cutoff: f64,
    buffer_radius: f64,
) -> Result<String, Error> {
    let costs = single_source_shortest_path_weight(graph, source, start_time);

    let node_indices = costs
        .iter()
        .filter(|(_, &cost)| cost <= cutoff)
        .map(|(node, _)| *node)
        .collect::<HashSet<NodeIndex>>();

    let start = std::time::Instant::now();

    let buffers: Vec<GeosGeometry> = graph
        .iter_edge_weights(node_indices)
        .filter_map(|weight| {
            weight
                .geometry()
                .map(|line| LineString::new([line.start, line.end].into()))
                .and_then(|line| GeosGeometry::try_from(line).ok())
                .and_then(|geom: GeosGeometry| geom.buffer(buffer_radius, 2).ok())
                .filter(|buffer| buffer.geometry_type() == GeometryTypes::Polygon)
        })
        .collect();

    println!("Elapsed {:?}", start.elapsed());

    // Perform unary union on the multipolygon
    let start = std::time::Instant::now();
    let mut union_geom = GeosGeometry::create_multipolygon(buffers)?.unary_union()?;

    union_geom.normalize()?;
    println!("Elapsed 2 {:?}", start.elapsed());

    union_geom.to_wkt().map_err(Error::from)
}
