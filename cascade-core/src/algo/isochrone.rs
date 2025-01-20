use geo::{Geometry, MultiPolygon};
use geojson::{de::deserialize_geometry, ser::serialize_geometry};
use geos::{Geom, Geometry as GeosGeometry, GeometryTypes};
use hashbrown::HashSet;
use petgraph::prelude::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    algo::single_source_shortest_path_weight, connectors::SnappedPoint, prelude::*, Error,
};

fn isochrone_internal(
    graph: &TransitGraph,
    source: &SnappedPoint,
    start_time: u32,
    cutoff: f64,
    buffer_radius: f64,
) -> Result<GeosGeometry, Error> {
    let costs = single_source_shortest_path_weight(graph, source, start_time);

    let node_indices = costs
        .iter()
        .filter(|(_, &cost)| cost <= cutoff)
        .map(|(node, _)| *node)
        .collect::<HashSet<NodeIndex>>();

    let buffers: Vec<GeosGeometry> = graph
        .iter_edge_weights(node_indices)
        .filter_map(|weight| {
            weight
                .geometry()
                .and_then(|line| GeosGeometry::try_from(line).ok())
                .and_then(|geom: GeosGeometry| geom.buffer(buffer_radius, 2).ok())
                .filter(|buffer| buffer.geometry_type() == GeometryTypes::Polygon)
        })
        .collect();

    let mut union_geom = GeosGeometry::create_multipolygon(buffers)?.unary_union()?;

    union_geom.normalize()?;

    Ok(union_geom)
}

pub fn calculate_isochrone(
    graph: &TransitGraph,
    source: &SnappedPoint,
    start_time: u32,
    cutoff: f64,
    buffer_radius: f64,
) -> Result<String, Error> {
    Ok(isochrone_internal(graph, source, start_time, cutoff, buffer_radius)?.to_wkt()?)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Isochrone {
    #[serde(
        serialize_with = "serialize_geometry",
        deserialize_with = "deserialize_geometry"
    )]
    pub geometry: MultiPolygon,
    pub id: String,
}

pub fn bulk_isochrones(
    graph: &TransitGraph,
    sources: &Vec<(String, SnappedPoint)>,
    start_time: u32,
    cutoff: f64,
    buffer_radius: f64,
) -> Result<Vec<Isochrone>, Error> {
    sources
        .into_par_iter()
        .map(|(id, source)| {
            // extract closest node for snapped_point

            let union_geom: Geometry =
                isochrone_internal(graph, source, start_time, cutoff, buffer_radius)?.try_into()?;

            Ok(Isochrone {
                id: id.clone(),
                geometry: union_geom.try_into()?, // convert to geo_types Geometry
            })
        })
        .collect::<Result<Vec<Isochrone>, Error>>()
}
