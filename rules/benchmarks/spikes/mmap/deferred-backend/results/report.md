# Deferred tract backend spike

Identical saved file lists, model, rules, thread settings and dependency versions. Both distributions support CPU and Metal. Twenty fresh-process Hyperfine runs after one warmup; warm OS caches; direct execution without a shell. Timings include startup and shutdown. Natural subsets and the one-file rule miss are reported separately.

| Mode | Files / workload | Eager median ms | Deferred median ms | Eager / deferred |
|---|---:|---:|---:|---:|
| cpu-backend-ready | backend ready | 4.698 | 4.023 | 1.17× |
| cpu-ml | 1 (rule miss) | 6.205 | 5.472 | 1.13× |
| cpu-ml | 1 | 6.705 | 6.054 | 1.11× |
| cpu-ml | 2 | 12.178 | 11.669 | 1.04× |
| cpu-ml | 5 | 12.393 | 11.657 | 1.06× |
| cpu-ml | 10 | 13.526 | 13.555 | 1.00× |
| cpu-ml | 100 | 68.722 | 66.828 | 1.03× |
| cpu-ml | 1000 | 605.332 | 596.993 | 1.01× |
| cpu-rules-ml | 1 (rule miss) | 6.921 | 5.729 | 1.21× |
| cpu-rules-ml | 1 | 4.320 | 2.935 | 1.47× |
| cpu-rules-ml | 2 | 12.245 | 11.630 | 1.05× |
| cpu-rules-ml | 5 | 12.499 | 11.679 | 1.07× |
| cpu-rules-ml | 10 | 12.490 | 11.662 | 1.07× |
| cpu-rules-ml | 100 | 43.719 | 42.440 | 1.03× |
| cpu-rules-ml | 1000 | 390.619 | 375.415 | 1.04× |
| gpu-backend-ready | backend ready | 61.167 | 62.325 | 0.98× |
| gpu-ml | 1 (rule miss) | 55.506 | 55.898 | 0.99× |
| gpu-ml | 1 | 54.481 | 56.257 | 0.97× |
| gpu-ml | 2 | 66.639 | 67.706 | 0.98× |
| gpu-ml | 5 | 68.207 | 68.765 | 0.99× |
| gpu-ml | 10 | 68.508 | 69.024 | 0.99× |
| gpu-ml | 100 | 83.152 | 82.163 | 1.01× |
| gpu-ml | 1000 | 197.092 | 199.172 | 0.99× |
| gpu-rules-ml | 1 (rule miss) | 56.391 | 56.010 | 1.01× |
| gpu-rules-ml | 1 | 3.654 | 4.681 | 0.78× |
| gpu-rules-ml | 2 | 65.601 | 67.404 | 0.97× |
| gpu-rules-ml | 5 | 66.969 | 67.971 | 0.99× |
| gpu-rules-ml | 10 | 68.993 | 69.318 | 1.00× |
| gpu-rules-ml | 100 | 78.935 | 78.059 | 1.01× |
| gpu-rules-ml | 1000 | 152.795 | 152.722 | 1.00× |
| rules-only | 1 (rule miss) | 3.891 | 2.758 | 1.41× |
| rules-only | 1 | 3.534 | 2.462 | 1.44× |
| rules-only | 2 | 4.240 | 2.558 | 1.66× |
| rules-only | 5 | 3.617 | 2.936 | 1.23× |
| rules-only | 10 | 3.839 | 3.120 | 1.23× |
| rules-only | 100 | 7.148 | 6.338 | 1.13× |
| rules-only | 1000 | 34.895 | 33.910 | 1.03× |

Exact JSON output parity passed for 35 paired mode/workload checks covering 1118 distinct files. This is a focused replay, not a new full-corpus accuracy run.

Backend-ready rows include library loading, plan construction and the unchanged GPU correctness probe, but no user-file inference. ML-only one-file rows include first inference. Thousand-file rows include startup and processing; they are not isolated persistent-session throughput. Raw timings, ranges, commands, input hashes and artifact hashes are retained in JSON.

| Workload | Rules-only decisions / files | Coverage |
|---|---:|---:|
| files-1-hits-0 | 0 / 1 | 0.0% |
| files-1-hits-natural | 1 / 1 | 100.0% |
| files-2-hits-natural | 0 / 2 | 0.0% |
| files-5-hits-natural | 3 / 5 | 60.0% |
| files-10-hits-natural | 4 / 10 | 40.0% |
| files-100-hits-natural | 39 / 100 | 39.0% |
| files-1000-hits-natural | 382 / 1000 | 38.2% |
