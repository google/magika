// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use magika_tract_runtime::{BackendRequest, Runtime as RawRuntime, BATCH_CLASSES, NUM_LABELS};
use ndarray::ArrayView2;

use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    prediction_mode: ReferencePredictionMode,
    path: Option<String>,
    content_base64: Option<String>,
    status: String,
    prediction: Option<Prediction>,
}

struct Sample {
    name: String,
    features: Features,
    expected: Prediction,
    cpu_output: &'static str,
}

#[test]
#[ignore = "requires an available GPU; run explicitly for GPU release qualification"]
fn reference_decisions_match_on_every_gpu_batch() {
    let mut cpu = RawRuntime::with_max_batch(BackendRequest::Cpu, 1).unwrap().session().unwrap();
    let mut samples = Vec::new();
    for kind in ["path", "content"] {
        let path =
            format!("../../tests_data/reference/{MODEL_NAME}-inference_examples_by_{kind}.json.gz");
        let mut json = String::new();
        GzDecoder::new(File::open(path).unwrap()).read_to_string(&mut json).unwrap();
        let fixtures: Vec<Fixture> = serde_json::from_str(&json).unwrap();
        let mut count = 0;
        for fixture in fixtures {
            if fixture.prediction_mode != ReferencePredictionMode::HighConfidence {
                continue;
            }
            count += 1;
            assert_eq!(fixture.status, "ok");
            let (name, bytes) = match (fixture.path, fixture.content_base64) {
                (Some(path), None) => {
                    let bytes = std::fs::read(format!("../../{path}")).unwrap();
                    (path, bytes)
                }
                (None, Some(content)) => {
                    let bytes = BASE64.decode(content.as_bytes()).unwrap();
                    (content, bytes)
                }
                _ => panic!("fixture must identify exactly one input"),
            };
            let expected = fixture.prediction.unwrap();
            match FeaturesOrRuled::extract(bytes.as_slice()).unwrap() {
                FeaturesOrRuled::Ruled(kind) => {
                    assert_prediction(FileType::Ruled(kind), expected, &name);
                }
                FeaturesOrRuled::Features(features) => {
                    let reference = cpu.run(&features.0, 1).unwrap();
                    let reference = FileType::convert(
                        ArrayView2::from_shape((1, NUM_LABELS), &reference).unwrap().into_dyn(),
                    );
                    let cpu_output = reference[0].info().label;
                    assert_eq!(cpu_output, expected.output, "{name}");
                    samples.push(Sample { name, features, expected, cpu_output });
                }
            }
        }
        assert!(count >= 40, "too few {kind} reference cases: {count}");
    }
    assert!(samples.len() >= 64, "must cover many distinct real inputs");
    let distinct: std::collections::HashSet<_> = samples.iter().map(|s| &s.features.0).collect();
    assert!(distinct.len() >= 32, "must distinguish reordered rows");

    let runtime = RawRuntime::new(BackendRequest::Gpu).unwrap();
    assert_eq!(runtime.backend_info().backend(), magika_tract_runtime::Backend::Gpu);
    let mut gpu = runtime.session().unwrap();
    // Backend scores may differ. The contract here is the final classification
    // and overwrite decision, with valid model outputs on every resident class.
    for batch in BATCH_CLASSES {
        for start in (0..samples.len()).step_by(batch) {
            // Rotate the final batch through the corpus so it also executes the full class.
            let rows: Vec<_> = (0..batch).map(|i| &samples[(start + i) % samples.len()]).collect();
            let input: Vec<_> = rows.iter().flat_map(|s| s.features.0.iter().copied()).collect();
            let scores = gpu.run(&input, batch).unwrap();
            assert_eq!(scores.len(), batch * NUM_LABELS);
            let predictions = FileType::convert(
                ArrayView2::from_shape((batch, NUM_LABELS), &scores).unwrap().into_dyn(),
            );
            for ((sample, actual), row) in
                rows.iter().zip(predictions).zip(scores.chunks_exact(NUM_LABELS))
            {
                let context = format!("batch {batch}, {}", sample.name);
                assert!(
                    row.iter().all(|score| score.is_finite() && (0.0..=1.0).contains(score)),
                    "{context}: invalid probability vector"
                );
                assert!(
                    (row.iter().sum::<f32>() - 1.0).abs() <= 1e-4,
                    "{context}: probabilities do not sum to one"
                );
                assert_eq!(actual.info().label, sample.cpu_output, "{context}");
                let FileType::Inferred(actual) = actual else {
                    panic!("expected inference: {context}")
                };
                assert_eq!(actual.content_type().info().label, sample.expected.output, "{context}");
                let reason = match actual.content_type {
                    None => ReferenceOverwriteReason::None,
                    Some((_, OverwriteReason::LowConfidence)) => {
                        ReferenceOverwriteReason::LowConfidence
                    }
                    Some((_, OverwriteReason::OverwriteMap)) => {
                        ReferenceOverwriteReason::OverwriteMap
                    }
                };
                assert_eq!(reason, sample.expected.overwrite_reason, "{context}");
            }
        }
        eprintln!("GPU batch {batch}: {} ML fixtures, exact final decisions", samples.len());
    }
}
