# ruff: noqa: F401
# type: ignore
"""
# Cascade (in Development)

**Cascade** is a Rust-based library built using `PyO3`,
designed to provide the same core functionality as `NxTransit`,
a Python library for creating and analyzing
multimodal graphs of urban transit systems using GTFS data.

See the original [NxTransit documentation](https://nxtransit.readthedocs.io/en/latest/)
for an overview of the features being ported and enhanced in this version.
"""

from cascade._cascade_core import (
    PyPoint,
    PyTransitGraph,
    calculate_od_matrix,
    create_graph,
    detailed_itinerary,
    shortest_path_weight,
    single_source_shortest_path_weight,
)
from cascade.validators import validate_feed

__all__ = [
    "PyTransitGraph",
    "PyPoint",
    "create_graph",
    "single_source_shortest_path_weight",
    "shortest_path_weight",
    "shortest_path",
    "calculate_od_matrix",
    "validate_feed",
    "detailed_itinerary",
]
