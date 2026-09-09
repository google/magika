// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;

use magika::{FileType, OverwriteReason, Runtime, Session, MODEL_NAME};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

#[pyclass(name = "MagikaResult")]
#[derive(Clone)]
pub struct PyMagikaResult {
    #[pyo3(get)]
    pub path: Option<String>,
    #[pyo3(get)]
    pub status: String,
    #[pyo3(get)]
    pub ok: bool,
    #[pyo3(get)]
    pub label: String,
    #[pyo3(get)]
    pub mime_type: String,
    #[pyo3(get)]
    pub group: String,
    #[pyo3(get)]
    pub description: String,
    #[pyo3(get)]
    pub extensions: Vec<String>,
    #[pyo3(get)]
    pub is_text: bool,
    #[pyo3(get)]
    pub score: f32,
    #[pyo3(get)]
    pub dl_label: String,
    #[pyo3(get)]
    pub overwrite_reason: String,
}

#[pymethods]
impl PyMagikaResult {
    fn __repr__(&self) -> String {
        if self.ok {
            format!(
                "MagikaResult(path={:?}, status={:?}, label={:?}, mime_type={:?}, score={:.4})",
                self.path, self.status, self.label, self.mime_type, self.score
            )
        } else {
            format!(
                "MagikaResult(path={:?}, status={:?})",
                self.path, self.status
            )
        }
    }
}

fn from_file_type(file_type: &FileType, path: Option<String>) -> PyMagikaResult {
    let info = file_type.info();
    let score = file_type.score();
    let (dl_label, overwrite_reason) = match file_type {
        FileType::Inferred(inferred) => {
            let dl = inferred.inferred_type.info().label.to_string();
            let reason = match inferred.content_type {
                None => "none",
                Some((_, OverwriteReason::LowConfidence)) => "low_confidence",
                Some((_, OverwriteReason::OverwriteMap)) => "overwrite_map",
            };
            (dl, reason.to_string())
        }
        FileType::Ruled(_) | FileType::Directory | FileType::Symlink => {
            ("undefined".to_string(), "none".to_string())
        }
    };

    PyMagikaResult {
        path,
        status: "ok".to_string(),
        ok: true,
        label: info.label.to_string(),
        mime_type: info.mime_type.to_string(),
        group: info.group.to_string(),
        description: info.description.to_string(),
        extensions: info.extensions.iter().map(|s| s.to_string()).collect(),
        is_text: info.is_text,
        score,
        dl_label,
        overwrite_reason,
    }
}

fn from_io_error(e: &std::io::Error, path: Option<String>) -> PyMagikaResult {
    let status = match e.kind() {
        std::io::ErrorKind::NotFound => "file_not_found_error",
        std::io::ErrorKind::PermissionDenied => "permission_error",
        _ => "unknown",
    };
    PyMagikaResult {
        path,
        status: status.to_string(),
        ok: false,
        label: String::new(),
        mime_type: String::new(),
        group: String::new(),
        description: String::new(),
        extensions: Vec::new(),
        is_text: false,
        score: 0.0,
        dl_label: String::new(),
        overwrite_reason: "none".to_string(),
    }
}

struct ThreadSession {
    runtime_ptr: usize,
    session: Session,
}

thread_local! {
    static SESSIONS: RefCell<Vec<ThreadSession>> = const { RefCell::new(Vec::new()) };
}

#[pyclass(name = "Magika")]
pub struct PyMagika {
    runtime: Arc<Runtime>,
}

impl PyMagika {
    fn with_session<R>(&self, f: impl FnOnce(&mut Session) -> anyhow::Result<R>) -> anyhow::Result<R> {
        let runtime_ptr = Arc::as_ptr(&self.runtime) as usize;
        SESSIONS.with(|sessions| {
            let mut sessions = sessions.borrow_mut();
            if let Some(entry) = sessions.iter_mut().find(|s| s.runtime_ptr == runtime_ptr) {
                f(&mut entry.session)
            } else {
                let session = self.runtime.session()?;
                sessions.push(ThreadSession { runtime_ptr, session });
                let entry = sessions.last_mut().unwrap();
                f(&mut entry.session)
            }
        })
    }
}

#[pymethods]
impl PyMagika {
    #[new]
    fn new() -> PyResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to initialize Magika runtime: {e}")))?;
        Ok(Self {
            runtime: Arc::new(runtime),
        })
    }

    fn identify_bytes(&self, py: Python<'_>, data: &[u8]) -> PyResult<PyMagikaResult> {
        let result = py.allow_threads(|| {
            self.with_session(|session| session.identify_content(data))
        });
        match result {
            Ok(file_type) => Ok(from_file_type(&file_type, None)),
            Err(e) => Err(PyRuntimeError::new_err(format!("Inference error: {e}"))),
        }
    }

    fn identify_path(&self, py: Python<'_>, path: &str) -> PyResult<PyMagikaResult> {
        let p = Path::new(path);
        let result: anyhow::Result<FileType> = py.allow_threads(|| {
            self.with_session(|session| session.identify_file(p))
        });
        match result {
            Ok(file_type) => Ok(from_file_type(&file_type, Some(path.to_string()))),
            Err(e) => {
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    Ok(from_io_error(io_err, Some(path.to_string())))
                } else {
                    Err(PyRuntimeError::new_err(format!("Inference error: {e}")))
                }
            }
        }
    }

    fn identify_paths(&self, py: Python<'_>, paths: Vec<String>) -> PyResult<Vec<PyMagikaResult>> {
        let results: anyhow::Result<Vec<PyMagikaResult>> = py.allow_threads(|| {
            self.with_session(|session| {
                let mut results = Vec::with_capacity(paths.len());
                for path_str in &paths {
                    let p = Path::new(path_str);
                    let res = match session.identify_file(p) {
                        Ok(file_type) => from_file_type(&file_type, Some(path_str.clone())),
                        Err(e) => {
                            if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                                from_io_error(io_err, Some(path_str.clone()))
                            } else {
                                PyMagikaResult {
                                    path: Some(path_str.clone()),
                                    status: "unknown".to_string(),
                                    ok: false,
                                    label: String::new(),
                                    mime_type: String::new(),
                                    group: String::new(),
                                    description: e.to_string(),
                                    extensions: Vec::new(),
                                    is_text: false,
                                    score: 0.0,
                                    dl_label: String::new(),
                                    overwrite_reason: "none".to_string(),
                                }
                            }
                        }
                    };
                    results.push(res);
                }
                Ok(results)
            })
        });
        match results {
            Ok(res) => Ok(res),
            Err(e) => Err(PyRuntimeError::new_err(format!("Inference error: {e}"))),
        }
    }

    #[staticmethod]
    fn get_default_model_name() -> &'static str {
        MODEL_NAME
    }

    fn get_model_name(&self) -> &'static str {
        MODEL_NAME
    }
}

#[pyfunction]
fn get_default_model_name() -> &'static str {
    MODEL_NAME
}

#[pymodule]
fn _magika(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMagika>()?;
    m.add_class::<PyMagikaResult>()?;
    m.add_function(wrap_pyfunction!(get_default_model_name, m)?)?;
    Ok(())
}
