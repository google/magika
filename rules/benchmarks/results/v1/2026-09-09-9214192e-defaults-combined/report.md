# Cross-tool benchmark 1.2.0

Revision: `9214192eda842d02c8d50d9af75f7d5a043901f3`. Status: complete.

Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.

| Tool               |   Files | Accuracy   | Precision   | Mapped coverage   | Macro recall   |   Wrong |   Errors |   Ambiguous |   Unmapped |   Rule hit % | Formats correct   |
|--------------------|---------|------------|-------------|-------------------|----------------|---------|----------|-------------|------------|--------------|-------------------|
| magika1            |   25421 | 57.73%     | 70.23%      | 82.20%            | 56.82%         |    6220 |        0 |           0 |          0 |              | 95/162            |
| magika2-ml         |   25421 | 57.73%     | 70.23%      | 82.20%            | 56.82%         |    6220 |        0 |           0 |          0 |              | 95/162            |
| magika2-rules      |   25421 | 78.93%     | 84.39%      | 93.52%            | 75.96%         |    3710 |        0 |           0 |          0 |              | 127/162           |
| magika2-gpu-ml     |   25421 | 57.73%     | 70.22%      | 82.21%            | 56.82%         |    6223 |        0 |           0 |          0 |              | 95/162            |
| magika2-gpu-rules  |   25421 | 78.93%     | 84.38%      | 93.53%            | 75.96%         |    3713 |        0 |           0 |          0 |              | 127/162           |
| magika2-auto-ml    |   25421 | 57.73%     | 70.22%      | 82.21%            | 56.82%         |    6223 |        0 |           0 |          0 |              | 95/162            |
| magika2-auto-rules |   25421 | 78.93%     | 84.38%      | 93.53%            | 75.96%         |    3713 |        0 |           0 |          0 |              | 127/162           |
| magika2-rules-only |   25421 | 37.86%     | 100.00%     | 37.86%            | 39.34%         |       0 |        0 |           0 |          0 |      37.8585 | 67/162            |
| libmagic           |   25421 | 25.46%     | 93.24%      | 27.31%            | 35.55%         |     469 |        0 |        8039 |       3801 |              | 63/162            |
| trid               |   25421 | 55.25%     | 78.32%      | 70.55%            | 63.10%         |    3888 |        0 |        4844 |       1402 |              | 119/162           |

| Tool               |   Files |   Requested rule % |   Observed rule % |   Median ms |   Files/s |   Trials |
|--------------------|---------|--------------------|-------------------|-------------|-----------|----------|
| libmagic           |       1 |                  0 |                   |       2.029 |     492.7 |        3 |
| magika1            |       1 |                  0 |                   |      58.378 |      17.1 |        3 |
| magika2-auto-ml    |       1 |                  0 |                   |       6.012 |     166.3 |        3 |
| magika2-auto-rules |       1 |                  0 |                   |       5.888 |     169.8 |        3 |
| magika2-gpu-ml     |       1 |                  0 |                   |      80.867 |      12.4 |        3 |
| magika2-gpu-rules  |       1 |                  0 |                   |      67.543 |      14.8 |        3 |
| magika2-ml         |       1 |                  0 |                   |       6.571 |     152.2 |        3 |
| magika2-rules      |       1 |                  0 |                   |       6.479 |     154.4 |        3 |
| magika2-rules-only |       1 |                  0 |               0   |       3.077 |     325   |        3 |
| trid               |       1 |                  0 |                   |      77.877 |      12.8 |        3 |
| libmagic           |       1 |                100 |                   |       2.416 |     413.9 |        3 |
| magika1            |       1 |                100 |                   |      61.709 |      16.2 |        3 |
| magika2-auto-ml    |       1 |                100 |                   |       5.596 |     178.7 |        3 |
| magika2-auto-rules |       1 |                100 |                   |       3.391 |     294.9 |        3 |
| magika2-gpu-ml     |       1 |                100 |                   |      64.174 |      15.6 |        3 |
| magika2-gpu-rules  |       1 |                100 |                   |       4.839 |     206.6 |        3 |
| magika2-ml         |       1 |                100 |                   |       6.846 |     146.1 |        3 |
| magika2-rules      |       1 |                100 |                   |       3.625 |     275.8 |        3 |
| magika2-rules-only |       1 |                100 |             100   |       2.923 |     342.1 |        3 |
| trid               |       1 |                100 |                   |      73.893 |      13.5 |        3 |
| libmagic           |       1 |                    |                   |       1.635 |     611.7 |        3 |
| magika1            |       1 |                    |                   |      57.575 |      17.4 |        3 |
| magika2-auto-ml    |       1 |                    |                   |       5.872 |     170.3 |        3 |
| magika2-auto-rules |       1 |                    |                   |       3.782 |     264.4 |        3 |
| magika2-gpu-ml     |       1 |                    |                   |      84.365 |      11.9 |        3 |
| magika2-gpu-rules  |       1 |                    |                   |       6.07  |     164.8 |        3 |
| magika2-ml         |       1 |                    |                   |       6.79  |     147.3 |        3 |
| magika2-rules      |       1 |                    |                   |       4.518 |     221.3 |        3 |
| magika2-rules-only |       1 |                    |             100   |       3.353 |     298.3 |        3 |
| trid               |       1 |                    |                   |      83.057 |      12   |        3 |
| libmagic           |       2 |                  0 |                   |       5.327 |     375.5 |        3 |
| magika1            |       2 |                  0 |                   |      57.462 |      34.8 |        3 |
| magika2-auto-ml    |       2 |                  0 |                   |      84.901 |      23.6 |        3 |
| magika2-auto-rules |       2 |                  0 |                   |      82.727 |      24.2 |        3 |
| magika2-gpu-ml     |       2 |                  0 |                   |     101.662 |      19.7 |        3 |
| magika2-gpu-rules  |       2 |                  0 |                   |      75.726 |      26.4 |        3 |
| magika2-ml         |       2 |                  0 |                   |       7.566 |     264.3 |        3 |
| magika2-rules      |       2 |                  0 |                   |       7.894 |     253.4 |        3 |
| magika2-rules-only |       2 |                  0 |               0   |       3.734 |     535.7 |        3 |
| trid               |       2 |                  0 |                   |     101.699 |      19.7 |        3 |
| libmagic           |       2 |                100 |                   |       1.789 |    1117.8 |        3 |
| magika1            |       2 |                100 |                   |      60.63  |      33   |        3 |
| magika2-auto-ml    |       2 |                100 |                   |      97.149 |      20.6 |        3 |
| magika2-auto-rules |       2 |                100 |                   |       5.897 |     339.2 |        3 |
| magika2-gpu-ml     |       2 |                100 |                   |      87.891 |      22.8 |        3 |
| magika2-gpu-rules  |       2 |                100 |                   |       5.201 |     384.5 |        3 |
| magika2-ml         |       2 |                100 |                   |       6.735 |     297   |        3 |
| magika2-rules      |       2 |                100 |                   |       4.145 |     482.6 |        3 |
| magika2-rules-only |       2 |                100 |             100   |       3.832 |     521.9 |        3 |
| trid               |       2 |                100 |                   |      79.558 |      25.1 |        3 |
| libmagic           |       2 |                 50 |                   |       1.846 |    1083.5 |        3 |
| magika1            |       2 |                 50 |                   |      62.248 |      32.1 |        3 |
| magika2-auto-ml    |       2 |                 50 |                   |      83.639 |      23.9 |        3 |
| magika2-auto-rules |       2 |                 50 |                   |      78.653 |      25.4 |        3 |
| magika2-gpu-ml     |       2 |                 50 |                   |      74.594 |      26.8 |        3 |
| magika2-gpu-rules  |       2 |                 50 |                   |      93.495 |      21.4 |        3 |
| magika2-ml         |       2 |                 50 |                   |       7.644 |     261.6 |        3 |
| magika2-rules      |       2 |                 50 |                   |       6.388 |     313.1 |        3 |
| magika2-rules-only |       2 |                 50 |              50   |       3.866 |     517.3 |        3 |
| trid               |       2 |                 50 |                   |      91.504 |      21.9 |        3 |
| libmagic           |       2 |                    |                   |       2.051 |     974.9 |        3 |
| magika1            |       2 |                    |                   |      65.674 |      30.5 |        3 |
| magika2-auto-ml    |       2 |                    |                   |      75.854 |      26.4 |        3 |
| magika2-auto-rules |       2 |                    |                   |      83.968 |      23.8 |        3 |
| magika2-gpu-ml     |       2 |                    |                   |      86.613 |      23.1 |        3 |
| magika2-gpu-rules  |       2 |                    |                   |      71.44  |      28   |        3 |
| magika2-ml         |       2 |                    |                   |       7.045 |     283.9 |        3 |
| magika2-rules      |       2 |                    |                   |       7.207 |     277.5 |        3 |
| magika2-rules-only |       2 |                    |               0   |       3.813 |     524.5 |        3 |
| trid               |       2 |                    |                   |     103.997 |      19.2 |        3 |
| libmagic           |       5 |                  0 |                   |       2.859 |    1748.7 |        3 |
| magika1            |       5 |                  0 |                   |      58.124 |      86   |        3 |
| magika2-auto-ml    |       5 |                  0 |                   |      91.105 |      54.9 |        3 |
| magika2-auto-rules |       5 |                  0 |                   |      98.33  |      50.8 |        3 |
| magika2-gpu-ml     |       5 |                  0 |                   |      89.943 |      55.6 |        3 |
| magika2-gpu-rules  |       5 |                  0 |                   |      81.299 |      61.5 |        3 |
| magika2-ml         |       5 |                  0 |                   |      12.798 |     390.7 |        3 |
| magika2-rules      |       5 |                  0 |                   |      12.702 |     393.6 |        3 |
| magika2-rules-only |       5 |                  0 |               0   |       3.797 |    1316.8 |        3 |
| trid               |       5 |                  0 |                   |      91.549 |      54.6 |        3 |
| libmagic           |       5 |                100 |                   |       2.95  |    1694.8 |        3 |
| magika1            |       5 |                100 |                   |      58.855 |      85   |        3 |
| magika2-auto-ml    |       5 |                100 |                   |      89.54  |      55.8 |        3 |
| magika2-auto-rules |       5 |                100 |                   |       4.965 |    1007.1 |        3 |
| magika2-gpu-ml     |       5 |                100 |                   |      87.507 |      57.1 |        3 |
| magika2-gpu-rules  |       5 |                100 |                   |       6.737 |     742.2 |        3 |
| magika2-ml         |       5 |                100 |                   |      14.157 |     353.2 |        3 |
| magika2-rules      |       5 |                100 |                   |       3.845 |    1300.5 |        3 |
| magika2-rules-only |       5 |                100 |             100   |       3.773 |    1325.1 |        3 |
| trid               |       5 |                100 |                   |     100.811 |      49.6 |        3 |
| libmagic           |       5 |                 20 |                   |       6.585 |     759.2 |        3 |
| magika1            |       5 |                 20 |                   |      58.942 |      84.8 |        3 |
| magika2-auto-ml    |       5 |                 20 |                   |      79.797 |      62.7 |        3 |
| magika2-auto-rules |       5 |                 20 |                   |      75.916 |      65.9 |        3 |
| magika2-gpu-ml     |       5 |                 20 |                   |      87.969 |      56.8 |        3 |
| magika2-gpu-rules  |       5 |                 20 |                   |      87.799 |      56.9 |        3 |
| magika2-ml         |       5 |                 20 |                   |      13.585 |     368   |        3 |
| magika2-rules      |       5 |                 20 |                   |      14.562 |     343.4 |        3 |
| magika2-rules-only |       5 |                 20 |              20   |       4.845 |    1031.9 |        3 |
| trid               |       5 |                 20 |                   |      81.477 |      61.4 |        3 |
| libmagic           |       5 |                    |                   |       4.513 |    1107.8 |        3 |
| magika1            |       5 |                    |                   |      61.49  |      81.3 |        3 |
| magika2-auto-ml    |       5 |                    |                   |      81.815 |      61.1 |        3 |
| magika2-auto-rules |       5 |                    |                   |      87.281 |      57.3 |        3 |
| magika2-gpu-ml     |       5 |                    |                   |      88.181 |      56.7 |        3 |
| magika2-gpu-rules  |       5 |                    |                   |      80.569 |      62.1 |        3 |
| magika2-ml         |       5 |                    |                   |      12.686 |     394.1 |        3 |
| magika2-rules      |       5 |                    |                   |      14.419 |     346.8 |        3 |
| magika2-rules-only |       5 |                    |              60   |       3.427 |    1459   |        3 |
| trid               |       5 |                    |                   |      76.641 |      65.2 |        3 |
| libmagic           |      10 |                  0 |                   |       8.746 |    1143.4 |        3 |
| magika1            |      10 |                  0 |                   |      62.178 |     160.8 |        3 |
| magika2-auto-ml    |      10 |                  0 |                   |      85.308 |     117.2 |        3 |
| magika2-auto-rules |      10 |                  0 |                   |      94.586 |     105.7 |        3 |
| magika2-gpu-ml     |      10 |                  0 |                   |      87.758 |     113.9 |        3 |
| magika2-gpu-rules  |      10 |                  0 |                   |      88.469 |     113   |        3 |
| magika2-ml         |      10 |                  0 |                   |      17.444 |     573.3 |        3 |
| magika2-rules      |      10 |                  0 |                   |      16.945 |     590.2 |        3 |
| magika2-rules-only |      10 |                  0 |               0   |       4.012 |    2492.4 |        3 |
| trid               |      10 |                  0 |                   |     260.17  |      38.4 |        3 |
| libmagic           |      10 |                100 |                   |       2.968 |    3369.1 |        3 |
| magika1            |      10 |                100 |                   |      71.052 |     140.7 |        3 |
| magika2-auto-ml    |      10 |                100 |                   |      82.313 |     121.5 |        3 |
| magika2-auto-rules |      10 |                100 |                   |       6.839 |    1462.1 |        3 |
| magika2-gpu-ml     |      10 |                100 |                   |      80.542 |     124.2 |        3 |
| magika2-gpu-rules  |      10 |                100 |                   |       6.108 |    1637.1 |        3 |
| magika2-ml         |      10 |                100 |                   |      15.265 |     655.1 |        3 |
| magika2-rules      |      10 |                100 |                   |       5.884 |    1699.6 |        3 |
| magika2-rules-only |      10 |                100 |             100   |       4.051 |    2468.6 |        3 |
| trid               |      10 |                100 |                   |      78.108 |     128   |        3 |
| libmagic           |      10 |                 20 |                   |       7.361 |    1358.5 |        3 |
| magika1            |      10 |                 20 |                   |      64.264 |     155.6 |        3 |
| magika2-auto-ml    |      10 |                 20 |                   |      85.682 |     116.7 |        3 |
| magika2-auto-rules |      10 |                 20 |                   |      86.037 |     116.2 |        3 |
| magika2-gpu-ml     |      10 |                 20 |                   |      80.517 |     124.2 |        3 |
| magika2-gpu-rules  |      10 |                 20 |                   |      93.046 |     107.5 |        3 |
| magika2-ml         |      10 |                 20 |                   |      16.083 |     621.8 |        3 |
| magika2-rules      |      10 |                 20 |                   |      14.981 |     667.5 |        3 |
| magika2-rules-only |      10 |                 20 |              20   |       4.831 |    2070   |        3 |
| trid               |      10 |                 20 |                   |      85.526 |     116.9 |        3 |
| libmagic           |      10 |                 50 |                   |       3.403 |    2938.7 |        3 |
| magika1            |      10 |                 50 |                   |      62.388 |     160.3 |        3 |
| magika2-auto-ml    |      10 |                 50 |                   |      81.159 |     123.2 |        3 |
| magika2-auto-rules |      10 |                 50 |                   |      80.664 |     124   |        3 |
| magika2-gpu-ml     |      10 |                 50 |                   |      77.311 |     129.3 |        3 |
| magika2-gpu-rules  |      10 |                 50 |                   |      80.828 |     123.7 |        3 |
| magika2-ml         |      10 |                 50 |                   |      15.431 |     648   |        3 |
| magika2-rules      |      10 |                 50 |                   |      15.267 |     655   |        3 |
| magika2-rules-only |      10 |                 50 |              50   |       3.714 |    2692.3 |        3 |
| trid               |      10 |                 50 |                   |     117.4   |      85.2 |        3 |
| libmagic           |      10 |                    |                   |       6.092 |    1641.5 |        3 |
| magika1            |      10 |                    |                   |      63.47  |     157.6 |        3 |
| magika2-auto-ml    |      10 |                    |                   |      87.702 |     114   |        3 |
| magika2-auto-rules |      10 |                    |                   |     110.39  |      90.6 |        3 |
| magika2-gpu-ml     |      10 |                    |                   |      87.713 |     114   |        3 |
| magika2-gpu-rules  |      10 |                    |                   |      88.655 |     112.8 |        3 |
| magika2-ml         |      10 |                    |                   |      15.867 |     630.3 |        3 |
| magika2-rules      |      10 |                    |                   |      14.666 |     681.8 |        3 |
| magika2-rules-only |      10 |                    |              40   |       4.082 |    2450   |        3 |
| trid               |      10 |                    |                   |     109.836 |      91   |        3 |
| libmagic           |      25 |                  0 |                   |       8.769 |    2851.1 |        3 |
| magika1            |      25 |                  0 |                   |      68.074 |     367.2 |        3 |
| magika2-auto-ml    |      25 |                  0 |                   |      97.08  |     257.5 |        3 |
| magika2-auto-rules |      25 |                  0 |                   |      78.387 |     318.9 |        3 |
| magika2-gpu-ml     |      25 |                  0 |                   |      88.235 |     283.3 |        3 |
| magika2-gpu-rules  |      25 |                  0 |                   |      94.006 |     265.9 |        3 |
| magika2-ml         |      25 |                  0 |                   |      20.671 |    1209.4 |        3 |
| magika2-rules      |      25 |                  0 |                   |      21.036 |    1188.5 |        3 |
| magika2-rules-only |      25 |                  0 |               0   |       4.962 |    5038.2 |        3 |
| trid               |      25 |                  0 |                   |     724.584 |      34.5 |        3 |
| libmagic           |      25 |                100 |                   |       6.533 |    3826.9 |        3 |
| magika1            |      25 |                100 |                   |      62.807 |     398   |        3 |
| magika2-auto-ml    |      25 |                100 |                   |      83.178 |     300.6 |        3 |
| magika2-auto-rules |      25 |                100 |                   |       8.104 |    3084.7 |        3 |
| magika2-gpu-ml     |      25 |                100 |                   |      83.614 |     299   |        3 |
| magika2-gpu-rules  |      25 |                100 |                   |       8.775 |    2849   |        3 |
| magika2-ml         |      25 |                100 |                   |      21.989 |    1136.9 |        3 |
| magika2-rules      |      25 |                100 |                   |       7.182 |    3481   |        3 |
| magika2-rules-only |      25 |                100 |             100   |       5.248 |    4763.9 |        3 |
| trid               |      25 |                100 |                   |     103.466 |     241.6 |        3 |
| libmagic           |      25 |                 20 |                   |       9.832 |    2542.8 |        3 |
| magika1            |      25 |                 20 |                   |      63.681 |     392.6 |        3 |
| magika2-auto-ml    |      25 |                 20 |                   |      75.31  |     332   |        3 |
| magika2-auto-rules |      25 |                 20 |                   |      81.55  |     306.6 |        3 |
| magika2-gpu-ml     |      25 |                 20 |                   |      89.02  |     280.8 |        3 |
| magika2-gpu-rules  |      25 |                 20 |                   |      88.403 |     282.8 |        3 |
| magika2-ml         |      25 |                 20 |                   |      21.108 |    1184.4 |        3 |
| magika2-rules      |      25 |                 20 |                   |      18.325 |    1364.3 |        3 |
| magika2-rules-only |      25 |                 20 |              20   |       5.219 |    4790.5 |        3 |
| trid               |      25 |                 20 |                   |     227.605 |     109.8 |        3 |
| libmagic           |      25 |                    |                   |      13.133 |    1903.6 |        3 |
| magika1            |      25 |                    |                   |      72.251 |     346   |        3 |
| magika2-auto-ml    |      25 |                    |                   |      91.196 |     274.1 |        3 |
| magika2-auto-rules |      25 |                    |                   |      78.552 |     318.3 |        3 |
| magika2-gpu-ml     |      25 |                    |                   |      88.323 |     283.1 |        3 |
| magika2-gpu-rules  |      25 |                    |                   |      88.743 |     281.7 |        3 |
| magika2-ml         |      25 |                    |                   |      21.078 |    1186.1 |        3 |
| magika2-rules      |      25 |                    |                   |      18.35  |    1362.4 |        3 |
| magika2-rules-only |      25 |                    |              32   |       5.636 |    4435.4 |        3 |
| trid               |      25 |                    |                   |     100.825 |     248   |        3 |
| libmagic           |     100 |                  0 |                   |      42.268 |    2365.9 |        3 |
| magika1            |     100 |                  0 |                   |     109.474 |     913.5 |        3 |
| magika2-auto-ml    |     100 |                  0 |                   |     128.093 |     780.7 |        3 |
| magika2-auto-rules |     100 |                  0 |                   |      89.57  |    1116.4 |        3 |
| magika2-gpu-ml     |     100 |                  0 |                   |      93.445 |    1070.1 |        3 |
| magika2-gpu-rules  |     100 |                  0 |                   |      96.584 |    1035.4 |        3 |
| magika2-ml         |     100 |                  0 |                   |      44.898 |    2227.3 |        3 |
| magika2-rules      |     100 |                  0 |                   |      44.844 |    2230   |        3 |
| magika2-rules-only |     100 |                  0 |               0   |       9.781 |   10223.6 |        3 |
| trid               |     100 |                  0 |                   |     469.034 |     213.2 |        3 |
| libmagic           |     100 |                100 |                   |      18.893 |    5293   |        3 |
| magika1            |     100 |                100 |                   |     106.52  |     938.8 |        3 |
| magika2-auto-ml    |     100 |                100 |                   |     111.548 |     896.5 |        3 |
| magika2-auto-rules |     100 |                100 |                   |      12.95  |    7722.2 |        3 |
| magika2-gpu-ml     |     100 |                100 |                   |      94.836 |    1054.5 |        3 |
| magika2-gpu-rules  |     100 |                100 |                   |      12.619 |    7924.8 |        3 |
| magika2-ml         |     100 |                100 |                   |      37.507 |    2666.1 |        3 |
| magika2-rules      |     100 |                100 |                   |       9.713 |   10295   |        3 |
| magika2-rules-only |     100 |                100 |             100   |      10.46  |    9560.5 |        3 |
| trid               |     100 |                100 |                   |     625.65  |     159.8 |        3 |
| libmagic           |     100 |                 20 |                   |      36.616 |    2731.1 |        3 |
| magika1            |     100 |                 20 |                   |      99.23  |    1007.8 |        3 |
| magika2-auto-ml    |     100 |                 20 |                   |      86.188 |    1160.3 |        3 |
| magika2-auto-rules |     100 |                 20 |                   |      84.383 |    1185.1 |        3 |
| magika2-gpu-ml     |     100 |                 20 |                   |     113.957 |     877.5 |        3 |
| magika2-gpu-rules  |     100 |                 20 |                   |      94.618 |    1056.9 |        3 |
| magika2-ml         |     100 |                 20 |                   |      39.952 |    2503   |        3 |
| magika2-rules      |     100 |                 20 |                   |      36.061 |    2773.1 |        3 |
| magika2-rules-only |     100 |                 20 |              20   |       9.83  |   10173.2 |        3 |
| trid               |     100 |                 20 |                   |     506.749 |     197.3 |        3 |
| libmagic           |     100 |                 50 |                   |      22.05  |    4535.2 |        3 |
| magika1            |     100 |                 50 |                   |      99.831 |    1001.7 |        3 |
| magika2-auto-ml    |     100 |                 50 |                   |      97.051 |    1030.4 |        3 |
| magika2-auto-rules |     100 |                 50 |                   |      84.673 |    1181   |        3 |
| magika2-gpu-ml     |     100 |                 50 |                   |      89.696 |    1114.9 |        3 |
| magika2-gpu-rules  |     100 |                 50 |                   |      99.548 |    1004.5 |        3 |
| magika2-ml         |     100 |                 50 |                   |      39.655 |    2521.7 |        3 |
| magika2-rules      |     100 |                 50 |                   |      32.491 |    3077.8 |        3 |
| magika2-rules-only |     100 |                 50 |              50   |      11.34  |    8818.1 |        3 |
| trid               |     100 |                 50 |                   |     609.499 |     164.1 |        3 |
| libmagic           |     100 |                    |                   |      35.272 |    2835.1 |        3 |
| magika1            |     100 |                    |                   |     106.241 |     941.3 |        3 |
| magika2-auto-ml    |     100 |                    |                   |      88.458 |    1130.5 |        3 |
| magika2-auto-rules |     100 |                    |                   |      93.306 |    1071.7 |        3 |
| magika2-gpu-ml     |     100 |                    |                   |      89.648 |    1115.5 |        3 |
| magika2-gpu-rules  |     100 |                    |                   |      84.612 |    1181.9 |        3 |
| magika2-ml         |     100 |                    |                   |      44.009 |    2272.3 |        3 |
| magika2-rules      |     100 |                    |                   |      30.717 |    3255.5 |        3 |
| magika2-rules-only |     100 |                    |              39   |       9.332 |   10716.1 |        3 |
| trid               |     100 |                    |                   |     304.641 |     328.3 |        3 |
| libmagic           |    1000 |                  0 |                   |     487.18  |    2052.6 |        3 |
| magika1            |    1000 |                  0 |                   |     523.375 |    1910.7 |        3 |
| magika2-auto-ml    |    1000 |                  0 |                   |     173.622 |    5759.6 |        3 |
| magika2-auto-rules |    1000 |                  0 |                   |     183.266 |    5456.5 |        3 |
| magika2-gpu-ml     |    1000 |                  0 |                   |     160.518 |    6229.8 |        3 |
| magika2-gpu-rules  |    1000 |                  0 |                   |     180.516 |    5539.7 |        3 |
| magika2-ml         |    1000 |                  0 |                   |     269.191 |    3714.8 |        3 |
| magika2-rules      |    1000 |                  0 |                   |     266.671 |    3749.9 |        3 |
| magika2-rules-only |    1000 |                  0 |               0   |      66.706 |   14991.2 |        3 |
| trid               |    1000 |                  0 |                   |    5915.37  |     169.1 |        3 |
| libmagic           |    1000 |                100 |                   |     124.213 |    8050.7 |        3 |
| magika1            |    1000 |                100 |                   |     484.976 |    2062   |        3 |
| magika2-auto-ml    |    1000 |                100 |                   |     187.869 |    5322.9 |        3 |
| magika2-auto-rules |    1000 |                100 |                   |      76.858 |   13011   |        3 |
| magika2-gpu-ml     |    1000 |                100 |                   |     182.881 |    5468   |        3 |
| magika2-gpu-rules  |    1000 |                100 |                   |      73.995 |   13514.5 |        3 |
| magika2-ml         |    1000 |                100 |                   |     265.188 |    3770.9 |        3 |
| magika2-rules      |    1000 |                100 |                   |      69.561 |   14376   |        3 |
| magika2-rules-only |    1000 |                100 |             100   |      75.956 |   13165.6 |        3 |
| trid               |    1000 |                100 |                   |    3394.29  |     294.6 |        3 |
| libmagic           |    1000 |                 20 |                   |     348.597 |    2868.6 |        3 |
| magika1            |    1000 |                 20 |                   |     507.98  |    1968.6 |        3 |
| magika2-auto-ml    |    1000 |                 20 |                   |     183.895 |    5437.9 |        3 |
| magika2-auto-rules |    1000 |                 20 |                   |     154.719 |    6463.3 |        3 |
| magika2-gpu-ml     |    1000 |                 20 |                   |     170.312 |    5871.6 |        3 |
| magika2-gpu-rules  |    1000 |                 20 |                   |     164.367 |    6084   |        3 |
| magika2-ml         |    1000 |                 20 |                   |     266.893 |    3746.8 |        3 |
| magika2-rules      |    1000 |                 20 |                   |     235.697 |    4242.7 |        3 |
| magika2-rules-only |    1000 |                 20 |              20   |      67.552 |   14803.4 |        3 |
| trid               |    1000 |                 20 |                   |    4830.49  |     207   |        3 |
| libmagic           |    1000 |                 50 |                   |     289.639 |    3452.6 |        3 |
| magika1            |    1000 |                 50 |                   |     512.571 |    1950.9 |        3 |
| magika2-auto-ml    |    1000 |                 50 |                   |     182.772 |    5471.3 |        3 |
| magika2-auto-rules |    1000 |                 50 |                   |     127.769 |    7826.6 |        3 |
| magika2-gpu-ml     |    1000 |                 50 |                   |     165.938 |    6026.3 |        3 |
| magika2-gpu-rules  |    1000 |                 50 |                   |     134.5   |    7434.9 |        3 |
| magika2-ml         |    1000 |                 50 |                   |     293.061 |    3412.3 |        3 |
| magika2-rules      |    1000 |                 50 |                   |     173.733 |    5756   |        3 |
| magika2-rules-only |    1000 |                 50 |              50   |      67.076 |   14908.5 |        3 |
| trid               |    1000 |                 50 |                   |    3733.01  |     267.9 |        3 |
| libmagic           |    1000 |                    |                   |     347.321 |    2879.2 |        3 |
| magika1            |    1000 |                    |                   |     493.08  |    2028.1 |        3 |
| magika2-auto-ml    |    1000 |                    |                   |     194.141 |    5150.9 |        3 |
| magika2-auto-rules |    1000 |                    |                   |     135.836 |    7361.8 |        3 |
| magika2-gpu-ml     |    1000 |                    |                   |     186.206 |    5370.4 |        3 |
| magika2-gpu-rules  |    1000 |                    |                   |     134.269 |    7447.7 |        3 |
| magika2-ml         |    1000 |                    |                   |     265.384 |    3768.1 |        3 |
| magika2-rules      |    1000 |                    |                   |     178.838 |    5591.6 |        3 |
| magika2-rules-only |    1000 |                    |              38.2 |      67.087 |   14906.1 |        3 |
| trid               |    1000 |                    |                   |    5156.85  |     193.9 |        3 |

Rule ratios count signature decisions from the rules-only reference; unknowns abstain and no ML control runs. A dash is the natural sample mix or a tool without a rules mode.

Common unambiguous vocabulary: 138 classes, 11446 files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.

| Tool               | Common-vocabulary accuracy   | Mapped coverage   | Macro recall   |
|--------------------|------------------------------|-------------------|----------------|
| magika1            | 98.22%                       | 99.86%            | 98.12%         |
| magika2-ml         | 98.22%                       | 99.86%            | 98.12%         |
| magika2-rules      | 98.26%                       | 99.87%            | 98.19%         |
| magika2-gpu-ml     | 98.22%                       | 99.89%            | 98.12%         |
| magika2-gpu-rules  | 98.26%                       | 99.90%            | 98.19%         |
| magika2-auto-ml    | 98.22%                       | 99.89%            | 98.12%         |
| magika2-auto-rules | 98.26%                       | 99.90%            | 98.19%         |
| magika2-rules-only | 32.37%                       | 32.37%            | 41.35%         |
| libmagic           | 39.57%                       | 42.11%            | 63.18%         |
| trid               | 64.01%                       | 85.95%            | 75.79%         |

