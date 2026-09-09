# Cross-tool benchmark 1.1.0

Revision: `dd99c69c1b0de205937dedf7ebb58fd16e675905`. Status: complete.

Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.

| Tool            |   Files | Accuracy   | Precision   | Mapped coverage   | Macro recall   |   Wrong |   Errors |   Ambiguous |   Unmapped |   Rule hit % | Formats correct   |
|-----------------|---------|------------|-------------|-------------------|----------------|---------|----------|-------------|------------|--------------|-------------------|
| before          |    1018 | 58.15%     | 70.56%      | 82.42%            | 59.88%         |     247 |        0 |           0 |          0 |              | 94/153            |
| after           |    1018 | 58.15%     | 70.56%      | 82.42%            | 59.88%         |     247 |        0 |           0 |          0 |              | 94/153            |
| rules-reference |    1018 | 38.31%     | 100.00%     | 38.31%            | 38.95%         |       0 |        0 |           0 |          0 |      38.3104 | 62/153            |

| Tool            |   Files | Requested rule %   |   Observed rule % |   Median ms |   Files/s |   Trials |
|-----------------|---------|--------------------|-------------------|-------------|-----------|----------|
| after           |       1 |                    |                   |       6.043 |     165.5 |        5 |
| before          |       1 |                    |                   |       5.989 |     167   |        5 |
| rules-reference |       1 |                    |             100   |       3.134 |     319.1 |        5 |
| after           |       2 |                    |                   |       7.585 |     263.7 |        5 |
| before          |       2 |                    |                   |      12.14  |     164.7 |        5 |
| rules-reference |       2 |                    |               0   |       3.279 |     610   |        5 |
| after           |       5 |                    |                   |      12.157 |     411.3 |        5 |
| before          |       5 |                    |                   |      12.297 |     406.6 |        5 |
| rules-reference |       5 |                    |              60   |       3.262 |    1532.7 |        5 |
| after           |      10 |                    |                   |      13.456 |     743.2 |        5 |
| before          |      10 |                    |                   |      13.539 |     738.6 |        5 |
| rules-reference |      10 |                    |              40   |       3.534 |    2829.3 |        5 |
| after           |    1000 |                    |                   |     604.552 |    1654.1 |        5 |
| before          |    1000 |                    |                   |     611.864 |    1634.4 |        5 |
| rules-reference |    1000 |                    |              38.2 |      35.458 |   28202.6 |        5 |

Rule ratios count signature decisions from the rules-only reference; unknowns abstain and no ML control runs. A dash is the natural sample mix or a tool without a rules mode.

Common unambiguous vocabulary: 212 classes, 617 files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.

| Tool            | Common-vocabulary accuracy   | Mapped coverage   | Macro recall   |
|-----------------|------------------------------|-------------------|----------------|
| before          | 95.95%                       | 100.00%           | 97.46%         |
| after           | 95.95%                       | 100.00%           | 97.46%         |
| rules-reference | 25.93%                       | 25.93%            | 34.23%         |

