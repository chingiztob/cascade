import cascade

gtfs_path = "/home/chingiz/Rust/py_rust/cascade/cascade-bin/files/Saint_Petersburg"
pbf_path = "/home/chingiz/Rust/osm/roads_SZ.pbf"
departure = 0
duration = 60000
weekday = "sunday"

graph = cascade.create_graph(gtfs_path, pbf_path, departure, duration, weekday)

print(cascade.demo_wrapper(graph))
