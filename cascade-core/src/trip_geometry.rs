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

pub(crate) fn split_segments_by_stops(
    shapes_df: &DataFrame,
    stops_df: &DataFrame,
) -> Result<HashMap<(String, String), LineString<f64>>, Error> {
    let rtree = create_stops_rtree(stops_df)?;

    let grouped_shapes = shapes_df.group_by(["shape_id"])?;
    let mut segment_map = HashMap::new();

    grouped_shapes.apply(|group| {
        // Sort points by shape_pt_sequence
        let group = group.sort(["shape_pt_sequence"], SortMultipleOptions::default())?;

        // Extract latitude and longitude columns
        let lats = group.column("shape_pt_lat")?.f64()?.iter();
        let lons = group.column("shape_pt_lon")?.f64()?.iter();

        let mut current_linestring = Vec::new();
        let mut prev_stop_id: Option<String> = None;

        for (lat_opt, lon_opt) in lats.zip(lons) {
            // Handle missing values
            let lat = lat_opt.ok_or_else(|| Error::MissingValue("shape_pt_lat".to_string()))?;
            let lon = lon_opt.ok_or_else(|| Error::MissingValue("shape_pt_lon".to_string()))?;

            let point = Point::new(lon, lat);

            // Use find_closest_stop to identify nearby stops
            if let Ok((stop_id, distance)) = find_closest_stop(&point, &rtree) {
                if distance < 10.0 {
                    println!("{point:?}, {stop_id}, {distance}");
                    // If within proximity of a stop, finalize the current segment
                    if let Some(prev_id) = &prev_stop_id {
                        let key = (prev_id.clone(), stop_id.clone());
                        let linestring = LineString::from(std::mem::take(&mut current_linestring));
                        segment_map.insert(key, linestring);
                    }

                    // Update the previous stop and reset the current segment
                    prev_stop_id = Some(stop_id.clone());
                    current_linestring.push(point);
                }
            }

            // Add the current point to the line segment
            current_linestring.push(point);
        }

        // Finalize the last segment if there's a valid previous stop
        if let Some(prev_id) = prev_stop_id {
            if !current_linestring.is_empty() {
                let key = (prev_id.clone(), prev_id); // Self-loop case (single stop)
                let linestring = LineString::from(current_linestring);
                segment_map.insert(key, linestring);
            }
        }

        Ok(group)
    })?;

    // print map length
    println!("Map length is {}", segment_map.len());

    Ok(segment_map)
}
