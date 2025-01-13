Cascade
=======

Cascade is a fast and portable python library for working with urban transit networks implemented with Rust.
It combines GTFS (transit schedules) and OpenStreetMap (street data) to create multimodal transit models.

Features
--------
- Check if GTFS files are valid for routing
- Create transit graphs from GTFS and OSM data.
- Find the fastest routes between places based on schedules.
- Fast and efficient, built with Rust.

Installation
------------

This library is portable, have zero python dependencies and
can be easily installed with Pip:

.. code-block:: bash

   pip install cascade


Preparing OSM Data
------------------

To work with OSM data, you can prepare PBF files using the `osmium` tool.

Clipping Data by Boundary
~~~~~~~~~~~~~~~~~~~~~~~~~

Extract data within a specific geographic boundary defined by a GeoJSON polygon:

.. code-block:: bash

   osmium extract --polygon=border.geojson source_file.pbf -o target_file.pbf

Extract Highways Only
~~~~~~~~~~~~~~~~~~~~~

Extract highways only from a PBF file:

.. code-block:: bash

   osmium tags-filter -o highways.osm.pbf input.pbf w/highway

Example Usage in Python
-----------------------

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

Documentation
-------------

.. toctree::
   :maxdepth: 2
   :caption: Contents:

   cascade.rst

.. toctree::
   :maxdepth: 1
   :caption: Demo

   example_2.ipynb

License
-------

Package is open source and licensed under the MIT OR Apache-2.0 license .
OpenStreetMap's open data license requires that derivative works provide proper attribution.
For more details, see the `OpenStreetMap copyright page <https://www.openstreetmap.org/copyright/>`_.
