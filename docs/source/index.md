---
myst:
  html_meta:
    "description lang=en": |
      Top-level documentation for pydata-sphinx theme, with links to the rest
      of the site..
html_theme.sidebar_secondary.remove: true
---

# Cascade

Cascade is a Rust library for creating and analyzing multimodal graphs
of urban transit systems using GTFS (General Transit Feed Specification)
and OpenStreetMap (OSM) data. Core functionality is implemented in Rust
allowing for high performance and low memory usage.

The package enables the detailed analysis of transit systems by incorporating time-dependent nature of public transportation. This includes:

- GTFS feed validation.
- Shortest path calculations with time-specific departures.
- Generating travel time matrices to evaluate travel durations between multiple network points.
- More features are planned for future updates.

## Preparing OSM Data

To work with OSM data, you can prepare PBF files using the [`osmium`](https://osmcode.org/osmium-tool/) tool.

Extract data within a specific geographic boundary defined by a `GeoJSON` polygon:

```bash
osmium extract --polygon=border.geojson source_file.pbf -o target_file.pbf
```

Extract highways only

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

## Installation

```python
pip install cascade
```

## Contents

```{toctree}
  :maxdepth: 2
  :caption: Contents:
```

## Documentation

```{toctree}
  :maxdepth: 1

cascade.md
example_2.ipynb
```
