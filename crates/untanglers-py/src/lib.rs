use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use untanglers_core as core;
use untanglers_core::error::OptimizerError;
use untanglers_core::hierarchy_optimizer::Hierarchy;
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
  pub fn layout_optimizer_new(
    nodes_left: Vec<Vec<String>>,
    edges: Vec<Vec<(String, String, usize)>>,
  ) -> PyResult<Self> {
    let inner = core::layout_optimizer::LayoutOptimizer::<String>::new(nodes_left, edges).map_err(to_pyerr)?;
    Ok(Self { inner })
  }

  pub fn swap_nodes(&mut self, temperature: f64, max_iterations: usize, layer_index: usize) -> PyResult<i64> {
    self
      .inner
      .swap_nodes(temperature, max_iterations, layer_index)
      .map_err(to_pyerr)
  }

  pub fn cooldown(
    &mut self,
    start_temp: f64,
    end_temp: f64,
    steps: usize,
    max_iterations: usize,
    layer_index: usize,
  ) -> PyResult<i64> {
    self
      .inner
      .cooldown(start_temp, end_temp, steps, max_iterations, layer_index)
      .map_err(to_pyerr)
  }

  pub fn get_nodes(&self) -> Vec<Vec<String>> {
    self.inner.get_nodes()
  }

  pub fn count_crossings(&self) -> usize {
    self.inner.count_crossings()
  }
}

#[pyclass]
struct HierarchyOptimizer {
  inner: core::hierarchy_optimizer::HierarchyOptimizer<String>,
}

#[pymethods]
impl HierarchyOptimizer {
  #[new]
  pub fn layout_optimizer_new(
    nodes_left: Vec<Vec<String>>,
    edges: Vec<Vec<(String, String, usize)>>,
    hierarchy: Hierarchy,
  ) -> PyResult<Self> {
    let inner =
      core::hierarchy_optimizer::HierarchyOptimizer::<String>::new(nodes_left, edges, hierarchy).map_err(to_pyerr)?;
    Ok(Self { inner })
  }

  #[pyo3(signature = (temperature, max_iterations, layer_index, granularity))]
  pub fn swap_nodes(
    &mut self,
    temperature: f64,
    max_iterations: usize,
    layer_index: usize,
    granularity: Option<usize>,
  ) -> PyResult<i64> {
    self
      .inner
      .swap_nodes(temperature, max_iterations, layer_index, granularity)
      .map_err(to_pyerr)
  }

  #[pyo3(signature = (start_temp, end_temp, steps, max_iterations, layer_index, granularity))]
  pub fn cooldown(
    &mut self,
    start_temp: f64,
    end_temp: f64,
    steps: usize,
    max_iterations: usize,
    layer_index: usize,
    granularity: Option<usize>,
  ) -> PyResult<i64> {
    self
      .inner
      .cooldown(start_temp, end_temp, steps, max_iterations, layer_index, granularity)
      .map_err(to_pyerr)
  }

  pub fn optimize(
    &mut self,
    start_temp: f64,
    end_temp: f64,
    steps: usize,
    max_iterations: usize,
    passes: usize,
  ) -> PyResult<i64> {
    self
      .inner
      .optimize(start_temp, end_temp, steps, max_iterations, passes)
      .map_err(to_pyerr)
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

  #[pymodule_export]
  use crate::HierarchyOptimizer;
}
