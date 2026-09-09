# Cross-tool benchmark 1.1.0

Revision: `754c063e-fusion-dirty`. Status: complete.

Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.

| Tool            |   Files | Accuracy   | Precision   | Mapped coverage   | Macro recall   |   Wrong |   Errors |   Ambiguous |   Unmapped |   Rule hit % | Formats correct   |
|-----------------|---------|------------|-------------|-------------------|----------------|---------|----------|-------------|------------|--------------|-------------------|
| before          |   25421 | 57.73%     | 70.23%      | 82.20%            | 56.82%         |    6220 |        0 |           0 |          0 |              | 95/162            |
| after           |   25421 | 57.73%     | 70.23%      | 82.20%            | 56.82%         |    6220 |        0 |           0 |          0 |              | 95/162            |
| rules-reference |   25421 | 37.86%     | 100.00%     | 37.86%            | 39.34%         |       0 |        0 |           0 |          0 |      37.8585 | 67/162            |

| Tool            |   Files | Requested rule %   |   Observed rule % |   Median ms |   Files/s |   Trials |
|-----------------|---------|--------------------|-------------------|-------------|-----------|----------|
| after           |       1 |                    |                   |       6.426 |     155.6 |        5 |
| before          |       1 |                    |                   |       6.604 |     151.4 |        5 |
| rules-reference |       1 |                    |             100   |       3.49  |     286.5 |        5 |
| after           |       2 |                    |                   |       6.551 |     305.3 |        5 |
| before          |       2 |                    |                   |       7.817 |     255.8 |        5 |
| rules-reference |       2 |                    |               0   |       3.302 |     605.8 |        5 |
| after           |       5 |                    |                   |      13.465 |     371.3 |        5 |
| before          |       5 |                    |                   |      12.538 |     398.8 |        5 |
| rules-reference |       5 |                    |              60   |       3.941 |    1268.7 |        5 |
| after           |      10 |                    |                   |      17.173 |     582.3 |        5 |
| before          |      10 |                    |                   |      16.28  |     614.2 |        5 |
| rules-reference |      10 |                    |              40   |       4.514 |    2215.4 |        5 |
| after           |     100 |                    |                   |      71.88  |    1391.2 |        5 |
| before          |     100 |                    |                   |      71.619 |    1396.3 |        5 |
| rules-reference |     100 |                    |              39   |       8.232 |   12148.3 |        5 |
| after           |    1000 |                    |                   |     625.532 |    1598.6 |        5 |
| before          |    1000 |                    |                   |     605.735 |    1650.9 |        5 |
| rules-reference |    1000 |                    |              38.2 |      39.411 |   25373.7 |        5 |

Rule ratios count signature decisions from the rules-only reference; unknowns abstain and no ML control runs. A dash is the natural sample mix or a tool without a rules mode.

Common unambiguous vocabulary: 212 classes, 15230 files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.

| Tool            | Common-vocabulary accuracy   | Mapped coverage   | Macro recall   |
|-----------------|------------------------------|-------------------|----------------|
| before          | 96.36%                       | 99.88%            | 96.89%         |
| after           | 96.36%                       | 99.88%            | 96.89%         |
| rules-reference | 27.95%                       | 27.95%            | 34.63%         |

