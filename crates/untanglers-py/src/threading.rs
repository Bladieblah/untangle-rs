use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::thread;
use std::time::Duration;

/// Run a long-running Rust function in a background thread,
/// while keeping Python responsive to signals (Ctrl+C, etc.).
pub fn run_in_thread<F, T, E>(py: Python<'_>, f: F) -> PyResult<T>
where
  F: Send + 'static + FnOnce() -> Result<T, E>,
  T: Send + 'static,
  E: Send + 'static + std::fmt::Display,
{
  let handle = thread::spawn(f);

  loop {
    if handle.is_finished() {
      break;
    }
    py.check_signals()?; // raises KeyboardInterrupt if SIGINT
    thread::sleep(Duration::from_millis(100));
  }

  handle
    .join()
    .map_err(|_| PyRuntimeError::new_err("Worker thread panicked"))?
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}
