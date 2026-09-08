# Cross-tool benchmark 1.0.0

Revision: `425a5e38`. Status: partial.

Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.

| Tool              |   Files | Accuracy   | Precision   | Mapped coverage   | Macro recall   |   Wrong |   Errors |   Ambiguous |   Unmapped |   Rule hit % | Formats correct   |
|-------------------|---------|------------|-------------|-------------------|----------------|---------|----------|-------------|------------|--------------|-------------------|
| magika2-gpu-ml    |   25421 | 57.73%     | 70.22%      | 82.21%            | 56.82%         |    6223 |        0 |           0 |          0 |              | 95/162            |
| magika2-gpu-rules |   25421 | 78.93%     | 84.38%      | 93.53%            | 75.96%         |    3713 |        0 |           0 |          0 |      37.8585 | 127/162           |

| Tool              |   Files |   Requested rule % |   Observed rule % |   Median ms |   Files/s |   Trials |
|-------------------|---------|--------------------|-------------------|-------------|-----------|----------|
| magika2-gpu-ml    |       1 |                  0 |                   |      70.516 |      14.2 |        3 |
| magika2-gpu-rules |       1 |                  0 |               0   |      73.888 |      13.5 |        3 |
| magika2-gpu-ml    |       1 |                100 |                   |      71.018 |      14.1 |        3 |
| magika2-gpu-rules |       1 |                100 |             100   |      10.319 |      96.9 |        3 |
| magika2-gpu-ml    |       1 |                    |                   |      70.47  |      14.2 |        3 |
| magika2-gpu-rules |       1 |                    |             100   |      10.464 |      95.6 |        3 |
| magika2-gpu-ml    |       2 |                  0 |                   |     105.537 |      19   |        3 |
| magika2-gpu-rules |       2 |                  0 |               0   |      84.086 |      23.8 |        3 |
| magika2-gpu-ml    |       2 |                100 |                   |      78.85  |      25.4 |        3 |
| magika2-gpu-rules |       2 |                100 |             100   |      11.162 |     179.2 |        3 |
| magika2-gpu-ml    |       2 |                 50 |                   |      81.402 |      24.6 |        3 |
| magika2-gpu-rules |       2 |                 50 |              50   |      81.906 |      24.4 |        3 |
| magika2-gpu-ml    |       2 |                    |                   |      78.382 |      25.5 |        3 |
| magika2-gpu-rules |       2 |                    |               0   |      82.988 |      24.1 |        3 |
| magika2-gpu-ml    |       5 |                  0 |                   |      84.09  |      59.5 |        3 |
| magika2-gpu-rules |       5 |                  0 |               0   |      85.511 |      58.5 |        3 |
| magika2-gpu-ml    |       5 |                100 |                   |      79.571 |      62.8 |        3 |
| magika2-gpu-rules |       5 |                100 |             100   |      10.202 |     490.1 |        3 |
| magika2-gpu-ml    |       5 |                 20 |                   |      79.545 |      62.9 |        3 |
| magika2-gpu-rules |       5 |                 20 |              20   |      78.766 |      63.5 |        3 |
| magika2-gpu-ml    |       5 |                    |                   |      78.912 |      63.4 |        3 |
| magika2-gpu-rules |       5 |                    |              60   |      78.396 |      63.8 |        3 |
| magika2-gpu-ml    |      10 |                  0 |                   |      79.87  |     125.2 |        3 |
| magika2-gpu-rules |      10 |                  0 |               0   |      79.186 |     126.3 |        3 |
| magika2-gpu-ml    |      10 |                100 |                   |      87.979 |     113.7 |        3 |
| magika2-gpu-rules |      10 |                100 |             100   |      10.79  |     926.8 |        3 |
| magika2-gpu-ml    |      10 |                 20 |                   |      79.422 |     125.9 |        3 |
| magika2-gpu-rules |      10 |                 20 |              20   |      87.461 |     114.3 |        3 |
| magika2-gpu-ml    |      10 |                 50 |                   |      79.614 |     125.6 |        3 |
| magika2-gpu-rules |      10 |                 50 |              50   |      79.144 |     126.4 |        3 |
| magika2-gpu-ml    |      10 |                    |                   |      79.281 |     126.1 |        3 |
| magika2-gpu-rules |      10 |                    |              40   |      84.139 |     118.9 |        3 |
| magika2-gpu-ml    |      25 |                  0 |                   |      78.999 |     316.5 |        3 |
| magika2-gpu-rules |      25 |                  0 |               0   |      79.091 |     316.1 |        3 |
| magika2-gpu-ml    |      25 |                100 |                   |      88.06  |     283.9 |        3 |
| magika2-gpu-rules |      25 |                100 |             100   |      11.528 |    2168.7 |        3 |
| magika2-gpu-ml    |      25 |                 20 |                   |      79.122 |     316   |        3 |
| magika2-gpu-rules |      25 |                 20 |              20   |      79.114 |     316   |        3 |
| magika2-gpu-ml    |      25 |                    |                   |      87.389 |     286.1 |        3 |
| magika2-gpu-rules |      25 |                    |              32   |      87.186 |     286.7 |        3 |
| magika2-gpu-ml    |     100 |                  0 |                   |      87.812 |    1138.8 |        3 |
| magika2-gpu-rules |     100 |                  0 |               0   |      97.028 |    1030.6 |        3 |
| magika2-gpu-ml    |     100 |                100 |                   |      95.767 |    1044.2 |        3 |
| magika2-gpu-rules |     100 |                100 |             100   |      14.847 |    6735.3 |        3 |
| magika2-gpu-ml    |     100 |                 20 |                   |      87.798 |    1139   |        3 |
| magika2-gpu-rules |     100 |                 20 |              20   |      93.532 |    1069.1 |        3 |
| magika2-gpu-ml    |     100 |                 50 |                   |      87.083 |    1148.3 |        3 |
| magika2-gpu-rules |     100 |                 50 |              50   |      87.314 |    1145.3 |        3 |
| magika2-gpu-ml    |     100 |                    |                   |      97.256 |    1028.2 |        3 |
| magika2-gpu-rules |     100 |                    |              39   |      87.374 |    1144.5 |        3 |
| magika2-gpu-ml    |    1000 |                  0 |                   |     234.731 |    4260.2 |        3 |
| magika2-gpu-rules |    1000 |                  0 |               0   |     230.276 |    4342.6 |        3 |
| magika2-gpu-ml    |    1000 |                100 |                   |     226.238 |    4420.1 |        3 |
| magika2-gpu-rules |    1000 |                100 |             100   |      52.102 |   19193.3 |        3 |
| magika2-gpu-ml    |    1000 |                 20 |                   |     229.641 |    4354.6 |        3 |
| magika2-gpu-rules |    1000 |                 20 |              20   |     197.512 |    5063   |        3 |
| magika2-gpu-ml    |    1000 |                 50 |                   |     229.797 |    4351.7 |        3 |
| magika2-gpu-rules |    1000 |                 50 |              50   |     164.124 |    6093   |        3 |
| magika2-gpu-ml    |    1000 |                    |                   |     226.505 |    4414.9 |        3 |
| magika2-gpu-rules |    1000 |                    |              38.2 |     170.765 |    5856   |        3 |

Rule ratios refer to the named Magika rules reference versus its ML-only control. A dash is the natural sample mix or a tool without a rules mode.

Common unambiguous vocabulary: 212 classes, 15230 files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.

| Tool              | Common-vocabulary accuracy   | Mapped coverage   | Macro recall   |
|-------------------|------------------------------|-------------------|----------------|
| magika2-gpu-ml    | 96.36%                       | 99.89%            | 96.89%         |
| magika2-gpu-rules | 96.50%                       | 99.90%            | 97.07%         |

| Unavailable tool   | Reason                                                               |
|--------------------|----------------------------------------------------------------------|
| libmagic-gpu       | file/libmagic is a CPU CLI.                                          |
| magika1-gpu        | Released Rust CLI 1.1.0 exposes no GPU execution-provider selection. |
| trid-gpu           | TrID 2.48 is a CPU CLI.                                              |

