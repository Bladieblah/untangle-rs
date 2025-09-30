use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::time::Duration;

/// Run a long-running Rust function in a background thread,
/// while keeping Python responsive to signals (Ctrl+C, etc.).
pub fn run_in_thread<F, T, E>(py: Python<'_>, f: F) -> PyResult<T>
where
  F: Send + 'static + FnOnce() -> Result<T, E>,
  T: Send + 'static,
  E: Send + 'static + std::fmt::Display,
{
  let handle = std::thread::spawn(f);

  loop {
    if handle.is_finished() {
      break;
    }

    // reacquire GIL just for signals
    Python::with_gil(|py| py.check_signals())?;

    // release GIL while we wait
    py.allow_threads(|| std::thread::sleep(Duration::from_millis(200)));
  }

  handle
    .join()
    .map_err(|_| PyRuntimeError::new_err("Worker thread panicked"))?
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}
