use thiserror::Error;

#[derive(Debug, Error)]
pub enum OptimizerError {
  #[error("expected one hierarchy for each node layer, got {hierarchy} vs {layers}")]
  HierarchyMismatch { hierarchy: usize, layers: usize },

  #[error("Hierarchy at layer {layer_index}, level {granularity} has total size {size} != node count {node_count}")]
  HierarchySizeMismatch { layer_index: usize, granularity: usize, size: usize, node_count: usize },

  #[error("Hierarchy at layer {layer_index}, level {granularity} does not align with its child level, {next_size} > {self_size}")]
  HierarchyAlignmentError { layer_index: usize, granularity: usize, next_size: usize, self_size: usize },

  #[error("expected n-1 edge layers for n node layers, got E={edges} vs N={layers}")]
  EdgeLayerMismatch { edges: usize, layers: usize },

  #[error("Edges contains missing node {node_name:?} at layer {layer_index}")]
  MissingNode { node_name: String, layer_index: usize},

  #[error("Layer index out of range: {layer_index} > {layer_count} - 1")]
  InvalidLayer { layer_index: usize, layer_count: usize},
}
