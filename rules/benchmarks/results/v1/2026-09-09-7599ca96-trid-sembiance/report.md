# Cross-tool benchmark 1.2.2

Revision: `7599ca9665df180e5aa39e7f91376a61a354151d`. Status: complete.

Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.

| Tool               |   Files | Accuracy   | Precision   | Mapped coverage   | Macro recall   |   Wrong |   Errors |   Ambiguous |   Unmapped |   Rule hit % | Formats correct   |
|--------------------|---------|------------|-------------|-------------------|----------------|---------|----------|-------------|------------|--------------|-------------------|
| magika2-rules-only |    2400 | 22.21%     | 100.00%     | 22.21%            | 28.90%         |       0 |        0 |           0 |          0 |      22.2083 | 53/167            |
| trid               |    2400 | 42.58%     | 90.68%      | 46.96%            | 55.41%         |     105 |        0 |         402 |        475 |              | 108/167           |
| trid-stringzilla   |    2400 | 42.58%     | 90.68%      | 46.96%            | 55.41%         |     105 |        0 |         402 |        475 |              | 108/167           |

| Tool               |   Files | Requested rule %   |   Observed rule % |   Median ms |   Files/s |   Trials |
|--------------------|---------|--------------------|-------------------|-------------|-----------|----------|
| magika2-rules-only |       1 |                    |             100   |       2.843 |     351.8 |        3 |
| trid               |       1 |                    |                   |      69.244 |      14.4 |        3 |
| trid-stringzilla   |       1 |                    |                   |      68.437 |      14.6 |        3 |
| magika2-rules-only |       2 |                    |               0   |       2.843 |     703.4 |        3 |
| trid               |       2 |                    |                   |      72.675 |      27.5 |        3 |
| trid-stringzilla   |       2 |                    |                   |      72.539 |      27.6 |        3 |
| magika2-rules-only |       5 |                    |              20   |       4.228 |    1182.7 |        3 |
| trid               |       5 |                    |                   |      88.71  |      56.4 |        3 |
| trid-stringzilla   |       5 |                    |                   |      70.108 |      71.3 |        3 |
| magika2-rules-only |      10 |                    |              20   |       4.188 |    2387.6 |        3 |
| trid               |      10 |                    |                   |      87.844 |     113.8 |        3 |
| trid-stringzilla   |      10 |                    |                   |      95.546 |     104.7 |        3 |
| magika2-rules-only |      25 |                    |              24   |       5.061 |    4939.3 |        3 |
| trid               |      25 |                    |                   |     166.841 |     149.8 |        3 |
| trid-stringzilla   |      25 |                    |                   |      88.672 |     281.9 |        3 |
| magika2-rules-only |     100 |                    |              29   |       9.686 |   10323.9 |        3 |
| trid               |     100 |                    |                   |     179.313 |     557.7 |        3 |
| trid-stringzilla   |     100 |                    |                   |     112.531 |     888.6 |        3 |
| magika2-rules-only |    1000 |                    |              22.9 |      56.551 |   17683.2 |        3 |
| trid               |    1000 |                    |                   |    1464.41  |     682.9 |        3 |
| trid-stringzilla   |    1000 |                    |                   |     444.247 |    2251   |        3 |

Rule ratios count signature decisions from the rules-only reference; unknowns abstain and no ML control runs. A dash is the natural sample mix or a tool without a rules mode.

Common unambiguous vocabulary: 183 classes, 1262 files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.

| Tool               | Common-vocabulary accuracy   | Mapped coverage   | Macro recall   |
|--------------------|------------------------------|-------------------|----------------|
| magika2-rules-only | 26.62%                       | 26.62%            | 26.59%         |
| trid               | 61.81%                       | 67.67%            | 64.75%         |
| trid-stringzilla   | 61.81%                       | 67.67%            | 64.75%         |

