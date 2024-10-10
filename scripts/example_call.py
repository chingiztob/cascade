import cascade

gtfs_path = "/home/chingiz/Rust/py_rust/cascade/cascade-bin/files/Saint_Petersburg"
pbf_path = "/home/chingiz/Rust/osm/roads_SZ.pbf"
departure = 0
duration = 90000
weekday = "monday"

graph = cascade.create_graph(gtfs_path, pbf_path, departure, duration, weekday)

print(cascade.single_source_shortest_path(graph, 43200, 30.320234, 59.875912))

print(cascade.shortest_path(graph, 75000))
