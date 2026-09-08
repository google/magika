# Cross-tool benchmark 1.1.0

Revision: `24b86573616b9fe30a02920e8171064ae08b8dbc`. Status: complete.

Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.

| Tool             |   Files | Accuracy   | Precision   | Mapped coverage   | Macro recall   |   Wrong |   Errors |   Ambiguous |   Unmapped |   Rule hit % | Formats correct   |
|------------------|---------|------------|-------------|-------------------|----------------|---------|----------|-------------|------------|--------------|-------------------|
| rules-serialized |   25421 | 37.86%     | 100.00%     | 37.86%            | 39.34%         |       0 |        0 |           0 |          0 |              | 67/162            |
| rules-mapped     |   25421 | 37.86%     | 100.00%     | 37.86%            | 39.34%         |       0 |        0 |           0 |          0 |      37.8585 | 67/162            |

| Tool             |   Files |   Requested rule % |   Observed rule % |   Median ms |   Files/s |   Trials |
|------------------|---------|--------------------|-------------------|-------------|-----------|----------|
| rules-mapped     |       1 |                  0 |               0   |       9.839 |     101.6 |        3 |
| rules-serialized |       1 |                  0 |                   |      10.321 |      96.9 |        3 |
| rules-mapped     |       1 |                100 |             100   |       9.32  |     107.3 |        3 |
| rules-serialized |       1 |                100 |                   |      10.311 |      97   |        3 |
| rules-mapped     |       1 |                    |             100   |       9.634 |     103.8 |        3 |
| rules-serialized |       1 |                    |                   |      10.956 |      91.3 |        3 |
| rules-mapped     |       2 |                  0 |               0   |      10.212 |     195.9 |        3 |
| rules-serialized |       2 |                  0 |                   |      11.544 |     173.3 |        3 |
| rules-mapped     |       2 |                100 |             100   |      10.462 |     191.2 |        3 |
| rules-serialized |       2 |                100 |                   |      10.749 |     186.1 |        3 |
| rules-mapped     |       2 |                 50 |              50   |       9.779 |     204.5 |        3 |
| rules-serialized |       2 |                 50 |                   |      10.511 |     190.3 |        3 |
| rules-mapped     |       2 |                    |               0   |      10.666 |     187.5 |        3 |
| rules-serialized |       2 |                    |                   |      10.727 |     186.4 |        3 |
| rules-mapped     |       5 |                  0 |               0   |       9.974 |     501.3 |        3 |
| rules-serialized |       5 |                  0 |                   |      11.164 |     447.9 |        3 |
| rules-mapped     |       5 |                100 |             100   |       9.875 |     506.3 |        3 |
| rules-serialized |       5 |                100 |                   |      10.386 |     481.4 |        3 |
| rules-mapped     |       5 |                 20 |              20   |       9.732 |     513.8 |        3 |
| rules-serialized |       5 |                 20 |                   |      10.166 |     491.8 |        3 |
| rules-mapped     |       5 |                    |              60   |      10.486 |     476.8 |        3 |
| rules-serialized |       5 |                    |                   |      10.678 |     468.3 |        3 |
| rules-mapped     |      10 |                  0 |               0   |       9.905 |    1009.6 |        3 |
| rules-serialized |      10 |                  0 |                   |      10.781 |     927.6 |        3 |
| rules-mapped     |      10 |                100 |             100   |       9.641 |    1037.2 |        3 |
| rules-serialized |      10 |                100 |                   |      11.498 |     869.7 |        3 |
| rules-mapped     |      10 |                 20 |              20   |      10.63  |     940.7 |        3 |
| rules-serialized |      10 |                 20 |                   |      11.364 |     880   |        3 |
| rules-mapped     |      10 |                 50 |              50   |      10.433 |     958.5 |        3 |
| rules-serialized |      10 |                 50 |                   |      10.337 |     967.4 |        3 |
| rules-mapped     |      10 |                    |              40   |      10.779 |     927.7 |        3 |
| rules-serialized |      10 |                    |                   |      11.317 |     883.6 |        3 |
| rules-mapped     |      25 |                  0 |               0   |      11.085 |    2255.3 |        3 |
| rules-serialized |      25 |                  0 |                   |      11.269 |    2218.4 |        3 |
| rules-mapped     |      25 |                100 |             100   |      10.623 |    2353.5 |        3 |
| rules-serialized |      25 |                100 |                   |      11.889 |    2102.8 |        3 |
| rules-mapped     |      25 |                 20 |              20   |      10.408 |    2402   |        3 |
| rules-serialized |      25 |                 20 |                   |      11.58  |    2158.9 |        3 |
| rules-mapped     |      25 |                    |              32   |      10.55  |    2369.6 |        3 |
| rules-serialized |      25 |                    |                   |      11.673 |    2141.6 |        3 |
| rules-mapped     |     100 |                  0 |               0   |      13.136 |    7612.5 |        3 |
| rules-serialized |     100 |                  0 |                   |      14.449 |    6920.7 |        3 |
| rules-mapped     |     100 |                100 |             100   |      13.944 |    7171.7 |        3 |
| rules-serialized |     100 |                100 |                   |      13.989 |    7148.7 |        3 |
| rules-mapped     |     100 |                 20 |              20   |      13.108 |    7629   |        3 |
| rules-serialized |     100 |                 20 |                   |      13.816 |    7238   |        3 |
| rules-mapped     |     100 |                 50 |              50   |      13.3   |    7519.1 |        3 |
| rules-serialized |     100 |                 50 |                   |      14.11  |    7087   |        3 |
| rules-mapped     |     100 |                    |              39   |      14.446 |    6922.3 |        3 |
| rules-serialized |     100 |                    |                   |      15.018 |    6658.8 |        3 |
| rules-mapped     |    1000 |                  0 |               0   |      42.365 |   23604.1 |        3 |
| rules-serialized |    1000 |                  0 |                   |      46.836 |   21351.1 |        3 |
| rules-mapped     |    1000 |                100 |             100   |      47.892 |   20880.4 |        3 |
| rules-serialized |    1000 |                100 |                   |      48.626 |   20565   |        3 |
| rules-mapped     |    1000 |                 20 |              20   |      42.206 |   23693.4 |        3 |
| rules-serialized |    1000 |                 20 |                   |      50.55  |   19782.3 |        3 |
| rules-mapped     |    1000 |                 50 |              50   |      52.377 |   19092.3 |        3 |
| rules-serialized |    1000 |                 50 |                   |      50.24  |   19904.3 |        3 |
| rules-mapped     |    1000 |                    |              38.2 |      47.509 |   21048.6 |        3 |
| rules-serialized |    1000 |                    |                   |      43.984 |   22735.3 |        3 |

Rule ratios count signature decisions from the rules-only reference; unknowns abstain and no ML control runs. A dash is the natural sample mix or a tool without a rules mode.

Common unambiguous vocabulary: 212 classes, 15230 files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.

| Tool             | Common-vocabulary accuracy   | Mapped coverage   | Macro recall   |
|------------------|------------------------------|-------------------|----------------|
| rules-serialized | 27.95%                       | 27.95%            | 34.63%         |
| rules-mapped     | 27.95%                       | 27.95%            | 34.63%         |

