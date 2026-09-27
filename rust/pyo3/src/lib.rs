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
use std::sync::OnceLock;

use magika::{ContentType, Features, FeaturesOrRuled, FileType, OverwriteReason, Runtime, Session, MODEL_NAME};
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
            // FIXME(https://github.com/google/magika/issues/1480): Use ContentType::Undefined
            // from magika-lib once exposed on magika::ContentType.
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

enum PathDisposition {
    Immediate(PyMagikaResult),
    Features(Features),
}

// FIXME(https://github.com/google/magika/issues/1481): Remove custom no_dereference
// metadata lookup once magika-lib's Session::identify_file / FeaturesOrRuled::extract_file
// supports `no_dereference`.
// FIXME(https://github.com/google/magika/issues/1482): Remove custom `!metadata.is_file()`
// check once magika-lib handles non-regular / special files (e.g., /dev/null, FIFOs)
// as FileType::Ruled(ContentType::Unknown).
fn extract_path_disposition(path_str: &str, no_dereference: bool) -> anyhow::Result<PathDisposition> {
    let p = Path::new(path_str);
    let metadata_res = if no_dereference {
        std::fs::symlink_metadata(p)
    } else {
        std::fs::metadata(p)
    };
    let metadata = match metadata_res {
        Ok(m) => m,
        Err(io_err) => {
            return Ok(PathDisposition::Immediate(from_io_error(
                &io_err,
                Some(path_str.to_string()),
            )));
        }
    };

    if no_dereference && metadata.is_symlink() {
        return Ok(PathDisposition::Immediate(from_file_type(
            &FileType::Symlink,
            Some(path_str.to_string()),
        )));
    }
    if metadata.is_dir() {
        return Ok(PathDisposition::Immediate(from_file_type(
            &FileType::Directory,
            Some(path_str.to_string()),
        )));
    }
    if !metadata.is_file() {
        return Ok(PathDisposition::Immediate(from_file_type(
            &FileType::Ruled(ContentType::Unknown),
            Some(path_str.to_string()),
        )));
    }

    let file = match std::fs::File::open(p) {
        Ok(f) => f,
        Err(io_err) => {
            return Ok(PathDisposition::Immediate(from_io_error(
                &io_err,
                Some(path_str.to_string()),
            )));
        }
    };

    match FeaturesOrRuled::extract(file) {
        Ok(FeaturesOrRuled::Ruled(ct)) => Ok(PathDisposition::Immediate(from_file_type(
            &FileType::Ruled(ct),
            Some(path_str.to_string()),
        ))),
        Ok(FeaturesOrRuled::Features(features)) => Ok(PathDisposition::Features(features)),
        Err(e) => {
            if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                Ok(PathDisposition::Immediate(from_io_error(
                    io_err,
                    Some(path_str.to_string()),
                )))
            } else {
                Err(e)
            }
        }
    }
}

static RUNTIME: OnceLock<Result<Runtime, String>> = OnceLock::new();

fn get_shared_runtime() -> anyhow::Result<&'static Runtime> {
    let res = RUNTIME.get_or_init(|| Runtime::new().map_err(|e| e.to_string()));
    match res {
        Ok(rt) => Ok(rt),
        Err(msg) => anyhow::bail!("{msg}"),
    }
}

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
}

#[pyclass(name = "Magika")]
pub struct PyMagika {
    no_dereference: bool,
}

impl PyMagika {
    fn with_session<R>(&self, f: impl FnOnce(&mut Session) -> anyhow::Result<R>) -> anyhow::Result<R> {
        let runtime = get_shared_runtime()?;
        SESSION.with(|cell| {
            let mut slot = cell.borrow_mut();
            if slot.is_none() {
                *slot = Some(runtime.session()?);
            }
            f(slot.as_mut().unwrap())
        })
    }
}

#[pymethods]
impl PyMagika {
    #[new]
    #[pyo3(signature = (no_dereference=false))]
    fn new(no_dereference: bool) -> PyResult<Self> {
        get_shared_runtime()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to initialize Magika runtime: {e}")))?;
        Ok(Self { no_dereference })
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
        let no_dereference = self.no_dereference;
        let result: anyhow::Result<PyMagikaResult> = py.allow_threads(|| {
            match extract_path_disposition(path, no_dereference)? {
                PathDisposition::Immediate(res) => Ok(res),
                PathDisposition::Features(features) => {
                    let file_type = self.with_session(|session| session.identify_features(&features))?;
                    Ok(from_file_type(&file_type, Some(path.to_string())))
                }
            }
        });
        match result {
            Ok(res) => Ok(res),
            Err(e) => Err(PyRuntimeError::new_err(format!("Inference error: {e}"))),
        }
    }

    fn identify_paths(&self, py: Python<'_>, paths: Vec<String>) -> PyResult<Vec<PyMagikaResult>> {
        let no_dereference = self.no_dereference;
        let results: anyhow::Result<Vec<PyMagikaResult>> = py.allow_threads(|| {
            let mut slots: Vec<Option<PyMagikaResult>> = Vec::with_capacity(paths.len());
            let mut batch_indices: Vec<usize> = Vec::new();
            let mut batch_features: Vec<Features> = Vec::new();

            for (idx, path_str) in paths.iter().enumerate() {
                match extract_path_disposition(path_str, no_dereference) {
                    Ok(PathDisposition::Immediate(res)) => {
                        slots.push(Some(res));
                    }
                    Ok(PathDisposition::Features(features)) => {
                        slots.push(None);
                        batch_indices.push(idx);
                        batch_features.push(features);
                    }
                    Err(e) => {
                        slots.push(Some(PyMagikaResult {
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
                        }));
                    }
                }
            }

            if !batch_features.is_empty() {
                let inferred_types = self.with_session(|session| {
                    session.identify_features_batch(&batch_features)
                })?;
                for (slot_idx, file_type) in batch_indices.into_iter().zip(inferred_types) {
                    slots[slot_idx] = Some(from_file_type(
                        &file_type,
                        Some(paths[slot_idx].clone()),
                    ));
                }
            }

            Ok(slots.into_iter().map(|s| s.unwrap()).collect())
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
