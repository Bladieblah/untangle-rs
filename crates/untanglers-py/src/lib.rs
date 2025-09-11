use pyo3::prelude::*;
use untanglers_core as core;

#[pyclass]
struct LayoutOptimizer {
  inner: core::layout_optimizer::LayoutOptimizer<String>,
}

#[pymethods]
impl LayoutOptimizer {
  #[new]
  pub fn crossings_new(nodes_left: Vec<Vec<String>>, edges: Vec<Vec<(String, String, usize)>>) -> Self {
    let inner = core::layout_optimizer::LayoutOptimizer::<String>::new(nodes_left, edges);
    Self { inner }
  }

  pub fn swap_nodes(&mut self, layer_index: usize, max_iterations: usize, temperature: f64) {
    self.inner.swap_nodes(layer_index, max_iterations, temperature);
  }

  pub fn cooldown(&mut self, layer_index: usize, max_iterations: usize, start_temp: f64, end_temp: f64, steps: usize) {
    self
      .inner
      .cooldown(layer_index, max_iterations, start_temp, end_temp, steps);
  }

  pub fn get_nodes(&self) -> Vec<Vec<String>> {
    self.inner.get_nodes()
  }

  pub fn count_crossings(&self) -> usize {
    self.inner.count_crossings()
  }
}

#[pymodule]
mod untanglers {
  #[pymodule_export]
  use crate::LayoutOptimizer;
}
