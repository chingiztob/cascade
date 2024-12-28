use geo::LineString;
use geojson::{Feature, FeatureCollection, Geometry, Value};

use crate::graph::{GraphEdge, Trip};
use serde_json::map::Map;

impl GraphEdge {
    pub(crate) fn calculate_itinerary(&self, current_time: u32, geometry: LineString) -> Segment {
        match self {
            Self::Transit(transit_edge) => {
                let trips = &transit_edge.edge_trips;
                match trips.binary_search_by(|trip| trip.departure_time.cmp(&current_time)) {
                    Ok(index) | Err(index) if index < trips.len() => {
                        let trip = trips[index].clone();

                        let weight = f64::from(trips[index].arrival_time - current_time);

                        Segment::Transit {
                            trip,
                            weight,
                            geometry,
                        }
                    }
                    // No trip found after current time, skip this edge
                    _ => Segment::NoService,
                }
            }
            Self::Transfer(walk_edge) | Self::Walk(walk_edge) => Segment::Pedestrian {
                weight: walk_edge.edge_weight,
                geometry,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Segment {
    Transit {
        trip: Trip,
        weight: f64,
        geometry: LineString,
    },
    Pedestrian {
        weight: f64,
        geometry: LineString,
    },
    NoService,
}

impl Segment {
    pub(crate) fn weight(&self) -> f64 {
        match self {
            Segment::Pedestrian { weight, .. } | Segment::Transit { weight, .. } => *weight,
            // Loop must be continued and `weight()` call point
            // - never reached
            Segment::NoService => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Itinerary {
    pub travel: Vec<Segment>,
}

impl Itinerary {
    pub(crate) fn new() -> Itinerary {
        Itinerary { travel: Vec::new() }
    }

    pub(crate) fn push(&mut self, segment: Segment) {
        self.travel.push(segment);
    }

    pub fn duration(&self) -> f64 {
        self.travel.iter().map(Segment::weight).sum()
    }

    pub fn combined_geometry(&self) -> LineString {
        let mut combined_coords = Vec::new();

        for segment in &self.travel {
            let geometry = match segment {
                Segment::Pedestrian { geometry, .. } | Segment::Transit { geometry, .. } => {
                    geometry
                }
                Segment::NoService => unreachable!("Unserviced segment traversed"),
            };

            if let Some(last_point) = combined_coords.last() {
                // Skip duplicate point at boundary
                if last_point == geometry.0.first().unwrap() {
                    combined_coords.extend(geometry.0.iter().skip(1));
                } else {
                    combined_coords.extend_from_slice(&geometry.0);
                }
            } else {
                combined_coords.extend_from_slice(&geometry.0);
            }
        }

        LineString(combined_coords)
    }
}

impl Itinerary {
    pub fn to_geojson(&self) -> geojson::GeoJson {
        let mut features = vec![];

        for segment in &self.travel {
            match segment {
                Segment::Transit {
                    trip,
                    weight,
                    geometry,
                } => {
                    let mut properties = Map::new();
                    properties.insert("type".to_string(), "Transit".into());
                    properties.insert("weight".to_string(), weight.to_string().into());
                    properties.insert("route_id".to_string(), trip.route_id.clone().into());
                    properties.insert(
                        "departure_time".to_string(),
                        trip.departure_time.to_string().into(),
                    );
                    properties.insert(
                        "arrival_time".to_string(),
                        trip.arrival_time.to_string().into(),
                    );
                    properties.insert(
                        "wheelchair_accessible".to_string(),
                        trip.wheelchair_accessible.to_string().into(),
                    );

                    features.push(Feature {
                        geometry: Some(geometry_to_geojson(geometry)),
                        properties: Some(properties),
                        id: None,
                        bbox: None,
                        foreign_members: None,
                    });
                }
                Segment::Pedestrian { weight, geometry } => {
                    let mut properties = Map::new();
                    properties.insert("type".to_string(), "Pedestrian".into());
                    properties.insert("weight".to_string(), weight.to_string().into());

                    features.push(Feature {
                        geometry: Some(geometry_to_geojson(geometry)),
                        properties: Some(properties),
                        id: None,
                        bbox: None,
                        foreign_members: None,
                    });
                }
                Segment::NoService => unreachable!(),
            }
        }

        geojson::GeoJson::FeatureCollection(FeatureCollection {
            features,
            bbox: None,
            foreign_members: None,
        })
    }
}

fn geometry_to_geojson(geometry: &LineString<f64>) -> Geometry {
    let coords: Vec<Vec<f64>> = geometry
        .coords()
        .map(|coord| vec![coord.x, coord.y])
        .collect();

    Geometry::new(Value::LineString(coords))
}
