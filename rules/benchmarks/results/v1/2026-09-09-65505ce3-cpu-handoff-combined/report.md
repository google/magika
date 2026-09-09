# Cross-tool benchmark 1.2.1

Revision: `65505ce38363b63d2c07d1370168a95451cf339a`. Status: complete.

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
| libmagic           |       1 |                    |                   |       1.26  |     793.5 |        3 |
| magika1            |       1 |                    |                   |      53.778 |      18.6 |        3 |
| magika2-auto-ml    |       1 |                    |                   |       5.182 |     193   |        3 |
| magika2-auto-rules |       1 |                    |                   |       2.77  |     361   |        3 |
| magika2-gpu-ml     |       1 |                    |                   |       6.129 |     163.2 |        3 |
| magika2-gpu-rules  |       1 |                    |                   |       2.814 |     355.3 |        3 |
| magika2-ml         |       1 |                    |                   |       5.042 |     198.3 |        3 |
| magika2-rules      |       1 |                    |                   |       2.918 |     342.8 |        3 |
| magika2-rules-only |       1 |                    |             100   |       2.672 |     374.2 |        3 |
| trid               |       1 |                    |                   |      70.516 |      14.2 |        3 |
| libmagic           |       2 |                    |                   |       1.528 |    1309.2 |        3 |
| magika1            |       2 |                    |                   |      54.162 |      36.9 |        3 |
| magika2-auto-ml    |       2 |                    |                   |       6.103 |     327.7 |        3 |
| magika2-auto-rules |       2 |                    |                   |       6.273 |     318.8 |        3 |
| magika2-gpu-ml     |       2 |                    |                   |       6.178 |     323.7 |        3 |
| magika2-gpu-rules  |       2 |                    |                   |       6.211 |     322   |        3 |
| magika2-ml         |       2 |                    |                   |       6.221 |     321.5 |        3 |
| magika2-rules      |       2 |                    |                   |       5.968 |     335.1 |        3 |
| magika2-rules-only |       2 |                    |               0   |       2.733 |     731.9 |        3 |
| trid               |       2 |                    |                   |      96.692 |      20.7 |        3 |
| libmagic           |       5 |                    |                   |       3.872 |    1291.3 |        3 |
| magika1            |       5 |                    |                   |      54.534 |      91.7 |        3 |
| magika2-auto-ml    |       5 |                    |                   |      12.407 |     403   |        3 |
| magika2-auto-rules |       5 |                    |                   |      12.29  |     406.8 |        3 |
| magika2-gpu-ml     |       5 |                    |                   |      12.397 |     403.3 |        3 |
| magika2-gpu-rules  |       5 |                    |                   |      12.162 |     411.1 |        3 |
| magika2-ml         |       5 |                    |                   |      11.749 |     425.6 |        3 |
| magika2-rules      |       5 |                    |                   |      11.911 |     419.8 |        3 |
| magika2-rules-only |       5 |                    |              60   |       2.823 |    1771.4 |        3 |
| trid               |       5 |                    |                   |      71.434 |      70   |        3 |
| libmagic           |      10 |                    |                   |       4.934 |    2026.6 |        3 |
| magika1            |      10 |                    |                   |      55.39  |     180.5 |        3 |
| magika2-auto-ml    |      10 |                    |                   |      14.675 |     681.4 |        3 |
| magika2-auto-rules |      10 |                    |                   |      12.576 |     795.1 |        3 |
| magika2-gpu-ml     |      10 |                    |                   |      15.114 |     661.6 |        3 |
| magika2-gpu-rules  |      10 |                    |                   |      12.531 |     798   |        3 |
| magika2-ml         |      10 |                    |                   |      13.141 |     761   |        3 |
| magika2-rules      |      10 |                    |                   |      12.354 |     809.5 |        3 |
| magika2-rules-only |      10 |                    |              40   |       3.153 |    3171.8 |        3 |
| trid               |      10 |                    |                   |      97.9   |     102.1 |        3 |
| libmagic           |      25 |                    |                   |      11.684 |    2139.7 |        3 |
| magika1            |      25 |                    |                   |      59.624 |     419.3 |        3 |
| magika2-auto-ml    |      25 |                    |                   |      20.164 |    1239.8 |        3 |
| magika2-auto-rules |      25 |                    |                   |      17.458 |    1432   |        3 |
| magika2-gpu-ml     |      25 |                    |                   |      20.405 |    1225.2 |        3 |
| magika2-gpu-rules  |      25 |                    |                   |      17.304 |    1444.7 |        3 |
| magika2-ml         |      25 |                    |                   |      19.274 |    1297.1 |        3 |
| magika2-rules      |      25 |                    |                   |      16.069 |    1555.8 |        3 |
| magika2-rules-only |      25 |                    |              32   |       4.104 |    6092.2 |        3 |
| trid               |      25 |                    |                   |      81.54  |     306.6 |        3 |
| libmagic           |     100 |                    |                   |      33.347 |    2998.8 |        3 |
| magika1            |     100 |                    |                   |      93.171 |    1073.3 |        3 |
| magika2-auto-ml    |     100 |                    |                   |      38.461 |    2600   |        3 |
| magika2-auto-rules |     100 |                    |                   |      28.611 |    3495.2 |        3 |
| magika2-gpu-ml     |     100 |                    |                   |      37.757 |    2648.5 |        3 |
| magika2-gpu-rules  |     100 |                    |                   |      29.857 |    3349.3 |        3 |
| magika2-ml         |     100 |                    |                   |      36.18  |    2764   |        3 |
| magika2-rules      |     100 |                    |                   |      28.132 |    3554.6 |        3 |
| magika2-rules-only |     100 |                    |              39   |       9.002 |   11108.2 |        3 |
| trid               |     100 |                    |                   |     272.841 |     366.5 |        3 |
| libmagic           |    1000 |                    |                   |     336.195 |    2974.5 |        3 |
| magika1            |    1000 |                    |                   |     410.84  |    2434   |        3 |
| magika2-auto-ml    |    1000 |                    |                   |     163.865 |    6102.6 |        3 |
| magika2-auto-rules |    1000 |                    |                   |     128.8   |    7764   |        3 |
| magika2-gpu-ml     |    1000 |                    |                   |     169.542 |    5898.3 |        3 |
| magika2-gpu-rules  |    1000 |                    |                   |     122.429 |    8168   |        3 |
| magika2-ml         |    1000 |                    |                   |     256.559 |    3897.7 |        3 |
| magika2-rules      |    1000 |                    |                   |     164.703 |    6071.5 |        3 |
| magika2-rules-only |    1000 |                    |              38.2 |      64.832 |   15424.4 |        3 |
| trid               |    1000 |                    |                   |    4850.05  |     206.2 |        3 |

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

