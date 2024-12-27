use crate::graph::{GraphEdge, Trip};

impl GraphEdge {
    pub(crate) fn calculate_itinerary(&self, current_time: u32) -> Segment {
        match self {
            Self::Transit(transit_edge) => {
                let trips = &transit_edge.edge_trips;
                match trips.binary_search_by(|trip| trip.departure_time.cmp(&current_time)) {
                    Ok(index) | Err(index) if index < trips.len() => {
                        let trip = trips[index].clone();

                        let weight = f64::from(trips[index].arrival_time - current_time);

                        Segment::Transit { trip, weight }
                    }
                    // No trip found after current time, skip this edge
                    _ => Segment::NoService,
                }
            }
            Self::Transfer(walk_edge) | Self::Walk(walk_edge) => {
                Segment::Pedestrian(walk_edge.edge_weight)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Segment {
    Transit { trip: Trip, weight: f64 },
    Pedestrian(f64),
    NoService,
}

impl Segment {
    pub(crate) fn weight(&self) -> f64 {
        match self {
            Segment::Pedestrian(weight) | Segment::Transit { trip: _, weight } => *weight,
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
}
