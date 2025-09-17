use thiserror::Error;

#[derive(Debug, Error)]
pub enum OptimizerError {
  #[error("expected one hierarchy for each node layer, got {hierarchy} vs {layers}")]
  HierarchyMismatch { hierarchy: usize, layers: usize },

  #[error("expected n-1 edge layers for n node layers, got E={edges} vs N={layers}")]
  EdgeLayerMismatch { edges: usize, layers: usize },

  #[error("Edges contains missing node {node_name:?} at layer {layer_index}")]
  MissingNode { node_name: String, layer_index: usize},
}
