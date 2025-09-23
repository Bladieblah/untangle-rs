use thiserror::Error;

#[derive(Debug, Error)]
pub enum OptimizerError {
  #[error("expected one hierarchy for each node layer, got {hierarchy} vs {layers}")]
  HierarchyMismatch { hierarchy: usize, layers: usize },

  #[error("found a group with size 0 in the hierarchy at layer {layer_index} level {granularity}")]
  EmptyGroup { layer_index: usize, granularity: usize },

  #[error("Hierarchy at layer {layer_index}, level {granularity} has total size {size} != node count {node_count}")]
  HierarchySizeMismatch {
    layer_index: usize,
    granularity: usize,
    size: usize,
    node_count: usize,
  },

  #[error("Hierarchy at layer {layer_index}, level {granularity} does not align with its child level, {next_size} > {self_size}")]
  HierarchyAlignmentError {
    layer_index: usize,
    granularity: usize,
    next_size: usize,
    self_size: usize,
  },

  #[error("expected n-1 edge layers for n node layers, got E={edges} vs N={layers}")]
  EdgeLayerMismatch { edges: usize, layers: usize },

  #[error("Duplicate edge in layer {layer_index}: ({node_a}, {node_b})")]
  DuplicateEdge {
    node_a: String,
    node_b: String,
    layer_index: usize,
  },

  #[error("Edges contains missing node {node_name:?} at layer {layer_index}")]
  MissingNode { node_name: String, layer_index: usize },

  #[error("Layer index out of range: {layer_index} > {layer_count} - 1")]
  InvalidLayer { layer_index: usize, layer_count: usize },
}
