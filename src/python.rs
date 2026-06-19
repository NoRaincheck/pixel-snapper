use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use std::path::PathBuf;

use crate::{Config, PixelSnapperError};

#[pyclass]
#[derive(Clone)]
struct PixelSnapperConfig {
    #[pyo3(get, set)]
    k_colors: usize,
    #[pyo3(get, set)]
    pixel_size_override: Option<f64>,
}

#[pymethods]
impl PixelSnapperConfig {
    #[new]
    #[pyo3(signature = (k_colors=16, pixel_size_override=None))]
    fn new(k_colors: usize, pixel_size_override: Option<f64>) -> Self {
        Self {
            k_colors,
            pixel_size_override,
        }
    }

    fn __repr__(&self) -> String {
        match self.pixel_size_override {
            Some(px) => format!(
                "PixelSnapperConfig(k_colors={}, pixel_size_override={})",
                self.k_colors, px
            ),
            None => format!("PixelSnapperConfig(k_colors={})", self.k_colors),
        }
    }
}

impl From<&PixelSnapperConfig> for Config {
    fn from(py_config: &PixelSnapperConfig) -> Self {
        Config {
            k_colors: py_config.k_colors,
            pixel_size_override: py_config.pixel_size_override,
            ..Default::default()
        }
    }
}

#[pyfunction]
#[pyo3(signature = (input_bytes, config=None))]
fn process_image<'py>(
    py: Python<'py>,
    input_bytes: Bound<'py, PyAny>,
    config: Option<&PixelSnapperConfig>,
) -> PyResult<Bound<'py, PyBytes>> {
    let bytes: Vec<u8> = input_bytes.extract()?;
    let rust_config = config.map(Config::from);
    let result = crate::process_image_to_bytes(&bytes, rust_config).map_err(map_error)?;
    Ok(PyBytes::new_bound(py, &result))
}

#[pyfunction]
#[pyo3(signature = (input_path, output_path, config=None))]
fn process_file_cli(
    input_path: PathBuf,
    output_path: PathBuf,
    config: Option<&PixelSnapperConfig>,
) -> PyResult<()> {
    let rust_config = config.map(Config::from).unwrap_or_default();
    crate::process_file(&input_path, &output_path, &rust_config).map_err(map_error)?;
    Ok(())
}

#[pyfunction]
#[pyo3(signature = (input_dir, output_dir, config=None, callback=None))]
fn process_batch(
    py: Python<'_>,
    input_dir: PathBuf,
    output_dir: PathBuf,
    config: Option<&PixelSnapperConfig>,
    callback: Option<PyObject>,
) -> PyResult<()> {
    let batch_config = crate::BatchConfig {
        input_dir,
        output_dir,
        k_colors: config.map(|c| c.k_colors).unwrap_or(16),
        pixel_size_override: config.and_then(|c| c.pixel_size_override),
    };

    py.allow_threads(|| {
        crate::process_batch_with_reporter(&batch_config, |event| {
            if let Some(ref cb) = callback {
                let _ = Python::with_gil(|py| {
                    let dict = PyDict::new_bound(py);
                    match &event {
                        crate::BatchEvent::BatchStarted { input_dir, total } => {
                            dict.set_item("type", "batch_started").unwrap();
                            dict.set_item("input_dir", input_dir.to_str().unwrap_or("")).unwrap();
                            dict.set_item("total", total).unwrap();
                        }
                        crate::BatchEvent::Started { input, index, total } => {
                            dict.set_item("type", "started").unwrap();
                            dict.set_item("input", input.to_str().unwrap_or("")).unwrap();
                            dict.set_item("index", index).unwrap();
                            dict.set_item("total", total).unwrap();
                        }
                        crate::BatchEvent::Finished { input, output, index, total } => {
                            dict.set_item("type", "finished").unwrap();
                            dict.set_item("input", input.to_str().unwrap_or("")).unwrap();
                            dict.set_item("output", output.to_str().unwrap_or("")).unwrap();
                            dict.set_item("index", index).unwrap();
                            dict.set_item("total", total).unwrap();
                        }
                        crate::BatchEvent::Failed { input, output, error, index, total } => {
                            dict.set_item("type", "failed").unwrap();
                            dict.set_item("input", input.to_str().unwrap_or("")).unwrap();
                            dict.set_item("output", output.to_str().unwrap_or("")).unwrap();
                            dict.set_item("error", error).unwrap();
                            dict.set_item("index", index).unwrap();
                            dict.set_item("total", total).unwrap();
                        }
                        crate::BatchEvent::BatchFinished { input_dir, total } => {
                            dict.set_item("type", "batch_finished").unwrap();
                            dict.set_item("input_dir", input_dir.to_str().unwrap_or("")).unwrap();
                            dict.set_item("total", total).unwrap();
                        }
                    }
                    let _ = cb.call1(py, (dict,));
                    Ok::<(), PyErr>(())
                });
            }
        })
    })
    .map_err(map_error)
}

fn map_error(e: PixelSnapperError) -> PyErr {
    match e {
        PixelSnapperError::ImageError(e) => PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()),
        PixelSnapperError::InvalidInput(msg) => PyErr::new::<pyo3::exceptions::PyValueError, _>(msg),
        PixelSnapperError::ProcessingError(msg) => PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(msg),
    }
}

#[pymodule]
fn pixel_snapper(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PixelSnapperConfig>()?;
    m.add_function(wrap_pyfunction!(process_image, m)?)?;
    m.add_function(wrap_pyfunction!(process_file_cli, m)?)?;
    m.add_function(wrap_pyfunction!(process_batch, m)?)?;
    Ok(())
}
