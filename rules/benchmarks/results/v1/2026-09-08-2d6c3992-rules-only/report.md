# Cross-tool benchmark 1.1.0

Revision: `2d6c39928b157746630a2efdd4da874cc90da75c`. Status: complete.

Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.

| Tool               |   Files | Accuracy   | Precision   | Mapped coverage   | Macro recall   |   Wrong |   Errors |   Ambiguous |   Unmapped |   Rule hit % | Formats correct   |
|--------------------|---------|------------|-------------|-------------------|----------------|---------|----------|-------------|------------|--------------|-------------------|
| magika2-rules-only |   25421 | 37.86%     | 100.00%     | 37.86%            | 39.34%         |       0 |        0 |           0 |          0 |      37.8585 | 67/162            |

| Tool               |   Files |   Requested rule % |   Observed rule % |   Median ms |   Files/s |   Trials |
|--------------------|---------|--------------------|-------------------|-------------|-----------|----------|
| magika2-rules-only |       1 |                  0 |               0   |       7.691 |     130   |        3 |
| magika2-rules-only |       1 |                100 |             100   |       7.807 |     128.1 |        3 |
| magika2-rules-only |       1 |                    |             100   |       7.634 |     131   |        3 |
| magika2-rules-only |       2 |                  0 |               0   |       9.23  |     216.7 |        3 |
| magika2-rules-only |       2 |                100 |             100   |       9.499 |     210.6 |        3 |
| magika2-rules-only |       2 |                 50 |              50   |       7.874 |     254   |        3 |
| magika2-rules-only |       2 |                    |               0   |       8.509 |     235.1 |        3 |
| magika2-rules-only |       5 |                  0 |               0   |       8.541 |     585.4 |        3 |
| magika2-rules-only |       5 |                100 |             100   |       8.919 |     560.6 |        3 |
| magika2-rules-only |       5 |                 20 |              20   |       7.875 |     634.9 |        3 |
| magika2-rules-only |       5 |                    |              60   |       9.135 |     547.4 |        3 |
| magika2-rules-only |      10 |                  0 |               0   |       8.448 |    1183.7 |        3 |
| magika2-rules-only |      10 |                100 |             100   |       8.157 |    1226   |        3 |
| magika2-rules-only |      10 |                 20 |              20   |       8.067 |    1239.6 |        3 |
| magika2-rules-only |      10 |                 50 |              50   |       8.371 |    1194.6 |        3 |
| magika2-rules-only |      10 |                    |              40   |       9.922 |    1007.9 |        3 |
| magika2-rules-only |      25 |                  0 |               0   |       9.917 |    2521   |        3 |
| magika2-rules-only |      25 |                100 |             100   |       9.15  |    2732.4 |        3 |
| magika2-rules-only |      25 |                 20 |              20   |       8.776 |    2848.6 |        3 |
| magika2-rules-only |      25 |                    |              32   |       8.607 |    2904.6 |        3 |
| magika2-rules-only |     100 |                  0 |               0   |      11.397 |    8774.2 |        3 |
| magika2-rules-only |     100 |                100 |             100   |      12.579 |    7949.7 |        3 |
| magika2-rules-only |     100 |                 20 |              20   |      11.764 |    8500.5 |        3 |
| magika2-rules-only |     100 |                 50 |              50   |      12.006 |    8329   |        3 |
| magika2-rules-only |     100 |                    |              39   |      11.794 |    8478.6 |        3 |
| magika2-rules-only |    1000 |                  0 |               0   |      47.932 |   20862.8 |        3 |
| magika2-rules-only |    1000 |                100 |             100   |      44.479 |   22482.7 |        3 |
| magika2-rules-only |    1000 |                 20 |              20   |      50.114 |   19954.6 |        3 |
| magika2-rules-only |    1000 |                 50 |              50   |      47.124 |   21220.6 |        3 |
| magika2-rules-only |    1000 |                    |              38.2 |      48.368 |   20675   |        3 |

Rule ratios count signature decisions from the rules-only reference; unknowns abstain and no ML control runs. A dash is the natural sample mix or a tool without a rules mode.

Common unambiguous vocabulary: 212 classes, 15230 files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.

| Tool               | Common-vocabulary accuracy   | Mapped coverage   | Macro recall   |
|--------------------|------------------------------|-------------------|----------------|
| magika2-rules-only | 27.95%                       | 27.95%            | 34.63%         |

