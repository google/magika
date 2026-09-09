# Cross-tool benchmark 1.2.0

Revision: `9214192eda842d02c8d50d9af75f7d5a043901f3`. Status: complete.

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

| Tool               |   Files |   Requested rule % |   Observed rule % |   Median ms |   Files/s |   Trials |
|--------------------|---------|--------------------|-------------------|-------------|-----------|----------|
| libmagic           |       1 |                  0 |                   |       1.604 |     623.4 |        3 |
| magika1            |       1 |                  0 |                   |      56.992 |      17.5 |        3 |
| magika2-auto-ml    |       1 |                  0 |                   |       6.475 |     154.5 |        3 |
| magika2-auto-rules |       1 |                  0 |                   |       6.719 |     148.8 |        3 |
| magika2-gpu-ml     |       1 |                  0 |                   |      75.864 |      13.2 |        3 |
| magika2-gpu-rules  |       1 |                  0 |                   |      71.163 |      14.1 |        3 |
| magika2-ml         |       1 |                  0 |                   |       8.83  |     113.3 |        3 |
| magika2-rules      |       1 |                  0 |                   |       7.144 |     140   |        3 |
| magika2-rules-only |       1 |                  0 |               0   |       3.292 |     303.8 |        3 |
| trid               |       1 |                  0 |                   |      76.861 |      13   |        3 |
| libmagic           |       1 |                100 |                   |       2.599 |     384.8 |        3 |
| magika1            |       1 |                100 |                   |      55.986 |      17.9 |        3 |
| magika2-auto-ml    |       1 |                100 |                   |       7.884 |     126.8 |        3 |
| magika2-auto-rules |       1 |                100 |                   |       4.685 |     213.5 |        3 |
| magika2-gpu-ml     |       1 |                100 |                   |      76.48  |      13.1 |        3 |
| magika2-gpu-rules  |       1 |                100 |                   |       4.823 |     207.3 |        3 |
| magika2-ml         |       1 |                100 |                   |       7.103 |     140.8 |        3 |
| magika2-rules      |       1 |                100 |                   |       8.039 |     124.4 |        3 |
| magika2-rules-only |       1 |                100 |             100   |       3.66  |     273.2 |        3 |
| trid               |       1 |                100 |                   |      74.307 |      13.5 |        3 |
| libmagic           |       1 |                    |                   |       1.6   |     624.9 |        3 |
| magika1            |       1 |                    |                   |      60.711 |      16.5 |        3 |
| magika2-auto-ml    |       1 |                    |                   |       8.047 |     124.3 |        3 |
| magika2-auto-rules |       1 |                    |                   |       3.684 |     271.4 |        3 |
| magika2-gpu-ml     |       1 |                    |                   |      79.552 |      12.6 |        3 |
| magika2-gpu-rules  |       1 |                    |                   |       4.786 |     208.9 |        3 |
| magika2-ml         |       1 |                    |                   |       6.28  |     159.2 |        3 |
| magika2-rules      |       1 |                    |                   |       3.169 |     315.6 |        3 |
| magika2-rules-only |       1 |                    |             100   |       3.2   |     312.5 |        3 |
| trid               |       1 |                    |                   |      95.115 |      10.5 |        3 |
| libmagic           |       2 |                  0 |                   |       8.341 |     239.8 |        3 |
| magika1            |       2 |                  0 |                   |      55.151 |      36.3 |        3 |
| magika2-auto-ml    |       2 |                  0 |                   |      84.914 |      23.6 |        3 |
| magika2-auto-rules |       2 |                  0 |                   |      77.314 |      25.9 |        3 |
| magika2-gpu-ml     |       2 |                  0 |                   |      94.696 |      21.1 |        3 |
| magika2-gpu-rules  |       2 |                  0 |                   |      86.471 |      23.1 |        3 |
| magika2-ml         |       2 |                  0 |                   |       7.911 |     252.8 |        3 |
| magika2-rules      |       2 |                  0 |                   |       7.725 |     258.9 |        3 |
| magika2-rules-only |       2 |                  0 |               0   |       6.177 |     323.8 |        3 |
| trid               |       2 |                  0 |                   |      81.681 |      24.5 |        3 |
| libmagic           |       2 |                100 |                   |       1.664 |    1201.8 |        3 |
| magika1            |       2 |                100 |                   |      58.242 |      34.3 |        3 |
| magika2-auto-ml    |       2 |                100 |                   |      84.017 |      23.8 |        3 |
| magika2-auto-rules |       2 |                100 |                   |       5.553 |     360.1 |        3 |
| magika2-gpu-ml     |       2 |                100 |                   |      87.998 |      22.7 |        3 |
| magika2-gpu-rules  |       2 |                100 |                   |       5.885 |     339.8 |        3 |
| magika2-ml         |       2 |                100 |                   |      12.487 |     160.2 |        3 |
| magika2-rules      |       2 |                100 |                   |       3.981 |     502.4 |        3 |
| magika2-rules-only |       2 |                100 |             100   |       3.236 |     618.1 |        3 |
| trid               |       2 |                100 |                   |      86.285 |      23.2 |        3 |
| libmagic           |       2 |                 50 |                   |       1.532 |    1305.7 |        3 |
| magika1            |       2 |                 50 |                   |      73.382 |      27.3 |        3 |
| magika2-auto-ml    |       2 |                 50 |                   |      94.744 |      21.1 |        3 |
| magika2-auto-rules |       2 |                 50 |                   |      80.402 |      24.9 |        3 |
| magika2-gpu-ml     |       2 |                 50 |                   |      84.645 |      23.6 |        3 |
| magika2-gpu-rules  |       2 |                 50 |                   |      76.22  |      26.2 |        3 |
| magika2-ml         |       2 |                 50 |                   |       6.821 |     293.2 |        3 |
| magika2-rules      |       2 |                 50 |                   |       7.878 |     253.9 |        3 |
| magika2-rules-only |       2 |                 50 |              50   |       3.506 |     570.5 |        3 |
| trid               |       2 |                 50 |                   |      75.567 |      26.5 |        3 |
| libmagic           |       2 |                    |                   |      10.359 |     193.1 |        3 |
| magika1            |       2 |                    |                   |      70.389 |      28.4 |        3 |
| magika2-auto-ml    |       2 |                    |                   |      87.161 |      22.9 |        3 |
| magika2-auto-rules |       2 |                    |                   |      88.645 |      22.6 |        3 |
| magika2-gpu-ml     |       2 |                    |                   |     103.239 |      19.4 |        3 |
| magika2-gpu-rules  |       2 |                    |                   |      88.691 |      22.6 |        3 |
| magika2-ml         |       2 |                    |                   |       7.268 |     275.2 |        3 |
| magika2-rules      |       2 |                    |                   |      11.727 |     170.6 |        3 |
| magika2-rules-only |       2 |                    |               0   |       3.458 |     578.4 |        3 |
| trid               |       2 |                    |                   |      75.129 |      26.6 |        3 |
| libmagic           |       5 |                  0 |                   |       4.945 |    1011   |        3 |
| magika1            |       5 |                  0 |                   |      69.681 |      71.8 |        3 |
| magika2-auto-ml    |       5 |                  0 |                   |      75.961 |      65.8 |        3 |
| magika2-auto-rules |       5 |                  0 |                   |      79.221 |      63.1 |        3 |
| magika2-gpu-ml     |       5 |                  0 |                   |     102.568 |      48.7 |        3 |
| magika2-gpu-rules  |       5 |                  0 |                   |      87.371 |      57.2 |        3 |
| magika2-ml         |       5 |                  0 |                   |      18.856 |     265.2 |        3 |
| magika2-rules      |       5 |                  0 |                   |      12.951 |     386.1 |        3 |
| magika2-rules-only |       5 |                  0 |               0   |      18.885 |     264.8 |        3 |
| trid               |       5 |                  0 |                   |      82.545 |      60.6 |        3 |
| libmagic           |       5 |                100 |                   |       3.018 |    1656.9 |        3 |
| magika1            |       5 |                100 |                   |      59.254 |      84.4 |        3 |
| magika2-auto-ml    |       5 |                100 |                   |     100.037 |      50   |        3 |
| magika2-auto-rules |       5 |                100 |                   |       5.133 |     974   |        3 |
| magika2-gpu-ml     |       5 |                100 |                   |     103.137 |      48.5 |        3 |
| magika2-gpu-rules  |       5 |                100 |                   |       5.673 |     881.3 |        3 |
| magika2-ml         |       5 |                100 |                   |      12.941 |     386.4 |        3 |
| magika2-rules      |       5 |                100 |                   |       3.825 |    1307.3 |        3 |
| magika2-rules-only |       5 |                100 |             100   |       3.836 |    1303.6 |        3 |
| trid               |       5 |                100 |                   |     102.09  |      49   |        3 |
| libmagic           |       5 |                 20 |                   |       5.375 |     930.3 |        3 |
| magika1            |       5 |                 20 |                   |      99.106 |      50.5 |        3 |
| magika2-auto-ml    |       5 |                 20 |                   |      88.435 |      56.5 |        3 |
| magika2-auto-rules |       5 |                 20 |                   |      95.177 |      52.5 |        3 |
| magika2-gpu-ml     |       5 |                 20 |                   |      88.695 |      56.4 |        3 |
| magika2-gpu-rules  |       5 |                 20 |                   |      81.79  |      61.1 |        3 |
| magika2-ml         |       5 |                 20 |                   |      18.772 |     266.4 |        3 |
| magika2-rules      |       5 |                 20 |                   |      12.8   |     390.6 |        3 |
| magika2-rules-only |       5 |                 20 |              20   |       3.949 |    1266.1 |        3 |
| trid               |       5 |                 20 |                   |      78.852 |      63.4 |        3 |
| libmagic           |       5 |                    |                   |       9.125 |     548   |        3 |
| magika1            |       5 |                    |                   |      59.13  |      84.6 |        3 |
| magika2-auto-ml    |       5 |                    |                   |      86.961 |      57.5 |        3 |
| magika2-auto-rules |       5 |                    |                   |      93.414 |      53.5 |        3 |
| magika2-gpu-ml     |       5 |                    |                   |      80.02  |      62.5 |        3 |
| magika2-gpu-rules  |       5 |                    |                   |      84.183 |      59.4 |        3 |
| magika2-ml         |       5 |                    |                   |      17.462 |     286.3 |        3 |
| magika2-rules      |       5 |                    |                   |      13.413 |     372.8 |        3 |
| magika2-rules-only |       5 |                    |              20   |       6.493 |     770   |        3 |
| trid               |       5 |                    |                   |      75.029 |      66.6 |        3 |
| libmagic           |      10 |                  0 |                   |       4.364 |    2291.5 |        3 |
| magika1            |      10 |                  0 |                   |      62.409 |     160.2 |        3 |
| magika2-auto-ml    |      10 |                  0 |                   |      78.5   |     127.4 |        3 |
| magika2-auto-rules |      10 |                  0 |                   |      88.034 |     113.6 |        3 |
| magika2-gpu-ml     |      10 |                  0 |                   |      79.506 |     125.8 |        3 |
| magika2-gpu-rules  |      10 |                  0 |                   |      81.486 |     122.7 |        3 |
| magika2-ml         |      10 |                  0 |                   |      21.223 |     471.2 |        3 |
| magika2-rules      |      10 |                  0 |                   |      17.692 |     565.2 |        3 |
| magika2-rules-only |      10 |                  0 |               0   |       3.56  |    2809.1 |        3 |
| trid               |      10 |                  0 |                   |      82.978 |     120.5 |        3 |
| libmagic           |      10 |                100 |                   |       4.988 |    2004.8 |        3 |
| magika1            |      10 |                100 |                   |      73.794 |     135.5 |        3 |
| magika2-auto-ml    |      10 |                100 |                   |      88.921 |     112.5 |        3 |
| magika2-auto-rules |      10 |                100 |                   |       9.776 |    1022.9 |        3 |
| magika2-gpu-ml     |      10 |                100 |                   |     108.784 |      91.9 |        3 |
| magika2-gpu-rules  |      10 |                100 |                   |       5.689 |    1757.6 |        3 |
| magika2-ml         |      10 |                100 |                   |      15.159 |     659.7 |        3 |
| magika2-rules      |      10 |                100 |                   |       4.277 |    2338.2 |        3 |
| magika2-rules-only |      10 |                100 |             100   |       5.614 |    1781.4 |        3 |
| trid               |      10 |                100 |                   |      83.187 |     120.2 |        3 |
| libmagic           |      10 |                 20 |                   |       7.714 |    1296.3 |        3 |
| magika1            |      10 |                 20 |                   |      59.002 |     169.5 |        3 |
| magika2-auto-ml    |      10 |                 20 |                   |      85.473 |     117   |        3 |
| magika2-auto-rules |      10 |                 20 |                   |      86.867 |     115.1 |        3 |
| magika2-gpu-ml     |      10 |                 20 |                   |      79.236 |     126.2 |        3 |
| magika2-gpu-rules  |      10 |                 20 |                   |      98.506 |     101.5 |        3 |
| magika2-ml         |      10 |                 20 |                   |      17.256 |     579.5 |        3 |
| magika2-rules      |      10 |                 20 |                   |      12.797 |     781.4 |        3 |
| magika2-rules-only |      10 |                 20 |              20   |       3.964 |    2522.9 |        3 |
| trid               |      10 |                 20 |                   |     103.038 |      97.1 |        3 |
| libmagic           |      10 |                 50 |                   |       3.681 |    2716.6 |        3 |
| magika1            |      10 |                 50 |                   |      61.857 |     161.7 |        3 |
| magika2-auto-ml    |      10 |                 50 |                   |      94.436 |     105.9 |        3 |
| magika2-auto-rules |      10 |                 50 |                   |     114.121 |      87.6 |        3 |
| magika2-gpu-ml     |      10 |                 50 |                   |      86.44  |     115.7 |        3 |
| magika2-gpu-rules  |      10 |                 50 |                   |     113.643 |      88   |        3 |
| magika2-ml         |      10 |                 50 |                   |      17.743 |     563.6 |        3 |
| magika2-rules      |      10 |                 50 |                   |      16.329 |     612.4 |        3 |
| magika2-rules-only |      10 |                 50 |              50   |       4.529 |    2208.1 |        3 |
| trid               |      10 |                 50 |                   |     118.916 |      84.1 |        3 |
| libmagic           |      10 |                    |                   |      16.407 |     609.5 |        3 |
| magika1            |      10 |                    |                   |      60.792 |     164.5 |        3 |
| magika2-auto-ml    |      10 |                    |                   |      80.447 |     124.3 |        3 |
| magika2-auto-rules |      10 |                    |                   |      81.69  |     122.4 |        3 |
| magika2-gpu-ml     |      10 |                    |                   |     103.444 |      96.7 |        3 |
| magika2-gpu-rules  |      10 |                    |                   |      86.094 |     116.2 |        3 |
| magika2-ml         |      10 |                    |                   |      14.612 |     684.4 |        3 |
| magika2-rules      |      10 |                    |                   |      12.838 |     779   |        3 |
| magika2-rules-only |      10 |                    |              20   |       6.108 |    1637.2 |        3 |
| trid               |      10 |                    |                   |      99.482 |     100.5 |        3 |
| libmagic           |      25 |                  0 |                   |      22.363 |    1117.9 |        3 |
| magika1            |      25 |                  0 |                   |      65.616 |     381   |        3 |
| magika2-auto-ml    |      25 |                  0 |                   |      88.393 |     282.8 |        3 |
| magika2-auto-rules |      25 |                  0 |                   |      84.722 |     295.1 |        3 |
| magika2-gpu-ml     |      25 |                  0 |                   |      86.551 |     288.8 |        3 |
| magika2-gpu-rules  |      25 |                  0 |                   |      87.243 |     286.6 |        3 |
| magika2-ml         |      25 |                  0 |                   |      27.114 |     922   |        3 |
| magika2-rules      |      25 |                  0 |                   |      21.304 |    1173.5 |        3 |
| magika2-rules-only |      25 |                  0 |               0   |       4.719 |    5298.2 |        3 |
| trid               |      25 |                  0 |                   |      90.201 |     277.2 |        3 |
| libmagic           |      25 |                100 |                   |       5.369 |    4656.5 |        3 |
| magika1            |      25 |                100 |                   |      64.913 |     385.1 |        3 |
| magika2-auto-ml    |      25 |                100 |                   |     124.209 |     201.3 |        3 |
| magika2-auto-rules |      25 |                100 |                   |       8.267 |    3024.1 |        3 |
| magika2-gpu-ml     |      25 |                100 |                   |      89.044 |     280.8 |        3 |
| magika2-gpu-rules  |      25 |                100 |                   |      11.548 |    2165   |        3 |
| magika2-ml         |      25 |                100 |                   |      21.461 |    1164.9 |        3 |
| magika2-rules      |      25 |                100 |                   |       8.787 |    2845   |        3 |
| magika2-rules-only |      25 |                100 |             100   |       4.876 |    5127   |        3 |
| trid               |      25 |                100 |                   |     129.406 |     193.2 |        3 |
| libmagic           |      25 |                 20 |                   |      24.869 |    1005.3 |        3 |
| magika1            |      25 |                 20 |                   |      68.29  |     366.1 |        3 |
| magika2-auto-ml    |      25 |                 20 |                   |      86.901 |     287.7 |        3 |
| magika2-auto-rules |      25 |                 20 |                   |      89.731 |     278.6 |        3 |
| magika2-gpu-ml     |      25 |                 20 |                   |      88.127 |     283.7 |        3 |
| magika2-gpu-rules  |      25 |                 20 |                   |     113.088 |     221.1 |        3 |
| magika2-ml         |      25 |                 20 |                   |      20.755 |    1204.6 |        3 |
| magika2-rules      |      25 |                 20 |                   |      17.455 |    1432.3 |        3 |
| magika2-rules-only |      25 |                 20 |              20   |       5.046 |    4954.3 |        3 |
| trid               |      25 |                 20 |                   |     135.035 |     185.1 |        3 |
| libmagic           |      25 |                    |                   |       9.994 |    2501.4 |        3 |
| magika1            |      25 |                    |                   |      99.652 |     250.9 |        3 |
| magika2-auto-ml    |      25 |                    |                   |      86.164 |     290.1 |        3 |
| magika2-auto-rules |      25 |                    |                   |      88.747 |     281.7 |        3 |
| magika2-gpu-ml     |      25 |                    |                   |     108.265 |     230.9 |        3 |
| magika2-gpu-rules  |      25 |                    |                   |      89.12  |     280.5 |        3 |
| magika2-ml         |      25 |                    |                   |      20.93  |    1194.4 |        3 |
| magika2-rules      |      25 |                    |                   |      29.675 |     842.4 |        3 |
| magika2-rules-only |      25 |                    |              24   |       4.671 |    5352   |        3 |
| trid               |      25 |                    |                   |     186.171 |     134.3 |        3 |
| libmagic           |     100 |                  0 |                   |      77.098 |    1297.1 |        3 |
| magika1            |     100 |                  0 |                   |     105.82  |     945   |        3 |
| magika2-auto-ml    |     100 |                  0 |                   |      93.824 |    1065.8 |        3 |
| magika2-auto-rules |     100 |                  0 |                   |      96.314 |    1038.3 |        3 |
| magika2-gpu-ml     |     100 |                  0 |                   |      95.925 |    1042.5 |        3 |
| magika2-gpu-rules  |     100 |                  0 |                   |     103.411 |     967   |        3 |
| magika2-ml         |     100 |                  0 |                   |      45.439 |    2200.8 |        3 |
| magika2-rules      |     100 |                  0 |                   |      63.886 |    1565.3 |        3 |
| magika2-rules-only |     100 |                  0 |               0   |       8.659 |   11549   |        3 |
| trid               |     100 |                  0 |                   |     385.047 |     259.7 |        3 |
| libmagic           |     100 |                100 |                   |      13.64  |    7331.5 |        3 |
| magika1            |     100 |                100 |                   |     144.181 |     693.6 |        3 |
| magika2-auto-ml    |     100 |                100 |                   |     104.795 |     954.2 |        3 |
| magika2-auto-rules |     100 |                100 |                   |      12.851 |    7781.7 |        3 |
| magika2-gpu-ml     |     100 |                100 |                   |     119.143 |     839.3 |        3 |
| magika2-gpu-rules  |     100 |                100 |                   |      18.097 |    5525.8 |        3 |
| magika2-ml         |     100 |                100 |                   |      69.318 |    1442.6 |        3 |
| magika2-rules      |     100 |                100 |                   |      19.677 |    5082.2 |        3 |
| magika2-rules-only |     100 |                100 |             100   |       9.191 |   10880.3 |        3 |
| trid               |     100 |                100 |                   |     227.527 |     439.5 |        3 |
| libmagic           |     100 |                 20 |                   |      99.343 |    1006.6 |        3 |
| magika1            |     100 |                 20 |                   |      99.224 |    1007.8 |        3 |
| magika2-auto-ml    |     100 |                 20 |                   |      99.909 |    1000.9 |        3 |
| magika2-auto-rules |     100 |                 20 |                   |      86.838 |    1151.6 |        3 |
| magika2-gpu-ml     |     100 |                 20 |                   |     126.281 |     791.9 |        3 |
| magika2-gpu-rules  |     100 |                 20 |                   |      88.057 |    1135.6 |        3 |
| magika2-ml         |     100 |                 20 |                   |      39.536 |    2529.4 |        3 |
| magika2-rules      |     100 |                 20 |                   |      34.445 |    2903.2 |        3 |
| magika2-rules-only |     100 |                 20 |              20   |       9.699 |   10310.2 |        3 |
| trid               |     100 |                 20 |                   |     368.531 |     271.3 |        3 |
| libmagic           |     100 |                 50 |                   |      56.751 |    1762.1 |        3 |
| magika1            |     100 |                 50 |                   |     183.488 |     545   |        3 |
| magika2-auto-ml    |     100 |                 50 |                   |      92.682 |    1079   |        3 |
| magika2-auto-rules |     100 |                 50 |                   |     110.057 |     908.6 |        3 |
| magika2-gpu-ml     |     100 |                 50 |                   |      93.789 |    1066.2 |        3 |
| magika2-gpu-rules  |     100 |                 50 |                   |     106.51  |     938.9 |        3 |
| magika2-ml         |     100 |                 50 |                   |      42.121 |    2374.1 |        3 |
| magika2-rules      |     100 |                 50 |                   |      27.008 |    3702.6 |        3 |
| magika2-rules-only |     100 |                 50 |              50   |       9.938 |   10062.4 |        3 |
| trid               |     100 |                 50 |                   |     235.976 |     423.8 |        3 |
| libmagic           |     100 |                    |                   |      77.632 |    1288.1 |        3 |
| magika1            |     100 |                    |                   |     146.301 |     683.5 |        3 |
| magika2-auto-ml    |     100 |                    |                   |     116.58  |     857.8 |        3 |
| magika2-auto-rules |     100 |                    |                   |     117.987 |     847.5 |        3 |
| magika2-gpu-ml     |     100 |                    |                   |      96.637 |    1034.8 |        3 |
| magika2-gpu-rules  |     100 |                    |                   |     106.789 |     936.4 |        3 |
| magika2-ml         |     100 |                    |                   |      41.959 |    2383.3 |        3 |
| magika2-rules      |     100 |                    |                   |      36.701 |    2724.7 |        3 |
| magika2-rules-only |     100 |                    |              29   |       8.988 |   11126.3 |        3 |
| trid               |     100 |                    |                   |     197.238 |     507   |        3 |
| libmagic           |    1000 |                  0 |                   |     862.781 |    1159   |        3 |
| magika1            |    1000 |                  0 |                   |     559.608 |    1787   |        3 |
| magika2-auto-ml    |    1000 |                  0 |                   |     200.047 |    4998.8 |        3 |
| magika2-auto-rules |    1000 |                  0 |                   |     185.574 |    5388.7 |        3 |
| magika2-gpu-ml     |    1000 |                  0 |                   |     179.421 |    5573.5 |        3 |
| magika2-gpu-rules  |    1000 |                  0 |                   |     182.728 |    5472.6 |        3 |
| magika2-ml         |    1000 |                  0 |                   |     432.755 |    2310.8 |        3 |
| magika2-rules      |    1000 |                  0 |                   |     447.531 |    2234.5 |        3 |
| magika2-rules-only |    1000 |                  0 |               0   |      68.828 |   14528.9 |        3 |
| trid               |    1000 |                  0 |                   |    1894.52  |     527.8 |        3 |
| libmagic           |    1000 |                 20 |                   |     668.339 |    1496.2 |        3 |
| magika1            |    1000 |                 20 |                   |     716.316 |    1396   |        3 |
| magika2-auto-ml    |    1000 |                 20 |                   |     182.565 |    5477.5 |        3 |
| magika2-auto-rules |    1000 |                 20 |                   |     159.622 |    6264.8 |        3 |
| magika2-gpu-ml     |    1000 |                 20 |                   |     193.246 |    5174.8 |        3 |
| magika2-gpu-rules  |    1000 |                 20 |                   |     162.279 |    6162.2 |        3 |
| magika2-ml         |    1000 |                 20 |                   |     351.411 |    2845.7 |        3 |
| magika2-rules      |    1000 |                 20 |                   |     220.633 |    4532.4 |        3 |
| magika2-rules-only |    1000 |                 20 |              20   |      55.701 |   17952.9 |        3 |
| trid               |    1000 |                 20 |                   |    1542.52  |     648.3 |        3 |
| libmagic           |    1000 |                 50 |                   |     411.052 |    2432.8 |        3 |
| magika1            |    1000 |                 50 |                   |     475.169 |    2104.5 |        3 |
| magika2-auto-ml    |    1000 |                 50 |                   |     191.205 |    5230   |        3 |
| magika2-auto-rules |    1000 |                 50 |                   |     167.571 |    5967.6 |        3 |
| magika2-gpu-ml     |    1000 |                 50 |                   |     229.24  |    4362.2 |        3 |
| magika2-gpu-rules  |    1000 |                 50 |                   |     151.219 |    6612.9 |        3 |
| magika2-ml         |    1000 |                 50 |                   |     316.144 |    3163.1 |        3 |
| magika2-rules      |    1000 |                 50 |                   |     150.573 |    6641.3 |        3 |
| magika2-rules-only |    1000 |                 50 |              50   |      57.327 |   17443.8 |        3 |
| trid               |    1000 |                 50 |                   |    1642.92  |     608.7 |        3 |
| libmagic           |    1000 |                    |                   |     626.604 |    1595.9 |        3 |
| magika1            |    1000 |                    |                   |    1051.82  |     950.7 |        3 |
| magika2-auto-ml    |    1000 |                    |                   |     186.097 |    5373.6 |        3 |
| magika2-auto-rules |    1000 |                    |                   |     172.298 |    5803.9 |        3 |
| magika2-gpu-ml     |    1000 |                    |                   |     210.561 |    4749.2 |        3 |
| magika2-gpu-rules  |    1000 |                    |                   |     178.918 |    5589.1 |        3 |
| magika2-ml         |    1000 |                    |                   |     277.025 |    3609.8 |        3 |
| magika2-rules      |    1000 |                    |                   |     221.603 |    4512.6 |        3 |
| magika2-rules-only |    1000 |                    |              22.9 |      67.931 |   14720.8 |        3 |
| trid               |    1000 |                    |                   |    1685.32  |     593.4 |        3 |

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

