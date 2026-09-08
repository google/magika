// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Verifies an arbitrary converted NNEF model against its ONNX source.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result, bail, ensure};
use ndarray::Array2;
use tract_core::prelude::{
    Framework as _, IntoTValue as _, IntoTensor as _, TValue, TVec, Tensor, ToDim as _, tvec,
};
use tract_core::runtime::{DefaultRuntime, Runtime as _};

const MAX_SCORE_DIFFERENCE: f32 = 1e-4;
const USAGE: &str = "usage: verify-model --onnx-model PATH --nnef-model PATH \
    --feature-size N --batch N";

#[derive(Debug)]
struct Options {
    onnx_model: PathBuf,
    nnef_model: PathBuf,
    feature_size: usize,
    batch: usize,
}

fn main() -> Result<()> {
    let options = parse_options(std::env::args_os().skip(1))?;
    ensure!(options.batch > 0, "--batch must be greater than zero");
    ensure!(options.feature_size > 0, "--feature-size must be greater than zero");
    let input_len = options
        .batch
        .checked_mul(options.feature_size)
        .context("--batch times --feature-size is too large")?;
    let input = (0..input_len).map(|index| (index % 257) as i32).collect::<Vec<_>>();

    ort::init().with_telemetry(false).commit();
    let onnx = run_onnx(&options, &input)?;
    let nnef = run_nnef(&options, &input)?;
    ensure!(onnx.len() == nnef.len(), "ONNX and NNEF output lengths differ");
    ensure!(!onnx.is_empty(), "model output is empty");
    ensure!(onnx.iter().all(|score| score.is_finite()), "ONNX output contains a non-finite score");
    ensure!(nnef.iter().all(|score| score.is_finite()), "NNEF output contains a non-finite score");
    ensure!(onnx.len().is_multiple_of(options.batch), "model output is not batched");
    let labels = onnx.len() / options.batch;
    let max_difference =
        onnx.iter().zip(&nnef).map(|(onnx, nnef)| (onnx - nnef).abs()).fold(0.0_f32, f32::max);
    ensure!(
        max_difference <= MAX_SCORE_DIFFERENCE,
        "maximum score difference {max_difference} exceeds {MAX_SCORE_DIFFERENCE}"
    );
    for item in 0..options.batch {
        let start = item * labels;
        let end = start + labels;
        ensure!(
            argmax(&onnx[start..end]) == argmax(&nnef[start..end]),
            "winning label differs for batch item {item}"
        );
    }
    println!(
        "verified\tbatch={}\tfeature_size={}\tmax_score_difference={max_difference}",
        options.batch, options.feature_size
    );
    Ok(())
}

fn run_onnx(options: &Options, input: &[i32]) -> Result<Vec<f32>> {
    let mut builder = ort::session::Session::builder()?
        .with_intra_threads(1)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?
        .with_inter_threads(1)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let mut session = builder
        .commit_from_file(&options.onnx_model)
        .with_context(|| format!("loading ONNX model {}", options.onnx_model.display()))?;
    let input = Array2::from_shape_vec([options.batch, options.feature_size], input.to_vec())?;
    let input = ort::value::Tensor::from_array(input)?;
    let mut outputs = session.run(ort::inputs!("bytes" => input))?;
    let output = outputs.remove("target_label").context("ONNX output target_label is missing")?;
    let output = output.try_extract_array::<f32>()?;
    ensure!(output.ndim() == 2 && output.shape()[0] == options.batch, "invalid ONNX output shape");
    Ok(output.iter().copied().collect())
}

fn run_nnef(options: &Options, input: &[i32]) -> Result<Vec<f32>> {
    let model = tract_nnef::nnef()
        .model_for_path(&options.nnef_model)
        .with_context(|| format!("loading NNEF model {}", options.nnef_model.display()))?;
    let model = if let Some(symbol) = model.symbols.get("N") {
        model
            .set_symbols(&HashMap::from([(symbol, options.batch.to_dim())]))
            .context("binding the NNEF batch symbol")?
    } else {
        ensure!(
            model.input_fact(0)?.shape[0] == options.batch.to_dim(),
            "fixed NNEF batch does not match --batch"
        );
        model
    };
    let model = model.into_optimized().context("optimizing the NNEF model")?;
    let runnable = DefaultRuntime.prepare(model).context("preparing the NNEF model")?;
    let mut state = runnable.spawn().context("spawning the NNEF model")?;
    let input = Tensor::from_shape(&[options.batch, options.feature_size], input)?.into_tvalue();
    let mut outputs: TVec<TValue> = state.run(tvec!(input))?;
    ensure!(outputs.len() == 1, "NNEF model returned more than one output");
    let output = outputs.remove(0).into_tensor();
    ensure!(output.rank() == 2 && output.shape()[0] == options.batch, "invalid NNEF output shape");
    Ok(output.to_plain_array_view::<f32>()?.iter().copied().collect())
}

fn argmax(scores: &[f32]) -> usize {
    scores
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
        .unwrap()
}

fn parse_options(args: impl IntoIterator<Item = impl Into<std::ffi::OsString>>) -> Result<Options> {
    let mut onnx_model = None;
    let mut nnef_model = None;
    let mut feature_size = None;
    let mut batch = None;
    let mut args = args.into_iter().map(|arg| arg.into());
    while let Some(arg) = args.next() {
        let arg = arg.to_string_lossy();
        let mut value = || {
            args.next()
                .with_context(|| format!("missing value after {arg}"))
                .map(|value| value.to_string_lossy().into_owned())
        };
        match arg.as_ref() {
            "--onnx-model" => onnx_model = Some(PathBuf::from(value()?)),
            "--nnef-model" => nnef_model = Some(PathBuf::from(value()?)),
            "--feature-size" => {
                feature_size = Some(value()?.parse().context("invalid --feature-size")?)
            }
            "--batch" => batch = Some(value()?.parse().context("invalid --batch")?),
            "-h" | "--help" => bail!("{USAGE}"),
            option => bail!("unknown option {option}\n{USAGE}"),
        }
    }
    Ok(Options {
        onnx_model: onnx_model.context("--onnx-model is required")?,
        nnef_model: nnef_model.context("--nnef-model is required")?,
        feature_size: feature_size.context("--feature-size is required")?,
        batch: batch.context("--batch is required")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_required_model_argument() {
        let options = parse_options([
            "--onnx-model",
            "model.onnx",
            "--nnef-model",
            "model.nnef.tgz",
            "--feature-size",
            "2048",
            "--batch",
            "8",
        ])
        .unwrap();
        assert_eq!(options.feature_size, 2048);
        assert_eq!(options.batch, 8);
    }

    #[test]
    fn requires_both_models() {
        assert!(parse_options(["--feature-size", "2048", "--batch", "8"]).is_err());
    }
}
