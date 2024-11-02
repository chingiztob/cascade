/*!
# Cascade

**Cascade** is a library, designed to provide
the same core functionality as `NxTransit`,
a Python library for creating and analyzing
multimodal graphs of urban transit systems using GTFS data.
Core logic of library implemented in pure Rust, resulting in
significantly higher performance and lower memory usage

See the original [NxTransit documentation](https://nxtransit.readthedocs.io/en/latest/) for an overview of the features being ported and enhanced in this version.

## OSM pbf files with street network can be prepared with [`osmium`](https://osmcode.org/osmium-tool/)

### clip data by boundary

```bash
osmium extract --polygon=/border.geojson /soure_file.pbf -o /target_file.pbf
```

### extract highways only

```bash
osmium tags-filter -o highways.osm.pbf input.pbf w/highway
```
*/

use pyo3::prelude::*;

use crate::algo::{calculate_od_matrix, shortest_path_rs, single_source_shortest_path_rs, PyPoint};
use crate::graph::{create_graph, PyTransitGraph};

pub mod algo;
pub mod graph;

#[pymodule]
fn _cascade_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(single_source_shortest_path_rs, m)?)?;
    m.add_function(wrap_pyfunction!(shortest_path_rs, m)?)?;
    m.add_function(wrap_pyfunction!(create_graph, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_od_matrix, m)?)?;
    m.add_class::<PyTransitGraph>()?;
    m.add_class::<PyPoint>()?;
    Ok(())
}
