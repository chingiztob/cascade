use geo::LineString;
use geojson::{Feature, FeatureCollection, Geometry};
use serde_json::{json, map::Map};

use crate::graph::{GraphEdge, Trip};

impl GraphEdge {
    pub(crate) fn calculate_itinerary<'a>(
        &'a self,
        current_time: u32,
        geometry: Option<&'a LineString>,
        wheelchair: bool,
    ) -> Segment<'a> {
        match self {
            Self::Transit(transit_edge) => {
                let trips = &transit_edge.edge_trips;
                match trips.binary_search_by(|trip| trip.departure_time.cmp(&current_time)) {
                    Ok(index) | Err(index) if index < trips.len() => {
                        let trip = &trips[index];

                        // Skip trips that are not wheelchair accessible if wheelchair is required
                        if !trip.wheelchair_accessible && wheelchair {
                            return Segment::NoService;
                        }

                        let weight = f64::from(trip.arrival_time - current_time);

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
pub enum Segment<'a> {
    Transit {
        trip: &'a Trip,
        weight: f64,
        geometry: Option<&'a LineString>,
    },
    Pedestrian {
        weight: f64,
        geometry: Option<&'a LineString>,
    },
    NoService,
}

impl Segment<'_> {
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
pub struct Itinerary<'a> {
    pub segments: Vec<Segment<'a>>,
}

impl<'a> Itinerary<'a> {
    pub(crate) fn new() -> Itinerary<'a> {
        Itinerary {
            segments: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, segment: Segment<'a>) {
        self.segments.push(segment);
    }

    pub fn duration(&self) -> f64 {
        self.segments.iter().map(Segment::weight).sum()
    }

    pub fn to_geojson(&self) -> geojson::GeoJson {
        let mut features = vec![];

        for (i, segment) in self.segments.iter().enumerate() {
            match segment {
                Segment::Transit {
                    trip,
                    weight,
                    geometry,
                } => {
                    let mut properties = Map::new();

                    properties.insert("sequence".to_string(), i.into());
                    properties.insert("type".to_string(), "Transit".into());
                    properties.insert("weight".to_string(), weight.to_string().into());
                    properties.insert("route_id".to_string(), trip.route_id.clone().into());
                    properties.insert("departure_time".to_string(), trip.departure_time.into());
                    properties.insert("arrival_time".to_string(), trip.arrival_time.into());
                    properties.insert(
                        "wheelchair_accessible".to_string(),
                        trip.wheelchair_accessible.into(),
                    );

                    features.push(Feature {
                        geometry: geometry.map(|g| Geometry::new(g.into())),
                        properties: Some(properties),
                        id: None,
                        bbox: None,
                        foreign_members: None,
                    });
                }
                Segment::Pedestrian { weight, geometry } => {
                    let mut properties = Map::new();
                    properties.insert("sequence".to_string(), i.into());
                    properties.insert("type".to_string(), "Pedestrian".into());
                    properties.insert("weight".to_string(), json!(weight));

                    features.push(Feature {
                        geometry: geometry.map(|g| Geometry::new(g.into())),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{TransitEdge, WalkEdge};

    #[test]
    fn test_itinerary_duration() {
        let trip = Trip {
            route_id: "route_1".to_string(),
            departure_time: 1000,
            arrival_time: 1100,
            wheelchair_accessible: true,
        };

        let segment1 = Segment::Transit {
            trip: &trip,
            weight: 100.0,
            geometry: None,
        };

        let segment2 = Segment::Pedestrian {
            weight: 50.0,
            geometry: None,
        };

        let mut itinerary = Itinerary::new();
        itinerary.push(segment1);
        itinerary.push(segment2);

        let tolerance = 1e-6;
        assert!((itinerary.duration() - 150.0).abs() < tolerance);
    }

    #[test]
    fn test_itinerary_to_geojson() {
        let trip = Trip {
            route_id: "route_1".to_string(),
            departure_time: 1000,
            arrival_time: 1100,
            wheelchair_accessible: true,
        };

        let segment1 = Segment::Transit {
            trip: &trip,
            weight: 100.0,
            geometry: None,
        };

        let segment2 = Segment::Pedestrian {
            weight: 50.0,
            geometry: None,
        };

        let mut itinerary = Itinerary::new();
        itinerary.push(segment1);
        itinerary.push(segment2);

        let geojson = itinerary.to_geojson();
        if let geojson::GeoJson::FeatureCollection(fc) = geojson {
            assert_eq!(fc.features.len(), 2);
            assert_eq!(
                fc.features[0].properties.as_ref().unwrap()["type"],
                "Transit"
            );
            assert_eq!(
                fc.features[1].properties.as_ref().unwrap()["type"],
                "Pedestrian"
            );
        } else {
            panic!("Expected FeatureCollection");
        }
    }

    #[test]
    fn test_calculate_itinerary_transit() {
        let trip = Trip {
            route_id: "route_1".to_string(),
            departure_time: 1000,
            arrival_time: 1100,
            wheelchair_accessible: true,
        };

        let transit_edge = GraphEdge::Transit(TransitEdge {
            edge_trips: vec![trip],
            geometry: None,
        });

        let segment = transit_edge.calculate_itinerary(900, None, false);
        if let Segment::Transit { weight, .. } = segment {
            let tolerance = 1e-6;
            assert!((weight - 200.0).abs() < tolerance);
        } else {
            panic!("Expected Transit segment");
        }
    }

    #[test]
    fn test_calculate_itinerary_no_service() {
        let trip = Trip {
            route_id: "route_1".to_string(),
            departure_time: 1000,
            arrival_time: 1100,
            wheelchair_accessible: false,
        };

        let transit_edge = GraphEdge::Transit(TransitEdge {
            edge_trips: vec![trip],
            geometry: None,
        });

        let segment = transit_edge.calculate_itinerary(900, None, true);
        assert!(matches!(segment, Segment::NoService));

        let segment = transit_edge.calculate_itinerary(1500, None, false);
        assert!(matches!(segment, Segment::NoService));
    }

    #[test]
    fn test_calculate_itinerary_pedestrian() {
        let walk_edge = GraphEdge::Walk(WalkEdge {
            edge_weight: 50.0,
            geometry: None,
        });

        let segment = walk_edge.calculate_itinerary(0, None, false);
        if let Segment::Pedestrian { weight, .. } = segment {
            let tolerance = 1e-6;
            assert!((weight - 50.0).abs() < tolerance);
        } else {
            panic!("Expected Pedestrian segment");
        }
    }
}
