---
myst:
  html_meta:
    "description lang=en": |
      Top-level documentation for pydata-sphinx theme, with links to the rest
      of the site..
html_theme.sidebar_secondary.remove: true
---

# Cascade (in Development)

**Cascade** is a **blazingly-fast™**  Rust-based library built, designed to provide the same core functionality as **NxTransit**, a Python library for creating and analyzing multimodal graphs of urban transit systems using GTFS data.

See the original [NxTransit documentation](https://nxtransit.readthedocs.io/en/latest/) for an overview of the features being ported and enhanced in this version.

## How to get streets data

OSM pbf fike with street network can be prepared with [`osmium`](https://osmcode.org/osmium-tool/)

- clip data by boundary

```bash
osmium extract --polygon=/border.geojson /soure_file.pbf -o /target_file.pbf
```

- extract highways only

```bash
osmium tags-filter -o highways.osm.pbf input.pbf w/highway
```

## Installation

```python
pip install cascade
```
