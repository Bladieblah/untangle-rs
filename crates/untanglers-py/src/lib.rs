use pyo3::prelude::*;
use untanglers_core as core;

#[pyclass]
struct LayoutOptimizer {
  inner: core::optimizer::LayoutOptimizer<String>,
}

#[pymethods]
impl LayoutOptimizer {
  #[new]
  pub fn crossings_new(nodes_left: Vec<String>, nodes_right: Vec<String>, edges: Vec<(String, String, usize)>) -> Self {
    let inner = core::optimizer::LayoutOptimizer::<String>::new(nodes_left, nodes_right, edges);
    Self { inner }
  }

  pub fn swap_nodes_left(&mut self, max_iterations: usize, temperature: f64) {
    self.inner.swap_nodes_left(max_iterations, temperature);
  }

  pub fn swap_nodes_right(&mut self, max_iterations: usize, temperature: f64) {
    self.inner.swap_nodes_right(max_iterations, temperature);
  }

  pub fn get_nodes(&self) -> (Vec<String>, Vec<String>) {
    self.inner.get_nodes()
  }

  pub fn count_crossings(&self) -> usize {
    self.inner.count_crossings()
  }
}

/// A Python module implemented in Rust.
#[pymodule]
mod untanglers {
  #[pymodule_export]
  use crate::LayoutOptimizer;
}
