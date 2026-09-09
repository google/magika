# Cross-tool benchmark 1.2.1

Revision: `a8429b03b514730254e628a12c2b0cf48591a480`. Status: complete.

Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.

| Tool               |   Files | Accuracy   | Precision   | Mapped coverage   | Macro recall   |   Wrong |   Errors |   Ambiguous |   Unmapped |   Rule hit % | Formats correct   |
|--------------------|---------|------------|-------------|-------------------|----------------|---------|----------|-------------|------------|--------------|-------------------|
| magika1            |   25421 | 57.73%     | 70.23%      | 82.20%            | 56.82%         |    6220 |        0 |           0 |          0 |              | 95/162            |
| magika2-ml         |   25421 | 57.73%     | 70.23%      | 82.20%            | 56.82%         |    6220 |        0 |           0 |          0 |              | 95/162            |
| magika2-rules      |   25421 | 78.93%     | 84.39%      | 93.52%            | 75.96%         |    3710 |        0 |           0 |          0 |              | 127/162           |
| magika2-gpu-ml     |   25421 | 57.73%     | 70.23%      | 82.20%            | 56.82%         |    6220 |        0 |           0 |          0 |              | 95/162            |
| magika2-gpu-rules  |   25421 | 78.93%     | 84.39%      | 93.52%            | 75.96%         |    3710 |        0 |           0 |          0 |              | 127/162           |
| magika2-auto-ml    |   25421 | 57.73%     | 70.23%      | 82.20%            | 56.82%         |    6220 |        0 |           0 |          0 |              | 95/162            |
| magika2-auto-rules |   25421 | 78.93%     | 84.39%      | 93.52%            | 75.96%         |    3710 |        0 |           0 |          0 |              | 127/162           |
| magika2-rules-only |   25421 | 37.86%     | 100.00%     | 37.86%            | 39.34%         |       0 |        0 |           0 |          0 |      37.8585 | 67/162            |
| libmagic           |   25421 | 25.46%     | 93.24%      | 27.31%            | 35.55%         |     469 |        0 |        8039 |       3801 |              | 63/162            |
| trid               |   25421 | 55.25%     | 78.32%      | 70.55%            | 63.10%         |    3888 |        0 |        4844 |       1402 |              | 119/162           |

| Tool               |   Files | Requested rule %   |   Observed rule % |   Median ms |   Files/s |   Trials |
|--------------------|---------|--------------------|-------------------|-------------|-----------|----------|
| libmagic           |       1 |                    |                   |       1.382 |     723.4 |        3 |
| magika1            |       1 |                    |                   |      63.711 |      15.7 |        3 |
| magika2-auto-ml    |       1 |                    |                   |       6.832 |     146.4 |        3 |
| magika2-auto-rules |       1 |                    |                   |       3.522 |     283.9 |        3 |
| magika2-gpu-ml     |       1 |                    |                   |       7.514 |     133.1 |        3 |
| magika2-gpu-rules  |       1 |                    |                   |       3.63  |     275.5 |        3 |
| magika2-ml         |       1 |                    |                   |       5.533 |     180.7 |        3 |
| magika2-rules      |       1 |                    |                   |       3.773 |     265   |        3 |
| magika2-rules-only |       1 |                    |             100   |       3.208 |     311.7 |        3 |
| trid               |       1 |                    |                   |      84.236 |      11.9 |        3 |
| libmagic           |       2 |                    |                   |       2.067 |     967.6 |        3 |
| magika1            |       2 |                    |                   |      75.418 |      26.5 |        3 |
| magika2-auto-ml    |       2 |                    |                   |       8.903 |     224.6 |        3 |
| magika2-auto-rules |       2 |                    |                   |       7.131 |     280.5 |        3 |
| magika2-gpu-ml     |       2 |                    |                   |       8.794 |     227.4 |        3 |
| magika2-gpu-rules  |       2 |                    |                   |       7.007 |     285.4 |        3 |
| magika2-ml         |       2 |                    |                   |       7.72  |     259.1 |        3 |
| magika2-rules      |       2 |                    |                   |       7.571 |     264.1 |        3 |
| magika2-rules-only |       2 |                    |               0   |       2.778 |     720   |        3 |
| trid               |       2 |                    |                   |     113.601 |      17.6 |        3 |
| libmagic           |       5 |                    |                   |       3.995 |    1251.5 |        3 |
| magika1            |       5 |                    |                   |      59.294 |      84.3 |        3 |
| magika2-auto-ml    |       5 |                    |                   |      14.501 |     344.8 |        3 |
| magika2-auto-rules |       5 |                    |                   |      13.793 |     362.5 |        3 |
| magika2-gpu-ml     |       5 |                    |                   |      13.591 |     367.9 |        3 |
| magika2-gpu-rules  |       5 |                    |                   |      16.35  |     305.8 |        3 |
| magika2-ml         |       5 |                    |                   |      12.295 |     406.7 |        3 |
| magika2-rules      |       5 |                    |                   |      12.02  |     416   |        3 |
| magika2-rules-only |       5 |                    |              60   |       3.851 |    1298.3 |        3 |
| trid               |       5 |                    |                   |      79.067 |      63.2 |        3 |
| libmagic           |      10 |                    |                   |       5.624 |    1778.1 |        3 |
| magika1            |      10 |                    |                   |      64.932 |     154   |        3 |
| magika2-auto-ml    |      10 |                    |                   |      21.085 |     474.3 |        3 |
| magika2-auto-rules |      10 |                    |                   |      13.465 |     742.7 |        3 |
| magika2-gpu-ml     |      10 |                    |                   |      17.931 |     557.7 |        3 |
| magika2-gpu-rules  |      10 |                    |                   |      13.923 |     718.2 |        3 |
| magika2-ml         |      10 |                    |                   |      16.027 |     623.9 |        3 |
| magika2-rules      |      10 |                    |                   |      12.126 |     824.6 |        3 |
| magika2-rules-only |      10 |                    |              40   |       3.438 |    2908.4 |        3 |
| trid               |      10 |                    |                   |     111.666 |      89.6 |        3 |
| libmagic           |      25 |                    |                   |      17.268 |    1447.8 |        3 |
| magika1            |      25 |                    |                   |      65.462 |     381.9 |        3 |
| magika2-auto-ml    |      25 |                    |                   |      24.355 |    1026.5 |        3 |
| magika2-auto-rules |      25 |                    |                   |      20.52  |    1218.4 |        3 |
| magika2-gpu-ml     |      25 |                    |                   |      21.232 |    1177.5 |        3 |
| magika2-gpu-rules  |      25 |                    |                   |      28.514 |     876.8 |        3 |
| magika2-ml         |      25 |                    |                   |      19.914 |    1255.4 |        3 |
| magika2-rules      |      25 |                    |                   |      16.801 |    1488   |        3 |
| magika2-rules-only |      25 |                    |              32   |       4.49  |    5567.4 |        3 |
| trid               |      25 |                    |                   |     105.357 |     237.3 |        3 |
| libmagic           |     100 |                    |                   |      36.274 |    2756.8 |        3 |
| magika1            |     100 |                    |                   |     110.18  |     907.6 |        3 |
| magika2-auto-ml    |     100 |                    |                   |      53.291 |    1876.5 |        3 |
| magika2-auto-rules |     100 |                    |                   |      38.138 |    2622.1 |        3 |
| magika2-gpu-ml     |     100 |                    |                   |      45.144 |    2215.1 |        3 |
| magika2-gpu-rules  |     100 |                    |                   |      37.227 |    2686.3 |        3 |
| magika2-ml         |     100 |                    |                   |      46.493 |    2150.9 |        3 |
| magika2-rules      |     100 |                    |                   |      30.791 |    3247.7 |        3 |
| magika2-rules-only |     100 |                    |              39   |       9.767 |   10238.9 |        3 |
| trid               |     100 |                    |                   |     320.119 |     312.4 |        3 |
| libmagic           |    1000 |                    |                   |     381.772 |    2619.4 |        3 |
| magika1            |    1000 |                    |                   |     539.247 |    1854.4 |        3 |
| magika2-auto-ml    |    1000 |                    |                   |     335.476 |    2980.8 |        3 |
| magika2-auto-rules |    1000 |                    |                   |     226.017 |    4424.4 |        3 |
| magika2-gpu-ml     |    1000 |                    |                   |     346.01  |    2890.1 |        3 |
| magika2-gpu-rules  |    1000 |                    |                   |     250.524 |    3991.6 |        3 |
| magika2-ml         |    1000 |                    |                   |     310.062 |    3225.2 |        3 |
| magika2-rules      |    1000 |                    |                   |     225.441 |    4435.8 |        3 |
| magika2-rules-only |    1000 |                    |              38.2 |      83.141 |   12027.8 |        3 |
| trid               |    1000 |                    |                   |    5433.15  |     184.1 |        3 |

Rule ratios count signature decisions from the rules-only reference; unknowns abstain and no ML control runs. A dash is the natural sample mix or a tool without a rules mode.

Common unambiguous vocabulary: 138 classes, 11446 files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.

| Tool               | Common-vocabulary accuracy   | Mapped coverage   | Macro recall   |
|--------------------|------------------------------|-------------------|----------------|
| magika1            | 98.22%                       | 99.86%            | 98.12%         |
| magika2-ml         | 98.22%                       | 99.86%            | 98.12%         |
| magika2-rules      | 98.26%                       | 99.87%            | 98.19%         |
| magika2-gpu-ml     | 98.22%                       | 99.86%            | 98.12%         |
| magika2-gpu-rules  | 98.26%                       | 99.87%            | 98.19%         |
| magika2-auto-ml    | 98.22%                       | 99.86%            | 98.12%         |
| magika2-auto-rules | 98.26%                       | 99.87%            | 98.19%         |
| magika2-rules-only | 32.37%                       | 32.37%            | 41.35%         |
| libmagic           | 39.57%                       | 42.11%            | 63.18%         |
| trid               | 64.01%                       | 85.95%            | 75.79%         |

