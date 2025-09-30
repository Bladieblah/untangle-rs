mod threading;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::sync::{Arc, Mutex};
use untanglers_core as core;
use untanglers_core::error::OptimizerError;
use untanglers_core::hierarchy_optimizer::Hierarchy;
use untanglers_core::optimizer_ops::OptimizerOps;
use untanglers_core::utils;

use crate::threading::run_in_thread;

fn to_pyerr(err: OptimizerError) -> PyErr {
  PyValueError::new_err(err.to_string())
}

macro_rules! optimizers {
  ($ty: ty, $name1: ident, $name2: ident) => {
    #[pyclass]
    struct $name1 {
      inner: Arc<Mutex<core::layout_optimizer::LayoutOptimizer<$ty>>>,
    }

    #[pymethods]
    impl $name1 {
      #[new]
      pub fn layout_optimizer_new(nodes_left: Vec<Vec<$ty>>, edges: Vec<Vec<($ty, $ty, usize)>>) -> PyResult<Self> {
        let inner = core::layout_optimizer::LayoutOptimizer::<$ty>::new(nodes_left, edges).map_err(to_pyerr)?;
        Ok(Self {
          inner: Arc::new(Mutex::new(inner)),
        })
      }

      pub fn swap_nodes(&mut self, temperature: f64, max_iterations: usize, layer_index: usize) -> PyResult<usize> {
        let inner = Arc::clone(&self.inner);
        Python::with_gil(|py| {
          run_in_thread(py, move || {
            let mut guard = inner.lock().unwrap();
            guard.swap_nodes(temperature, max_iterations, layer_index)
          })
        })
      }

      pub fn cooldown(
        &mut self,
        py: Python<'_>,
        start_temp: f64,
        end_temp: f64,
        steps: usize,
        max_iterations: usize,
        layer_index: usize,
      ) -> PyResult<usize> {
        let inner = Arc::clone(&self.inner);
        run_in_thread(py, move || {
          let mut guard = inner.lock().unwrap();
          guard.cooldown(start_temp, end_temp, steps, max_iterations, layer_index)
        })
      }

      pub fn optimize(
        &mut self,
        py: Python<'_>,
        start_temp: f64,
        end_temp: f64,
        steps: usize,
        max_iterations: usize,
        passes: usize,
      ) -> PyResult<usize> {
        let inner = Arc::clone(&self.inner);
        run_in_thread(py, move || {
          let mut guard = inner.lock().unwrap();
          guard.optimize(start_temp, end_temp, steps, max_iterations, passes)
        })
      }

      pub fn get_nodes(&self) -> Vec<Vec<$ty>> {
        // cheap read; no thread needed
        self.inner.lock().unwrap().get_nodes()
      }

      pub fn count_crossings(&self) -> usize {
        self.inner.lock().unwrap().count_crossings()
      }
    }

    #[pyclass]
    struct $name2 {
      inner: Arc<Mutex<core::hierarchy_optimizer::HierarchyOptimizer<$ty>>>,
    }

    #[pymethods]
    impl $name2 {
      #[new]
      pub fn layout_optimizer_new(
        nodes_left: Vec<Vec<$ty>>,
        edges: Vec<Vec<($ty, $ty, usize)>>,
        hierarchy: Hierarchy,
      ) -> PyResult<Self> {
        let inner =
          core::hierarchy_optimizer::HierarchyOptimizer::<$ty>::new(nodes_left, edges, hierarchy).map_err(to_pyerr)?;
        Ok(Self {
          inner: Arc::new(Mutex::new(inner)),
        })
      }

      #[pyo3(signature = (temperature, max_iterations, layer_index, granularity))]
      pub fn swap_nodes(
        &mut self,
        py: Python<'_>,
        temperature: f64,
        max_iterations: usize,
        layer_index: usize,
        granularity: Option<usize>,
      ) -> PyResult<usize> {
        let inner = Arc::clone(&self.inner);
        run_in_thread(py, move || {
          let mut guard = inner.lock().unwrap();
          guard.swap_nodes(temperature, max_iterations, layer_index, granularity)
        })
      }

      #[allow(clippy::too_many_arguments)]
      #[pyo3(signature = (start_temp, end_temp, steps, max_iterations, layer_index, granularity))]
      pub fn cooldown(
        &mut self,
        py: Python<'_>,
        start_temp: f64,
        end_temp: f64,
        steps: usize,
        max_iterations: usize,
        layer_index: usize,
        granularity: Option<usize>,
      ) -> PyResult<usize> {
        let inner = Arc::clone(&self.inner);
        run_in_thread(py, move || {
          let mut guard = inner.lock().unwrap();
          guard.cooldown(start_temp, end_temp, steps, max_iterations, layer_index, granularity)
        })
      }

      pub fn optimize(
        &mut self,
        py: Python<'_>,
        start_temp: f64,
        end_temp: f64,
        steps: usize,
        max_iterations: usize,
        passes: usize,
      ) -> PyResult<usize> {
        let inner = Arc::clone(&self.inner);
        run_in_thread(py, move || {
          let mut guard = inner.lock().unwrap();
          guard.optimize(start_temp, end_temp, steps, max_iterations, passes)
        })
      }

      pub fn get_nodes(&self) -> Vec<Vec<$ty>> {
        self.inner.lock().unwrap().get_nodes()
      }

      pub fn count_crossings(&self) -> usize {
        self.inner.lock().unwrap().count_crossings()
      }
    }
  };
}

optimizers!(String, LayoutOptimizerString, HierarchyOptimizerString);
optimizers!(i32, LayoutOptimizerInt, HierarchyOptimizerInt);

#[pyfunction]
fn generate_multipartite_graph(py: Python<'_>, n_nodes: Vec<usize>) -> PyResult<utils::GraphType> {
  run_in_thread(py, move || utils::generate_multipartite_graph(n_nodes))
}

#[pymodule]
mod untanglers {
  #[pymodule_export]
  use crate::LayoutOptimizerString;

  #[pymodule_export]
  use crate::HierarchyOptimizerString;

  #[pymodule_export]
  use crate::LayoutOptimizerInt;

  #[pymodule_export]
  use crate::HierarchyOptimizerInt;

  #[pymodule_export]
  use crate::generate_multipartite_graph;
}
