# Cross-tool benchmark 1.2.1

Revision: `65505ce38363b63d2c07d1370168a95451cf339a`. Status: complete.

Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.

| Tool               |   Files | Accuracy   | Precision   | Mapped coverage   | Macro recall   |   Wrong |   Errors |   Ambiguous |   Unmapped |   Rule hit % | Formats correct   |
|--------------------|---------|------------|-------------|-------------------|----------------|---------|----------|-------------|------------|--------------|-------------------|
| magika1            |    2400 | 58.29%     | 71.27%      | 81.79%            | 56.59%         |     564 |        0 |           0 |          0 |              | 102/167           |
| magika2-ml         |    2400 | 58.29%     | 71.27%      | 81.79%            | 56.59%         |     564 |        0 |           0 |          0 |              | 102/167           |
| magika2-rules      |    2400 | 65.21%     | 76.27%      | 85.50%            | 69.84%         |     487 |        0 |           0 |          0 |              | 125/167           |
| magika2-gpu-ml     |    2400 | 58.29%     | 71.27%      | 81.79%            | 56.59%         |     564 |        0 |           0 |          0 |              | 102/167           |
| magika2-gpu-rules  |    2400 | 65.21%     | 76.27%      | 85.50%            | 69.84%         |     487 |        0 |           0 |          0 |              | 125/167           |
| magika2-auto-ml    |    2400 | 58.29%     | 71.27%      | 81.79%            | 56.59%         |     564 |        0 |           0 |          0 |              | 102/167           |
| magika2-auto-rules |    2400 | 65.21%     | 76.27%      | 85.50%            | 69.84%         |     487 |        0 |           0 |          0 |              | 125/167           |
| magika2-rules-only |    2400 | 22.21%     | 100.00%     | 22.21%            | 28.90%         |       0 |        0 |           0 |          0 |      22.2083 | 53/167            |
| libmagic           |    2400 | 33.29%     | 96.61%      | 34.46%            | 38.85%         |      28 |        0 |         545 |        314 |              | 74/167            |
| trid               |    2400 | 42.58%     | 90.68%      | 46.96%            | 55.41%         |     105 |        0 |         402 |        475 |              | 108/167           |

| Tool               |   Files | Requested rule %   |   Observed rule % |   Median ms |   Files/s |   Trials |
|--------------------|---------|--------------------|-------------------|-------------|-----------|----------|
| libmagic           |       1 |                    |                   |       1.105 |     905   |        3 |
| magika1            |       1 |                    |                   |      52.795 |      18.9 |        3 |
| magika2-auto-ml    |       1 |                    |                   |       4.888 |     204.6 |        3 |
| magika2-auto-rules |       1 |                    |                   |       2.802 |     356.9 |        3 |
| magika2-gpu-ml     |       1 |                    |                   |       6.192 |     161.5 |        3 |
| magika2-gpu-rules  |       1 |                    |                   |       2.693 |     371.4 |        3 |
| magika2-ml         |       1 |                    |                   |       5.434 |     184   |        3 |
| magika2-rules      |       1 |                    |                   |       2.644 |     378.2 |        3 |
| magika2-rules-only |       1 |                    |             100   |       2.474 |     404.1 |        3 |
| trid               |       1 |                    |                   |      66.853 |      15   |        3 |
| libmagic           |       2 |                    |                   |       7.731 |     258.7 |        3 |
| magika1            |       2 |                    |                   |      53.232 |      37.6 |        3 |
| magika2-auto-ml    |       2 |                    |                   |       6.344 |     315.2 |        3 |
| magika2-auto-rules |       2 |                    |                   |       6.233 |     320.9 |        3 |
| magika2-gpu-ml     |       2 |                    |                   |       5.968 |     335.1 |        3 |
| magika2-gpu-rules  |       2 |                    |                   |       6.531 |     306.2 |        3 |
| magika2-ml         |       2 |                    |                   |       5.996 |     333.5 |        3 |
| magika2-rules      |       2 |                    |                   |       5.991 |     333.8 |        3 |
| magika2-rules-only |       2 |                    |               0   |       2.764 |     723.7 |        3 |
| trid               |       2 |                    |                   |      66.332 |      30.2 |        3 |
| libmagic           |       5 |                    |                   |       7.938 |     629.9 |        3 |
| magika1            |       5 |                    |                   |      54.49  |      91.8 |        3 |
| magika2-auto-ml    |       5 |                    |                   |      12.009 |     416.4 |        3 |
| magika2-auto-rules |       5 |                    |                   |      12.169 |     410.9 |        3 |
| magika2-gpu-ml     |       5 |                    |                   |      12.387 |     403.7 |        3 |
| magika2-gpu-rules  |       5 |                    |                   |      12.157 |     411.3 |        3 |
| magika2-ml         |       5 |                    |                   |      12     |     416.7 |        3 |
| magika2-rules      |       5 |                    |                   |      11.694 |     427.6 |        3 |
| magika2-rules-only |       5 |                    |              20   |       2.743 |    1822.8 |        3 |
| trid               |       5 |                    |                   |      68.199 |      73.3 |        3 |
| libmagic           |      10 |                    |                   |      10.984 |     910.4 |        3 |
| magika1            |      10 |                    |                   |      56.128 |     178.2 |        3 |
| magika2-auto-ml    |      10 |                    |                   |      14.117 |     708.4 |        3 |
| magika2-auto-rules |      10 |                    |                   |      12.534 |     797.9 |        3 |
| magika2-gpu-ml     |      10 |                    |                   |      14.363 |     696.2 |        3 |
| magika2-gpu-rules  |      10 |                    |                   |      12.499 |     800   |        3 |
| magika2-ml         |      10 |                    |                   |      12.949 |     772.3 |        3 |
| magika2-rules      |      10 |                    |                   |      11.609 |     861.4 |        3 |
| magika2-rules-only |      10 |                    |              20   |       3.207 |    3117.7 |        3 |
| trid               |      10 |                    |                   |      83.654 |     119.5 |        3 |
| libmagic           |      25 |                    |                   |       8.236 |    3035.4 |        3 |
| magika1            |      25 |                    |                   |      58.448 |     427.7 |        3 |
| magika2-auto-ml    |      25 |                    |                   |      20.237 |    1235.4 |        3 |
| magika2-auto-rules |      25 |                    |                   |      17.779 |    1406.2 |        3 |
| magika2-gpu-ml     |      25 |                    |                   |      20.441 |    1223   |        3 |
| magika2-gpu-rules  |      25 |                    |                   |      16.546 |    1511   |        3 |
| magika2-ml         |      25 |                    |                   |      18.404 |    1358.4 |        3 |
| magika2-rules      |      25 |                    |                   |      15.777 |    1584.6 |        3 |
| magika2-rules-only |      25 |                    |              24   |       4.048 |    6175.8 |        3 |
| trid               |      25 |                    |                   |     173.629 |     144   |        3 |
| libmagic           |     100 |                    |                   |      78.826 |    1268.6 |        3 |
| magika1            |     100 |                    |                   |      90.902 |    1100.1 |        3 |
| magika2-auto-ml    |     100 |                    |                   |      38.332 |    2608.8 |        3 |
| magika2-auto-rules |     100 |                    |                   |      31.231 |    3202   |        3 |
| magika2-gpu-ml     |     100 |                    |                   |      37.849 |    2642.1 |        3 |
| magika2-gpu-rules  |     100 |                    |                   |      31.177 |    3207.5 |        3 |
| magika2-ml         |     100 |                    |                   |      38.052 |    2628   |        3 |
| magika2-rules      |     100 |                    |                   |      31.573 |    3167.2 |        3 |
| magika2-rules-only |     100 |                    |              29   |       7.796 |   12826.5 |        3 |
| trid               |     100 |                    |                   |     181.237 |     551.8 |        3 |
| libmagic           |    1000 |                    |                   |     630.304 |    1586.5 |        3 |
| magika1            |    1000 |                    |                   |     416.058 |    2403.5 |        3 |
| magika2-auto-ml    |    1000 |                    |                   |     178.215 |    5611.2 |        3 |
| magika2-auto-rules |    1000 |                    |                   |     140.406 |    7122.2 |        3 |
| magika2-gpu-ml     |    1000 |                    |                   |     163.176 |    6128.4 |        3 |
| magika2-gpu-rules  |    1000 |                    |                   |     139.442 |    7171.4 |        3 |
| magika2-ml         |    1000 |                    |                   |     252.8   |    3955.7 |        3 |
| magika2-rules      |    1000 |                    |                   |     199.86  |    5003.5 |        3 |
| magika2-rules-only |    1000 |                    |              22.9 |      53.636 |   18644.4 |        3 |
| trid               |    1000 |                    |                   |    1286.28  |     777.4 |        3 |

Rule ratios count signature decisions from the rules-only reference; unknowns abstain and no ML control runs. A dash is the natural sample mix or a tool without a rules mode.

Common unambiguous vocabulary: 138 classes, 1071 files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.

| Tool               | Common-vocabulary accuracy   | Mapped coverage   | Macro recall   |
|--------------------|------------------------------|-------------------|----------------|
| magika1            | 88.70%                       | 96.08%            | 91.96%         |
| magika2-ml         | 88.70%                       | 96.08%            | 91.96%         |
| magika2-rules      | 88.98%                       | 96.17%            | 92.17%         |
| magika2-gpu-ml     | 88.70%                       | 96.08%            | 91.96%         |
| magika2-gpu-rules  | 88.98%                       | 96.17%            | 92.17%         |
| magika2-auto-ml    | 88.70%                       | 96.08%            | 91.96%         |
| magika2-auto-rules | 88.98%                       | 96.17%            | 92.17%         |
| magika2-rules-only | 25.40%                       | 25.40%            | 28.69%         |
| libmagic           | 53.69%                       | 55.09%            | 59.69%         |
| trid               | 61.62%                       | 67.97%            | 66.63%         |

