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

//! Resident fixed-shape tract plan ownership and request routing.

use std::path::Path;

use anyhow::{Context as _, Result, ensure};

use crate::{Backend, FEATURE_SIZE, PADDING_TOKEN, PoolRouting, TractBackend};

const DIRECT_FUSED_POOL_MIN_BATCH: usize = 8;

pub(super) struct PlanPoolBackend {
    name: &'static str,
    plans: Vec<(usize, TractBackend)>,
    routing: PoolRouting,
}

impl PlanPoolBackend {
    pub(super) fn load_cpu(
        classes: &[usize], threads: usize, nnef_model: Option<&Path>, direct_fused: bool,
        routing: PoolRouting,
    ) -> Result<Self> {
        let mut plans = Vec::with_capacity(classes.len());
        for &class in classes {
            plans.push((
                class,
                TractBackend::load_cpu(
                    threads,
                    Some(class),
                    nnef_model,
                    direct_fused && class >= DIRECT_FUSED_POOL_MIN_BATCH,
                )?,
            ));
        }
        let mut pool = Self { name: "tract-cpu-pool", plans, routing };
        pool.warm_all()?;
        Ok(pool)
    }

    #[cfg(all(feature = "metal", target_os = "macos"))]
    pub(super) fn load_metal(
        classes: &[usize], gemm: Option<&str>, nnef_model: Option<&Path>, routing: PoolRouting,
    ) -> Result<Self> {
        let mut plans = Vec::with_capacity(classes.len());
        for &class in classes {
            plans.push((class, TractBackend::load_metal(Some(class), gemm, nnef_model)?));
        }
        let mut pool = Self { name: "tract-metal-pool", plans, routing };
        pool.warm_all()?;
        Ok(pool)
    }

    fn plan_for_class(&mut self, class: usize) -> Result<&mut TractBackend> {
        self.plans
            .iter_mut()
            .find(|(candidate, _)| *candidate == class)
            .map(|(_, backend)| backend)
            .with_context(|| format!("fixed class {class} is not resident"))
    }

    fn route(&self, batch: usize) -> Result<Vec<(usize, usize)>> {
        let classes = self.plans.iter().map(|(class, _)| *class).collect::<Vec<_>>();
        route_classes(&classes, self.routing, batch)
    }

    fn warm_all(&mut self) -> Result<()> {
        for (class, backend) in &mut self.plans {
            let input = vec![PADDING_TOKEN; *class * FEATURE_SIZE];
            backend.run(&input, *class).with_context(|| format!("warming batch-{class} plan"))?;
        }
        Ok(())
    }
}

fn route_classes(
    classes: &[usize], routing: PoolRouting, batch: usize,
) -> Result<Vec<(usize, usize)>> {
    if matches!(routing, PoolRouting::Ceil) {
        let class = classes
            .iter()
            .copied()
            .find(|class| *class >= batch)
            .with_context(|| format!("no resident fixed plan can fit batch {batch}"))?;
        return Ok(vec![(class, batch)]);
    }

    let mut remaining = batch;
    let mut route = Vec::new();
    while remaining > 0 {
        if let Some(class) = classes.iter().rev().copied().find(|class| *class <= remaining) {
            route.push((class, class));
            remaining -= class;
        } else {
            let class = classes
                .iter()
                .copied()
                .find(|class| *class >= remaining)
                .with_context(|| format!("no resident fixed plan can fit tail {remaining}"))?;
            route.push((class, remaining));
            remaining = 0;
        }
    }
    Ok(route)
}

impl Backend for PlanPoolBackend {
    fn name(&self) -> &'static str {
        self.name
    }

    fn run(&mut self, input: &[i32], batch: usize) -> Result<Vec<f32>> {
        ensure!(input.len() == batch * FEATURE_SIZE);
        let route = self.route(batch)?;
        let mut input_offset = 0;
        let mut outputs = Vec::new();
        for (class, items) in route {
            let input_len = items * FEATURE_SIZE;
            let chunk = &input[input_offset..input_offset + input_len];
            input_offset += input_len;
            let mut padded = Vec::new();
            let chunk = if class == items {
                chunk
            } else {
                padded.reserve(class * FEATURE_SIZE);
                padded.extend_from_slice(chunk);
                padded.resize(class * FEATURE_SIZE, PADDING_TOKEN);
                &padded
            };
            let backend = self.plan_for_class(class)?;
            let mut output = backend.run(chunk, class)?;
            ensure!(output.len().is_multiple_of(class));
            let scores_per_item = output.len() / class;
            output.truncate(items * scores_per_item);
            outputs.extend(output);
        }
        ensure!(input_offset == input.len());
        Ok(outputs)
    }

    fn selected_classes(&self, batch: usize) -> Option<Vec<usize>> {
        self.route(batch).ok().map(|route| route.into_iter().map(|(class, _)| class).collect())
    }

    fn plan_op_counts(&self) -> Option<std::collections::BTreeMap<String, usize>> {
        let mut counts = std::collections::BTreeMap::new();
        for (class, backend) in &self.plans {
            for (op, count) in backend.plan_op_counts()? {
                counts.insert(format!("batch-{class}/{op}"), count);
            }
        }
        Some(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLASSES: &[usize] = &[1, 8, 16, 32, 64];

    #[test]
    fn exact_routing_composes_resident_classes_without_padding() {
        assert_eq!(
            route_classes(CLASSES, PoolRouting::Exact, 10).unwrap(),
            [(8, 8), (1, 1), (1, 1)]
        );
    }

    #[test]
    fn ceiling_routing_uses_one_padded_class() {
        assert_eq!(route_classes(CLASSES, PoolRouting::Ceil, 10).unwrap(), [(16, 10)]);
    }

    #[test]
    fn exact_routing_pads_only_when_no_smaller_class_exists() {
        assert_eq!(route_classes(&[8, 16], PoolRouting::Exact, 3).unwrap(), [(8, 3)]);
    }
}
