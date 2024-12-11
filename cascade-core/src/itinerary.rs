use crate::graph::{GraphEdge, Trip};

#[allow(unused)]
impl GraphEdge {
    pub fn calculate_itinerary(&self, current_time: u32) -> f64 {
        match self {
            Self::Transit(transit_edge) => {
                let trips = &transit_edge.edge_trips;
                match trips.binary_search_by(|trip| trip.departure_time.cmp(&current_time)) {
                    Ok(index) | Err(index) if index < trips.len() => {
                        let trip = &trips[index];
                        let route_id = &trip.route_id;
                        let departure_time = &trip.departure_time;
                        let arrival_time = &trip.arrival_time;
                        let weight = f64::from(arrival_time - current_time);

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

#[allow(unused)]
enum ItineraryData {
    Transit(f64, Trip),
    Pedestrian(f64),
}
