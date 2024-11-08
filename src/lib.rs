/*!
# Cascade

Cascade is a Rust library for creating and analyzing multimodal graphs
of urban transit systems using GTFS (General Transit Feed Specification)
and OpenStreetMap (OSM) data. Core functionality is implemented in Rust
allowing for high performance and low memory usage.

## Features

- **Create transit graphs** from GTFS feeds and OSM PBF files.
- **Integrate transit and pedestrian networks** for multimodal analysis.
- **High performance and low memory usage** due to Rust's speed and safety.

## Preparing OSM Data

To work with OSM data, you can prepare PBF files using the [`osmium`](https://osmcode.org/osmium-tool/) tool.

### Clipping Data by Boundary

Extract data within a specific geographic boundary defined by a `GeoJSON` polygon:

```bash
osmium extract --polygon=border.geojson source_file.pbf -o target_file.pbf
```

### extract highways only

```bash
osmium tags-filter -o highways.osm.pbf input.pbf w/highway
```
### Example Usage in Python
```python
from cascade import create_graph, PyTransitGraph

gtfs_path = "path/to/City_GTFS"
pbf_path = "path/to/City.pbf"
departure = 0
duration = 86400
weekday = "monday"

graph = create_graph(gtfs_path, pbf_path, departure, duration, weekday)
```
*/

use pyo3::prelude::*;

use crate::algo::{
    calculate_od_matrix, shortest_path, shortest_path_weight, single_source_shortest_path, PyPoint,
};
use crate::graph::{create_graph, PyTransitGraph};

pub mod algo;
pub mod graph;

#[pymodule]
fn _cascade_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(single_source_shortest_path, m)?)?;
    m.add_function(wrap_pyfunction!(shortest_path, m)?)?;
    m.add_function(wrap_pyfunction!(shortest_path_weight, m)?)?;
    m.add_function(wrap_pyfunction!(create_graph, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_od_matrix, m)?)?;
    m.add_class::<PyTransitGraph>()?;
    m.add_class::<PyPoint>()?;
    Ok(())
}
