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
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use magika::pipeline::{Engine, EngineSession, Pipeline, PipelineOptions};
use magika::{
    ContentType, FileType, Options, OverwriteReason, PredictionMode, Rules, Runtime, TypeInfo,
    MODEL_NAME,
};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;

#[pyclass(name = "ContentTypeInfo", frozen)]
pub struct PyContentTypeInfo {
    info: &'static TypeInfo,
}

#[pymethods]
impl PyContentTypeInfo {
    #[getter]
    fn label(&self) -> &'static str {
        self.info.label
    }

    #[getter]
    fn mime_type(&self) -> &'static str {
        self.info.mime_type
    }

    #[getter]
    fn group(&self) -> &'static str {
        self.info.group
    }

    #[getter]
    fn description(&self) -> &'static str {
        self.info.description
    }

    #[getter]
    fn extensions(&self) -> Vec<&'static str> {
        self.info.extensions.to_vec()
    }

    #[getter]
    fn is_text(&self) -> bool {
        self.info.is_text
    }
}

#[pyclass(name = "MagikaResult")]
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
            format!("MagikaResult(path={:?}, status={:?})", self.path, self.status)
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
                Some((_, OverwriteReason::RulesVeto)) => "rules_veto",
            };
            (dl, reason.to_string())
        }
        FileType::Ruled(_) | FileType::Directory | FileType::Symlink | FileType::Unsupported => {
            (ContentType::Undefined.info().label.to_string(), "none".to_string())
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

/// The engine shared by every `Magika` instance: preparing the model happens once per process.
static ENGINE: OnceLock<Result<Arc<Engine>, String>> = OnceLock::new();

fn get_shared_engine() -> anyhow::Result<&'static Arc<Engine>> {
    match ENGINE.get_or_init(|| {
        Engine::new(Runtime::builder().with_max_batch(PipelineOptions::default().batch_size))
            .map(Arc::new)
            .map_err(|e| e.to_string())
    }) {
        Ok(engine) => Ok(engine),
        Err(msg) => anyhow::bail!("{msg}"),
    }
}

thread_local! {
    static SESSION: RefCell<Option<EngineSession<'static>>> = const { RefCell::new(None) };
}

/// The result of an error identifying a path, as a result rather than an exception for I/O.
fn from_error(error: anyhow::Error, path: &str) -> PyMagikaResult {
    match error.downcast_ref::<std::io::Error>() {
        Some(io_err) => from_io_error(io_err, Some(path.to_string())),
        None => PyMagikaResult {
            path: Some(path.to_string()),
            status: "unknown".to_string(),
            ok: false,
            label: String::new(),
            mime_type: String::new(),
            group: String::new(),
            description: error.to_string(),
            extensions: Vec::new(),
            is_text: false,
            score: 0.0,
            dl_label: String::new(),
            overwrite_reason: "none".to_string(),
        },
    }
}

#[pyclass(name = "Magika")]
pub struct PyMagika {
    options: Options,
}

impl PyMagika {
    fn with_session<R>(
        &self, f: impl FnOnce(&mut EngineSession<'static>, &Options) -> anyhow::Result<R>,
    ) -> anyhow::Result<R> {
        let engine: &'static Engine = get_shared_engine()?;
        SESSION.with(|cell| {
            let mut slot = cell.borrow_mut();
            f(slot.get_or_insert_with(|| engine.session()), &self.options)
        })
    }
}

#[pymethods]
impl PyMagika {
    #[new]
    #[pyo3(signature = (use_rules=true, use_model=true, prediction_mode="high_confidence", follow_symlink=true, rules=None, rules_files=None))]
    fn new(
        py: Python<'_>, use_rules: bool, use_model: bool, prediction_mode: &str,
        follow_symlink: bool, rules: Option<&str>, rules_files: Option<Vec<std::path::PathBuf>>,
    ) -> PyResult<Self> {
        let custom_rules = match (rules, rules_files) {
            (None, None) => None,
            (Some(text), None) => Some(Rules::compile(text)),
            (None, Some(paths)) => Some(Rules::from_files(paths)),
            (Some(_), Some(_)) => {
                return Err(PyValueError::new_err("Pass either rules or rules_files, not both"));
            }
        };
        let custom_rules = custom_rules
            .transpose()
            .map_err(|e| PyValueError::new_err(format!("Invalid custom rules: {e:#}")))?;
        let prediction_mode = match prediction_mode {
            "high_confidence" => PredictionMode::HighConfidence,
            "medium_confidence" => PredictionMode::MediumConfidence,
            "best_guess" => PredictionMode::BestGuess,
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Invalid prediction mode: {prediction_mode:?}"
                )));
            }
        };
        // Waiting for the CPU runtime here reports a failure to prepare the model at once, and
        // lets a forked process start only once no thread is still preparing it.
        py.detach(|| get_shared_engine()?.backend_info()).map_err(|e| {
            PyRuntimeError::new_err(format!("Failed to initialize Magika runtime: {e:#}"))
        })?;
        let mut options = Options::default();
        options.use_rules = use_rules;
        options.use_model = use_model;
        options.prediction_mode = prediction_mode;
        options.follow_symlink = follow_symlink;
        options.custom_rules = custom_rules;
        Ok(Self { options })
    }

    fn identify_bytes(&self, py: Python<'_>, data: &[u8]) -> PyResult<PyMagikaResult> {
        let result = py.detach(|| {
            self.with_session(|session, options| session.identify_content(data, options))
        });
        match result {
            Ok(file_type) => Ok(from_file_type(&file_type, None)),
            Err(e) => Err(PyRuntimeError::new_err(format!("Inference error: {e}"))),
        }
    }

    fn identify_path(&self, py: Python<'_>, path: &str) -> PyResult<PyMagikaResult> {
        let result = py
            .detach(|| self.with_session(|session, options| session.identify_file(path, options)));
        match result {
            Ok(file_type) => Ok(from_file_type(&file_type, Some(path.to_string()))),
            Err(e) if e.downcast_ref::<std::io::Error>().is_some() => Ok(from_error(e, path)),
            Err(e) => Err(PyRuntimeError::new_err(format!("Inference error: {e}"))),
        }
    }

    fn identify_paths(&self, py: Python<'_>, paths: Vec<String>) -> PyResult<Vec<PyMagikaResult>> {
        let results: anyhow::Result<Vec<PyMagikaResult>> = py.detach(|| {
            let engine = get_shared_engine()?.clone();
            let pipeline = Pipeline::new(engine, self.options.clone(), PipelineOptions::default())?;
            let identified = pipeline.identify_paths(paths.iter().map(PathBuf::from).collect())?;
            // Without recursion, there is one result per path, in order.
            identified
                .zip(&paths)
                .map(|(item, path)| {
                    Ok(match item?.1 {
                        Ok(file_type) => from_file_type(&file_type, Some(path.clone())),
                        Err(error) => from_error(error, path),
                    })
                })
                .collect()
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

    fn get_output_content_types(&self) -> Vec<&'static str> {
        TypeInfo::possible_output().into_iter().map(|x| x.label).collect()
    }

    fn get_model_content_types(&self) -> Vec<&'static str> {
        TypeInfo::model_output().into_iter().map(|x| x.label).collect()
    }
}

#[pyfunction]
fn get_default_model_name() -> &'static str {
    MODEL_NAME
}

#[pyfunction]
fn content_type_from_label(label: &str) -> Option<PyContentTypeInfo> {
    ContentType::from_label(label).map(|x| PyContentTypeInfo { info: x.info() })
}

#[pyfunction]
fn get_output_content_types() -> Vec<&'static str> {
    TypeInfo::possible_output().into_iter().map(|x| x.label).collect()
}

#[pyfunction]
fn get_model_content_types() -> Vec<&'static str> {
    TypeInfo::model_output().into_iter().map(|x| x.label).collect()
}

#[pymodule]
fn _magika(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyMagika>()?;
    m.add_class::<PyMagikaResult>()?;
    m.add_class::<PyContentTypeInfo>()?;
    m.add_function(wrap_pyfunction!(get_default_model_name, m)?)?;
    m.add_function(wrap_pyfunction!(content_type_from_label, m)?)?;
    m.add_function(wrap_pyfunction!(get_output_content_types, m)?)?;
    m.add_function(wrap_pyfunction!(get_model_content_types, m)?)?;
    Ok(())
}
