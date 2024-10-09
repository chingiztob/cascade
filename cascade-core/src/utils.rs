use std::fs::File;
use std::io::{self, BufWriter};

use petgraph_graphml::GraphMl;

use crate::graph::TransitGraph;

#[allow(clippy::missing_panics_doc)]
pub fn write_graphml(graph: &TransitGraph, path: &str, mode: &str) -> Result<(), io::Error> {
    let file = File::create(path).expect("Failed to create file");
    let writer = BufWriter::new(file);

    match mode {
        "edges" => {
            GraphMl::new(&graph.into_inner_graph())
                .export_edge_weights_display()
                .to_writer(writer)?;
            Ok(())
        }
        "nodes" => {
            GraphMl::new(&graph.into_inner_graph())
                .export_node_weights_display()
                .to_writer(writer)?;
            Ok(())
        }
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid mode")),
    }
}

#[must_use]
pub fn return_string() -> String {
    "Hello, world!".to_string()
}
