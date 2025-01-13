API Documentation
=================

This module provides algorithms for finding shortest paths in time-dependent transit graphs. It includes functions to:

- **Compute the shortest paths from a source node to all other nodes** using Dijkstra's algorithm (:func:`single_source_shortest_path`).
- **Find the shortest path weight** between a source and target node (:func:`shortest_path_weight`).
- **Retrieve the actual shortest path** between a source and target node as a sequence of node indices (:func:`shortest_path`).
- **Calculate an origin-destination (OD) matrix** for a set of points, providing the shortest path weights between all pairs of points (:func:`calculate_od_matrix`).

The module also defines a :class:`PyPoint` class, a Python wrapper for passing coordinates with an ID to the Rust backend, facilitating seamless integration between Rust and Python components.

Examples
--------

.. code-block:: python

   from cascade import create_graph, single_source_shortest_path, shortest_path_weight, shortest_path, PyPoint

   gtfs_path = "path/to/City_GTFS"
   pbf_path = "path/to/City.pbf"
   departure = 0
   duration = 86400
   weekday = "monday"

   graph = create_graph(gtfs_path, pbf_path, departure, duration, weekday)

   (source_x, source_y) = (59.851960, 30.221418)
   (target_x, target_y) = (59.978989, 30.502047)

   print(
       cascade.shortest_path_weight(
           graph=graph,
           dep_time=43200,
           source_x=source_x,
           source_y=source_y,
           target_x=target_x,
           target_y=target_y,
       )
   )

.. autofunction:: cascade.create_graph
.. autofunction:: cascade.single_source_shortest_path_weight
.. autofunction:: cascade.shortest_path_weight
.. autofunction:: cascade.shortest_path
.. autoclass:: cascade.PyPoint
   :members:
