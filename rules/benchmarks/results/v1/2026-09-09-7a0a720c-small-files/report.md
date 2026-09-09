# Cross-tool benchmark 1.1.0

Revision: `7a0a720c3d4b5e85782ba0f7e0b24cab3e983951`. Status: complete.

Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.

| Tool            |   Files | Accuracy   | Precision   | Mapped coverage   | Macro recall   |   Wrong |   Errors |   Ambiguous |   Unmapped |   Rule hit % | Formats correct   |
|-----------------|---------|------------|-------------|-------------------|----------------|---------|----------|-------------|------------|--------------|-------------------|
| before          |    1018 | 58.15%     | 70.56%      | 82.42%            | 59.88%         |     247 |        0 |           0 |          0 |              | 94/153            |
| after           |    1018 | 58.15%     | 70.56%      | 82.42%            | 59.88%         |     247 |        0 |           0 |          0 |              | 94/153            |
| rules-reference |    1018 | 38.31%     | 100.00%     | 38.31%            | 38.95%         |       0 |        0 |           0 |          0 |      38.3104 | 62/153            |

| Tool            |   Files | Requested rule %   |   Observed rule % |   Median ms |   Files/s |   Trials |
|-----------------|---------|--------------------|-------------------|-------------|-----------|----------|
| after           |       1 |                    |                   |       6.456 |     154.9 |        5 |
| before          |       1 |                    |                   |       6.263 |     159.7 |        5 |
| rules-reference |       1 |                    |             100   |       3.266 |     306.2 |        5 |
| after           |       2 |                    |                   |       7.714 |     259.3 |        5 |
| before          |       2 |                    |                   |      13.872 |     144.2 |        5 |
| rules-reference |       2 |                    |               0   |       3.756 |     532.6 |        5 |
| after           |       5 |                    |                   |      16.634 |     300.6 |        5 |
| before          |       5 |                    |                   |      12.199 |     409.9 |        5 |
| rules-reference |       5 |                    |              60   |       3.878 |    1289.2 |        5 |
| after           |      10 |                    |                   |      13.944 |     717.2 |        5 |
| before          |      10 |                    |                   |      16.007 |     624.7 |        5 |
| rules-reference |      10 |                    |              40   |       3.601 |    2777.3 |        5 |
| after           |    1000 |                    |                   |     615.024 |    1626   |        5 |
| before          |    1000 |                    |                   |     625.788 |    1598   |        5 |
| rules-reference |    1000 |                    |              38.2 |      37.949 |   26351.3 |        5 |

Rule ratios count signature decisions from the rules-only reference; unknowns abstain and no ML control runs. A dash is the natural sample mix or a tool without a rules mode.

Common unambiguous vocabulary: 212 classes, 617 files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.

| Tool            | Common-vocabulary accuracy   | Mapped coverage   | Macro recall   |
|-----------------|------------------------------|-------------------|----------------|
| before          | 95.95%                       | 100.00%           | 97.46%         |
| after           | 95.95%                       | 100.00%           | 97.46%         |
| rules-reference | 25.93%                       | 25.93%            | 34.23%         |

