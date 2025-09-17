use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use untanglers_core::error::OptimizerError;
use untanglers_core as core;
use untanglers_core::optimizer_ops::OptimizerOps;

fn to_pyerr(err: OptimizerError) -> PyErr {
    PyValueError::new_err(err.to_string())
}

#[pyclass]
struct LayoutOptimizer {
  inner: core::layout_optimizer::LayoutOptimizer<String>,
}

#[pymethods]
impl LayoutOptimizer {
  #[new]
  pub fn crossings_new(nodes_left: Vec<Vec<String>>, edges: Vec<Vec<(String, String, usize)>>) -> PyResult<Self> {
    let inner = core::layout_optimizer::LayoutOptimizer::<String>::new(nodes_left, edges).map_err(to_pyerr)?;
    Ok(Self { inner })
  }

  pub fn swap_nodes(&mut self, layer_index: usize, max_iterations: usize, temperature: f64) {
    self.inner.swap_nodes(layer_index, max_iterations, temperature);
  }

  pub fn cooldown(&mut self, start_temp: f64, end_temp: f64, steps: usize, max_iterations: usize, layer_index: usize) {
    self
      .inner
      .cooldown(start_temp, end_temp, steps, max_iterations, layer_index);
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
