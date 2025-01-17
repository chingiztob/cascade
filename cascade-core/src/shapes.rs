use ahash::{HashMap, HashMapExt};
use geo::{prelude::*, LineString, Point};
use polars::chunked_array::ops::SortMultipleOptions;
use polars::prelude::*;
use rstar::primitives::GeomWithData;
use rstar::RTree;

use crate::Error;

type RTreeStop = GeomWithData<Point, String>;

fn create_stops_rtree(stops_df: &DataFrame) -> Result<RTree<RTreeStop>, Error> {
    let mut rtree_source = Vec::new();

    let stop_ids = stops_df.column("stop_id")?.str()?.iter();
    let stop_lons = stops_df.column("stop_lon")?.f64()?.iter();
    let stop_lats = stops_df.column("stop_lat")?.f64()?.iter();

    for (stop_id, (stop_lon, stop_lat)) in stop_ids.zip(stop_lons.zip(stop_lats)) {
        let stop_id = stop_id.ok_or_else(|| Error::MissingValue("stop_id".to_string()))?;
        let stop_lon = stop_lon.ok_or_else(|| Error::MissingValue("stop_lon".to_string()))?;
        let stop_lat = stop_lat.ok_or_else(|| Error::MissingValue("stop_lat".to_string()))?;

        let stop = RTreeStop::new(Point::new(stop_lon, stop_lat), stop_id.to_string());
        rtree_source.push(stop);
    }

    Ok(RTree::bulk_load(rtree_source))
}

fn find_closest_stop(point: &Point, rtree: &RTree<RTreeStop>) -> Result<(String, f64), Error> {
    if let Some(nearest_stop) = rtree.nearest_neighbor(point) {
        let distance = Haversine::distance(*point, *nearest_stop.geom());
        let stop_id = nearest_stop.data.clone();
        Ok((stop_id, distance))
    } else {
        Err(Error::NodeNotFound(format!(
            "Nearest stop not found for point {point:?}"
        )))
    }
}

pub(crate) fn process_shapes(
    shapes_df: &DataFrame,
    stops_df: &DataFrame,
) -> Result<HashMap<(String, String), LineString<f64>>, Error> {
    let rtree = create_stops_rtree(stops_df)?;

    // Initialize a map to store geometry between stop pairs
    let mut segment_map = HashMap::new();

    // Group shapes by shape_id
    let grouped_shapes = shapes_df.group_by(["shape_id"])?;

    grouped_shapes.apply(|group| {
        // Extract the shape_id

        // Sort the shape points by their sequence
        let group = group.sort(["shape_pt_sequence"], SortMultipleOptions::default())?;

        // Get the points (latitude and longitude) and form a full LineString
        let lats = group.column("shape_pt_lat")?.f64()?.iter();
        let lons = group.column("shape_pt_lon")?.f64()?.iter();
        let mut full_linestring: Vec<Point<f64>> = Vec::new();

        for (lat, lon) in lats.zip(lons) {
            let lat = lat.ok_or_else(|| Error::MissingValue("shape_pt_lat missing".to_string()))?;
            let lon = lon.ok_or_else(|| Error::MissingValue("shape_pt_lon missing".to_string()))?;
            full_linestring.push(Point::new(lon, lat));
        }

        // Process the points to identify stop pairs
        let mut current_linestring = Vec::new();
        let mut prev_stop_id: Option<String> = None;

        for point in &full_linestring {
            // Find the closest stop to the current point
            if let Ok((stop_id, distance)) = find_closest_stop(point, &rtree) {
                if distance < 5.0 {
                    // If close to a stop, finalize the current segment
                    if let Some(prev_id) = &prev_stop_id {
                        // Create a new segment between stops
                        let key = (prev_id.clone(), stop_id.clone());
                        let linestring = LineString::from(std::mem::take(&mut current_linestring));
                        segment_map.insert(key, linestring);
                    }

                    // Update the previous stop and reset the current segment
                    prev_stop_id = Some(stop_id.clone());
                }
            }

            // Add the current point to the current segment
            current_linestring.push(*point);
        }

        // Handle the final segment if it exists
        if let Some(prev_id) = prev_stop_id {
            if !current_linestring.is_empty() {
                let key = (prev_id.clone(), prev_id.clone()); // Self-loop case for the last stop
                let linestring = LineString::from(current_linestring);
                segment_map.insert(key, linestring);
            }
        }

        Ok(group)
    })?;

    Ok(segment_map)
}
