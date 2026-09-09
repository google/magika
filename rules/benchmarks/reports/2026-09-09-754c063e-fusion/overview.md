# CPU artifact comparison

Dataset: **Adjudicated combined corpus**, version `snapshot-1411a5c0fd4a`. Measured at **2026-09-09T01:36:40.400275Z**; benchmark protocol `1.1.0`.

<details><summary>Exact tool configurations and build identities</summary>

- **Magika 2.0.0-dev @ 754c063e** — config `6b452bc0`, build `fd60f5c4a75782d6c4bc0222df558c7fb7a0a71cc847719bc511a0256155c2ca`: `{"backend": "CPU", "internal_batch": 8, "model": "standard_v3_3", "model_graph": "unfused-small-batches", "readers": 2, "rules": "off", "source_dirty": true, "threads": 2}`
- **Magika 2.0.0-dev @ 754c063e** — config `200453ba`, build `4b07179c5044ec38d663e01a40a91bca57b706b66ec9d196c27c447a7ced57e2`: `{"backend": "CPU", "internal_batch": 8, "model": "standard_v3_3", "model_graph": "fused-all-batches", "readers": 2, "rules": "off", "source_dirty": true, "threads": 2}`
- **Magika 2.0.0-dev @ 754c063e** — config `393d21e1`, build `8503b83da6a936257476e436b1e264f55538c89451ee52e9e35c8e7ec2254a25`: `{"backend": "CPU", "internal_batch": 8, "model": null, "model_graph": "fused-all-batches", "readers": 2, "rules": "only", "source_dirty": true, "threads": 0}`

Full commands, environment, source revisions and executable/artifact hashes are retained in `overview.json`.

</details>

25,421 identical normalized CPU decisions; zero tool errors. 5 measured runs after 1 warmup. Whole-process median milliseconds, including startup and shutdown; warm OS caches.

The two ML rows use the same CLI with different CPU runtime artifacts. The measured source tree was dirty; exact artifact hashes and the unfused/fused model configuration are retained above and in JSON. The rules-only row uses the unchanged native reference library.

| Tool | Version | Config | Accuracy | Precision | Coverage | Rule matches | 1 files | 2 files | 5 files | 10 files | 100 files | 1,000 files |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Magika | 2.0.0-dev @ 754c063e + dirty | CPU; ML (`6b452bc0`) | 57.73% | 70.23% | 82.20% | — | 6.60 | 7.82 | 12.54 | 16.28 | 71.62 | 605.74 |
| Magika | 2.0.0-dev @ 754c063e + dirty | CPU; ML (`200453ba`) | 57.73% | 70.23% | 82.20% | — | 6.43 | 6.55 | 13.46 | 17.17 | 71.88 | 625.53 |
| Magika | 2.0.0-dev @ 754c063e + dirty | CPU; rules-only (`393d21e1`) | 37.86% | 100.00% | 37.86% | 37.86% | 3.49 | 3.30 | 3.94 | 4.51 | 8.23 | 39.41 |

The initial timing run contained outliers. [Separate confirmation measurements and handoff dispositions](../2026-09-09-handoff-validation/dispositions.md) retain that limitation; no initial measurements were overwritten.
