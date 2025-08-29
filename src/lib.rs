use pyo3::prelude::*;

pub mod crossings;
pub mod utils;

#[pyclass]
struct Crossings {
  inner: crossings::Crossings<String>,
}

#[pymethods]
impl Crossings {
  #[new]
  pub fn crossings_new(nodes_left: Vec<String>, nodes_right: Vec<String>, edges: Vec<(String, String, usize)>) -> Self {
    let inner = crossings::Crossings::<String>::new(nodes_left, nodes_right, edges);
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
  use crate::Crossings;
}
