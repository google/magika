# Cross-tool benchmark 1.2.2

Revision: `7599ca9665df180e5aa39e7f91376a61a354151d`. Status: complete.

Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.

| Tool               |   Files | Accuracy   | Precision   | Mapped coverage   | Macro recall   |   Wrong |   Errors |   Ambiguous |   Unmapped |   Rule hit % | Formats correct   |
|--------------------|---------|------------|-------------|-------------------|----------------|---------|----------|-------------|------------|--------------|-------------------|
| magika2-rules-only |   25421 | 37.86%     | 100.00%     | 37.86%            | 39.34%         |       0 |        0 |           0 |          0 |      37.8585 | 67/162            |
| trid               |   25421 | 55.25%     | 78.32%      | 70.55%            | 63.10%         |    3888 |        0 |        4844 |       1402 |              | 119/162           |
| trid-stringzilla   |   25421 | 55.25%     | 78.32%      | 70.55%            | 63.10%         |    3888 |        0 |        4844 |       1402 |              | 119/162           |

| Tool               |   Files | Requested rule %   |   Observed rule % |   Median ms |   Files/s |   Trials |
|--------------------|---------|--------------------|-------------------|-------------|-----------|----------|
| magika2-rules-only |       1 |                    |             100   |       2.78  |     359.7 |        3 |
| trid               |       1 |                    |                   |      93.542 |      10.7 |        3 |
| trid-stringzilla   |       1 |                    |                   |      79.122 |      12.6 |        3 |
| magika2-rules-only |       2 |                    |               0   |       3.46  |     578   |        3 |
| trid               |       2 |                    |                   |     105.955 |      18.9 |        3 |
| trid-stringzilla   |       2 |                    |                   |      76.439 |      26.2 |        3 |
| magika2-rules-only |       5 |                    |              60   |       3.336 |    1498.7 |        3 |
| trid               |       5 |                    |                   |      77.059 |      64.9 |        3 |
| trid-stringzilla   |       5 |                    |                   |      94.445 |      52.9 |        3 |
| magika2-rules-only |      10 |                    |              40   |       3.24  |    3086.4 |        3 |
| trid               |      10 |                    |                   |     101.501 |      98.5 |        3 |
| trid-stringzilla   |      10 |                    |                   |      87.216 |     114.7 |        3 |
| magika2-rules-only |      25 |                    |              32   |       4.117 |    6072.6 |        3 |
| trid               |      25 |                    |                   |      85.865 |     291.2 |        3 |
| trid-stringzilla   |      25 |                    |                   |      85.496 |     292.4 |        3 |
| magika2-rules-only |     100 |                    |              39   |       8.591 |   11640   |        3 |
| trid               |     100 |                    |                   |     278.25  |     359.4 |        3 |
| trid-stringzilla   |     100 |                    |                   |     130.401 |     766.9 |        3 |
| magika2-rules-only |    1000 |                    |              38.2 |      64.964 |   15393.2 |        3 |
| trid               |    1000 |                    |                   |    4775.78  |     209.4 |        3 |
| trid-stringzilla   |    1000 |                    |                   |    1118.27  |     894.2 |        3 |

Rule ratios count signature decisions from the rules-only reference; unknowns abstain and no ML control runs. A dash is the natural sample mix or a tool without a rules mode.

Common unambiguous vocabulary: 183 classes, 14182 files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.

| Tool               | Common-vocabulary accuracy   | Mapped coverage   | Macro recall   |
|--------------------|------------------------------|-------------------|----------------|
| magika2-rules-only | 28.35%                       | 28.35%            | 35.92%         |
| trid               | 60.72%                       | 84.31%            | 71.96%         |
| trid-stringzilla   | 60.72%                       | 84.31%            | 71.96%         |

