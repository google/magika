# Cross-tool benchmark 1.1.0

Revision: `e5f7e339ffdf034767b1fed089e90a09b7a83bbb`. Status: complete.

Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.

| Tool               |   Files | Accuracy   | Precision   | Mapped coverage   | Macro recall   |   Wrong |   Errors |   Ambiguous |   Unmapped |   Rule hit % | Formats correct   |
|--------------------|---------|------------|-------------|-------------------|----------------|---------|----------|-------------|------------|--------------|-------------------|
| magika1            |   25421 | 57.73%     | 70.23%      | 82.20%            | 56.82%         |    6220 |        0 |           0 |          0 |              | 95/162            |
| magika2-ml         |   25421 | 57.73%     | 70.23%      | 82.20%            | 56.82%         |    6220 |        0 |           0 |          0 |              | 95/162            |
| magika2-rules      |   25421 | 78.93%     | 84.39%      | 93.52%            | 75.96%         |    3710 |        0 |           0 |          0 |              | 127/162           |
| magika2-gpu-ml     |   25421 | 57.73%     | 70.22%      | 82.21%            | 56.82%         |    6223 |        0 |           0 |          0 |              | 95/162            |
| magika2-gpu-rules  |   25421 | 78.93%     | 84.38%      | 93.53%            | 75.96%         |    3713 |        0 |           0 |          0 |              | 127/162           |
| magika2-rules-only |   25421 | 37.86%     | 100.00%     | 37.86%            | 39.34%         |       0 |        0 |           0 |          0 |      37.8585 | 67/162            |
| libmagic           |   25421 | 25.46%     | 93.24%      | 27.31%            | 35.55%         |     469 |        0 |        8039 |       3801 |              | 63/162            |
| trid               |   25421 | 55.25%     | 78.32%      | 70.55%            | 63.10%         |    3888 |        0 |        4844 |       1402 |              | 119/162           |

| Tool               |   Files |   Requested rule % |   Observed rule % |   Median ms |   Files/s |   Trials |
|--------------------|---------|--------------------|-------------------|-------------|-----------|----------|
| libmagic           |       1 |                  0 |                   |       1.648 |     606.9 |        3 |
| magika1            |       1 |                  0 |                   |      19.248 |      52   |        3 |
| magika2-gpu-ml     |       1 |                  0 |                   |      61.666 |      16.2 |        3 |
| magika2-gpu-rules  |       1 |                  0 |                   |      63.333 |      15.8 |        3 |
| magika2-ml         |       1 |                  0 |                   |       6.101 |     163.9 |        3 |
| magika2-rules      |       1 |                  0 |                   |       6.413 |     155.9 |        3 |
| magika2-rules-only |       1 |                  0 |               0   |       2.943 |     339.7 |        3 |
| trid               |       1 |                  0 |                   |      72.744 |      13.7 |        3 |
| libmagic           |       1 |                100 |                   |       2.029 |     492.9 |        3 |
| magika1            |       1 |                100 |                   |      18.262 |      54.8 |        3 |
| magika2-gpu-ml     |       1 |                100 |                   |      56.183 |      17.8 |        3 |
| magika2-gpu-rules  |       1 |                100 |                   |       4.966 |     201.4 |        3 |
| magika2-ml         |       1 |                100 |                   |       6.094 |     164.1 |        3 |
| magika2-rules      |       1 |                100 |                   |       3.398 |     294.3 |        3 |
| magika2-rules-only |       1 |                100 |             100   |       3.039 |     329.1 |        3 |
| trid               |       1 |                100 |                   |      71.766 |      13.9 |        3 |
| libmagic           |       1 |                    |                   |       1.391 |     718.8 |        3 |
| magika1            |       1 |                    |                   |      18.446 |      54.2 |        3 |
| magika2-gpu-ml     |       1 |                    |                   |      56.874 |      17.6 |        3 |
| magika2-gpu-rules  |       1 |                    |                   |       5.362 |     186.5 |        3 |
| magika2-ml         |       1 |                    |                   |       5.985 |     167.1 |        3 |
| magika2-rules      |       1 |                    |                   |       3.745 |     267   |        3 |
| magika2-rules-only |       1 |                    |             100   |       2.84  |     352.1 |        3 |
| trid               |       1 |                    |                   |      70.051 |      14.3 |        3 |
| libmagic           |       2 |                  0 |                   |       3.944 |     507.1 |        3 |
| magika1            |       2 |                  0 |                   |      19.545 |     102.3 |        3 |
| magika2-gpu-ml     |       2 |                  0 |                   |      67.983 |      29.4 |        3 |
| magika2-gpu-rules  |       2 |                  0 |                   |      70.154 |      28.5 |        3 |
| magika2-ml         |       2 |                  0 |                   |      12.306 |     162.5 |        3 |
| magika2-rules      |       2 |                  0 |                   |      11.899 |     168.1 |        3 |
| magika2-rules-only |       2 |                  0 |               0   |       2.734 |     731.6 |        3 |
| trid               |       2 |                  0 |                   |      96.624 |      20.7 |        3 |
| libmagic           |       2 |                100 |                   |       1.278 |    1565   |        3 |
| magika1            |       2 |                100 |                   |      20.105 |      99.5 |        3 |
| magika2-gpu-ml     |       2 |                100 |                   |      67.333 |      29.7 |        3 |
| magika2-gpu-rules  |       2 |                100 |                   |       4.701 |     425.4 |        3 |
| magika2-ml         |       2 |                100 |                   |      12.113 |     165.1 |        3 |
| magika2-rules      |       2 |                100 |                   |       3.279 |     610   |        3 |
| magika2-rules-only |       2 |                100 |             100   |       2.796 |     715.3 |        3 |
| trid               |       2 |                100 |                   |      72.703 |      27.5 |        3 |
| libmagic           |       2 |                 50 |                   |       1.549 |    1291.2 |        3 |
| magika1            |       2 |                 50 |                   |      19.982 |     100.1 |        3 |
| magika2-gpu-ml     |       2 |                 50 |                   |      69.373 |      28.8 |        3 |
| magika2-gpu-rules  |       2 |                 50 |                   |      67.324 |      29.7 |        3 |
| magika2-ml         |       2 |                 50 |                   |      12.003 |     166.6 |        3 |
| magika2-rules      |       2 |                 50 |                   |      11.892 |     168.2 |        3 |
| magika2-rules-only |       2 |                 50 |              50   |       3.091 |     647   |        3 |
| trid               |       2 |                 50 |                   |      83.3   |      24   |        3 |
| libmagic           |       2 |                    |                   |       1.627 |    1229.1 |        3 |
| magika1            |       2 |                    |                   |      20.673 |      96.7 |        3 |
| magika2-gpu-ml     |       2 |                    |                   |      68.699 |      29.1 |        3 |
| magika2-gpu-rules  |       2 |                    |                   |      68.364 |      29.3 |        3 |
| magika2-ml         |       2 |                    |                   |      12.392 |     161.4 |        3 |
| magika2-rules      |       2 |                    |                   |      12.274 |     163   |        3 |
| magika2-rules-only |       2 |                    |               0   |       3.259 |     613.7 |        3 |
| trid               |       2 |                    |                   |      98.775 |      20.2 |        3 |
| libmagic           |       5 |                  0 |                   |       2.286 |    2187.5 |        3 |
| magika1            |       5 |                  0 |                   |      26.378 |     189.6 |        3 |
| magika2-gpu-ml     |       5 |                  0 |                   |      69.376 |      72.1 |        3 |
| magika2-gpu-rules  |       5 |                  0 |                   |      66.881 |      74.8 |        3 |
| magika2-ml         |       5 |                  0 |                   |      11.955 |     418.2 |        3 |
| magika2-rules      |       5 |                  0 |                   |      12.083 |     413.8 |        3 |
| magika2-rules-only |       5 |                  0 |               0   |       3.19  |    1567.4 |        3 |
| trid               |       5 |                  0 |                   |      75.507 |      66.2 |        3 |
| libmagic           |       5 |                100 |                   |       2.118 |    2360.4 |        3 |
| magika1            |       5 |                100 |                   |      26.472 |     188.9 |        3 |
| magika2-gpu-ml     |       5 |                100 |                   |      74.563 |      67.1 |        3 |
| magika2-gpu-rules  |       5 |                100 |                   |       5.012 |     997.5 |        3 |
| magika2-ml         |       5 |                100 |                   |      11.955 |     418.2 |        3 |
| magika2-rules      |       5 |                100 |                   |       3.761 |    1329.4 |        3 |
| magika2-rules-only |       5 |                100 |             100   |       3.288 |    1520.6 |        3 |
| trid               |       5 |                100 |                   |      78.165 |      64   |        3 |
| libmagic           |       5 |                 20 |                   |       5.406 |     924.9 |        3 |
| magika1            |       5 |                 20 |                   |      25.443 |     196.5 |        3 |
| magika2-gpu-ml     |       5 |                 20 |                   |      73.597 |      67.9 |        3 |
| magika2-gpu-rules  |       5 |                 20 |                   |      70.576 |      70.8 |        3 |
| magika2-ml         |       5 |                 20 |                   |      12.508 |     399.7 |        3 |
| magika2-rules      |       5 |                 20 |                   |      11.96  |     418   |        3 |
| magika2-rules-only |       5 |                 20 |              20   |       3.155 |    1584.7 |        3 |
| trid               |       5 |                 20 |                   |      74.071 |      67.5 |        3 |
| libmagic           |       5 |                    |                   |       3.748 |    1334   |        3 |
| magika1            |       5 |                    |                   |      25.804 |     193.8 |        3 |
| magika2-gpu-ml     |       5 |                    |                   |      75.062 |      66.6 |        3 |
| magika2-gpu-rules  |       5 |                    |                   |      73.483 |      68   |        3 |
| magika2-ml         |       5 |                    |                   |      11.442 |     437   |        3 |
| magika2-rules      |       5 |                    |                   |      11.865 |     421.4 |        3 |
| magika2-rules-only |       5 |                    |              60   |       3.072 |    1627.6 |        3 |
| trid               |       5 |                    |                   |      73.688 |      67.9 |        3 |
| libmagic           |      10 |                  0 |                   |       7.21  |    1386.9 |        3 |
| magika1            |      10 |                  0 |                   |      34.081 |     293.4 |        3 |
| magika2-gpu-ml     |      10 |                  0 |                   |      72.529 |     137.9 |        3 |
| magika2-gpu-rules  |      10 |                  0 |                   |      73.39  |     136.3 |        3 |
| magika2-ml         |      10 |                  0 |                   |      13.82  |     723.6 |        3 |
| magika2-rules      |      10 |                  0 |                   |      13.667 |     731.7 |        3 |
| magika2-rules-only |      10 |                  0 |               0   |       3.055 |    3273   |        3 |
| trid               |      10 |                  0 |                   |     251.056 |      39.8 |        3 |
| libmagic           |      10 |                100 |                   |       2.602 |    3842.9 |        3 |
| magika1            |      10 |                100 |                   |      35.725 |     279.9 |        3 |
| magika2-gpu-ml     |      10 |                100 |                   |      69.123 |     144.7 |        3 |
| magika2-gpu-rules  |      10 |                100 |                   |       5.351 |    1868.9 |        3 |
| magika2-ml         |      10 |                100 |                   |      13.943 |     717.2 |        3 |
| magika2-rules      |      10 |                100 |                   |       3.997 |    2501.6 |        3 |
| magika2-rules-only |      10 |                100 |             100   |       3.059 |    3269.4 |        3 |
| trid               |      10 |                100 |                   |      72.447 |     138   |        3 |
| libmagic           |      10 |                 20 |                   |       7.006 |    1427.3 |        3 |
| magika1            |      10 |                 20 |                   |      35.117 |     284.8 |        3 |
| magika2-gpu-ml     |      10 |                 20 |                   |      70.048 |     142.8 |        3 |
| magika2-gpu-rules  |      10 |                 20 |                   |      71.101 |     140.6 |        3 |
| magika2-ml         |      10 |                 20 |                   |      13.473 |     742.2 |        3 |
| magika2-rules      |      10 |                 20 |                   |      12.013 |     832.4 |        3 |
| magika2-rules-only |      10 |                 20 |              20   |       3.107 |    3218.6 |        3 |
| trid               |      10 |                 20 |                   |      79.413 |     125.9 |        3 |
| libmagic           |      10 |                 50 |                   |       2.721 |    3674.8 |        3 |
| magika1            |      10 |                 50 |                   |      36.091 |     277.1 |        3 |
| magika2-gpu-ml     |      10 |                 50 |                   |      70.967 |     140.9 |        3 |
| magika2-gpu-rules  |      10 |                 50 |                   |      71.593 |     139.7 |        3 |
| magika2-ml         |      10 |                 50 |                   |      13.942 |     717.2 |        3 |
| magika2-rules      |      10 |                 50 |                   |      12.292 |     813.6 |        3 |
| magika2-rules-only |      10 |                 50 |              50   |       3.297 |    3033.1 |        3 |
| trid               |      10 |                 50 |                   |     104.966 |      95.3 |        3 |
| libmagic           |      10 |                    |                   |       5.162 |    1937.1 |        3 |
| magika1            |      10 |                    |                   |      35.105 |     284.9 |        3 |
| magika2-gpu-ml     |      10 |                    |                   |      68.44  |     146.1 |        3 |
| magika2-gpu-rules  |      10 |                    |                   |      73.019 |     136.9 |        3 |
| magika2-ml         |      10 |                    |                   |      13.34  |     749.6 |        3 |
| magika2-rules      |      10 |                    |                   |      11.853 |     843.7 |        3 |
| magika2-rules-only |      10 |                    |              40   |       3.14  |    3185.2 |        3 |
| trid               |      10 |                    |                   |     101.952 |      98.1 |        3 |
| libmagic           |      25 |                  0 |                   |       9.156 |    2730.4 |        3 |
| magika1            |      25 |                  0 |                   |      63.042 |     396.6 |        3 |
| magika2-gpu-ml     |      25 |                  0 |                   |      76.823 |     325.4 |        3 |
| magika2-gpu-rules  |      25 |                  0 |                   |      75.47  |     331.3 |        3 |
| magika2-ml         |      25 |                  0 |                   |      24.745 |    1010.3 |        3 |
| magika2-rules      |      25 |                  0 |                   |      25.239 |     990.5 |        3 |
| magika2-rules-only |      25 |                  0 |               0   |       3.743 |    6679.7 |        3 |
| trid               |      25 |                  0 |                   |     693.616 |      36   |        3 |
| libmagic           |      25 |                100 |                   |       4.944 |    5056.3 |        3 |
| magika1            |      25 |                100 |                   |      61.713 |     405.1 |        3 |
| magika2-gpu-ml     |      25 |                100 |                   |      77.708 |     321.7 |        3 |
| magika2-gpu-rules  |      25 |                100 |                   |       6.349 |    3937.5 |        3 |
| magika2-ml         |      25 |                100 |                   |      25.04  |     998.4 |        3 |
| magika2-rules      |      25 |                100 |                   |       4.623 |    5408   |        3 |
| magika2-rules-only |      25 |                100 |             100   |       4.321 |    5785.9 |        3 |
| trid               |      25 |                100 |                   |      98.981 |     252.6 |        3 |
| libmagic           |      25 |                 20 |                   |       8.301 |    3011.5 |        3 |
| magika1            |      25 |                 20 |                   |      63.158 |     395.8 |        3 |
| magika2-gpu-ml     |      25 |                 20 |                   |      77.278 |     323.5 |        3 |
| magika2-gpu-rules  |      25 |                 20 |                   |      77.101 |     324.3 |        3 |
| magika2-ml         |      25 |                 20 |                   |      25.324 |     987.2 |        3 |
| magika2-rules      |      25 |                 20 |                   |      21.063 |    1186.9 |        3 |
| magika2-rules-only |      25 |                 20 |              20   |       4.176 |    5985.9 |        3 |
| trid               |      25 |                 20 |                   |     209.356 |     119.4 |        3 |
| libmagic           |      25 |                    |                   |      12.098 |    2066.5 |        3 |
| magika1            |      25 |                    |                   |      62.142 |     402.3 |        3 |
| magika2-gpu-ml     |      25 |                    |                   |      75.815 |     329.7 |        3 |
| magika2-gpu-rules  |      25 |                    |                   |      79.567 |     314.2 |        3 |
| magika2-ml         |      25 |                    |                   |      23.858 |    1047.9 |        3 |
| magika2-rules      |      25 |                    |                   |      20.655 |    1210.4 |        3 |
| magika2-rules-only |      25 |                    |              32   |       3.812 |    6558   |        3 |
| trid               |      25 |                    |                   |      83.001 |     301.2 |        3 |
| libmagic           |     100 |                  0 |                   |      41.82  |    2391.2 |        3 |
| magika1            |     100 |                  0 |                   |     201.313 |     496.7 |        3 |
| magika2-gpu-ml     |     100 |                  0 |                   |      86.37  |    1157.8 |        3 |
| magika2-gpu-rules  |     100 |                  0 |                   |      87.986 |    1136.5 |        3 |
| magika2-ml         |     100 |                  0 |                   |      68.049 |    1469.5 |        3 |
| magika2-rules      |     100 |                  0 |                   |      68.719 |    1455.2 |        3 |
| magika2-rules-only |     100 |                  0 |               0   |       5.875 |   17020.2 |        3 |
| trid               |     100 |                  0 |                   |     429.839 |     232.6 |        3 |
| libmagic           |     100 |                100 |                   |      13.529 |    7391.7 |        3 |
| magika1            |     100 |                100 |                   |     202.25  |     494.4 |        3 |
| magika2-gpu-ml     |     100 |                100 |                   |      82.457 |    1212.7 |        3 |
| magika2-gpu-rules  |     100 |                100 |                   |       9.28  |   10776   |        3 |
| magika2-ml         |     100 |                100 |                   |      66.991 |    1492.7 |        3 |
| magika2-rules      |     100 |                100 |                   |       6.73  |   14859.2 |        3 |
| magika2-rules-only |     100 |                100 |             100   |       6.506 |   15369.5 |        3 |
| trid               |     100 |                100 |                   |     594.714 |     168.1 |        3 |
| libmagic           |     100 |                 20 |                   |      32.354 |    3090.8 |        3 |
| magika1            |     100 |                 20 |                   |     200.954 |     497.6 |        3 |
| magika2-gpu-ml     |     100 |                 20 |                   |      84.856 |    1178.5 |        3 |
| magika2-gpu-rules  |     100 |                 20 |                   |      82.876 |    1206.6 |        3 |
| magika2-ml         |     100 |                 20 |                   |      68.265 |    1464.9 |        3 |
| magika2-rules      |     100 |                 20 |                   |      54.416 |    1837.7 |        3 |
| magika2-rules-only |     100 |                 20 |              20   |       6.661 |   15011.8 |        3 |
| trid               |     100 |                 20 |                   |     492.66  |     203   |        3 |
| libmagic           |     100 |                 50 |                   |      21.661 |    4616.5 |        3 |
| magika1            |     100 |                 50 |                   |     200.899 |     497.8 |        3 |
| magika2-gpu-ml     |     100 |                 50 |                   |      84.146 |    1188.4 |        3 |
| magika2-gpu-rules  |     100 |                 50 |                   |      77.951 |    1282.9 |        3 |
| magika2-ml         |     100 |                 50 |                   |      68.43  |    1461.3 |        3 |
| magika2-rules      |     100 |                 50 |                   |      39.693 |    2519.3 |        3 |
| magika2-rules-only |     100 |                 50 |              50   |       6.308 |   15851.7 |        3 |
| trid               |     100 |                 50 |                   |     577.38  |     173.2 |        3 |
| libmagic           |     100 |                    |                   |      32.519 |    3075.2 |        3 |
| magika1            |     100 |                    |                   |     199.784 |     500.5 |        3 |
| magika2-gpu-ml     |     100 |                    |                   |      87.631 |    1141.2 |        3 |
| magika2-gpu-rules  |     100 |                    |                   |      83.797 |    1193.4 |        3 |
| magika2-ml         |     100 |                    |                   |      63.436 |    1576.4 |        3 |
| magika2-rules      |     100 |                    |                   |      43.652 |    2290.9 |        3 |
| magika2-rules-only |     100 |                    |              39   |       6.037 |   16563.4 |        3 |
| trid               |     100 |                    |                   |     284.604 |     351.4 |        3 |
| libmagic           |    1000 |                  0 |                   |     433.822 |    2305.1 |        3 |
| magika1            |    1000 |                  0 |                   |    1854.17  |     539.3 |        3 |
| magika2-gpu-ml     |    1000 |                  0 |                   |     211.199 |    4734.9 |        3 |
| magika2-gpu-rules  |    1000 |                  0 |                   |     200.773 |    4980.8 |        3 |
| magika2-ml         |    1000 |                  0 |                   |     600.417 |    1665.5 |        3 |
| magika2-rules      |    1000 |                  0 |                   |     588.863 |    1698.2 |        3 |
| magika2-rules-only |    1000 |                  0 |               0   |      35.504 |   28165.8 |        3 |
| trid               |    1000 |                  0 |                   |    5610.69  |     178.2 |        3 |
| libmagic           |    1000 |                100 |                   |     118.529 |    8436.7 |        3 |
| magika1            |    1000 |                100 |                   |    1856.16  |     538.7 |        3 |
| magika2-gpu-ml     |    1000 |                100 |                   |     202.762 |    4931.9 |        3 |
| magika2-gpu-rules  |    1000 |                100 |                   |      41.294 |   24216.7 |        3 |
| magika2-ml         |    1000 |                100 |                   |     589.057 |    1697.6 |        3 |
| magika2-rules      |    1000 |                100 |                   |      33.25  |   30075.3 |        3 |
| magika2-rules-only |    1000 |                100 |             100   |      32.591 |   30683.6 |        3 |
| trid               |    1000 |                100 |                   |    3119.2   |     320.6 |        3 |
| libmagic           |    1000 |                 20 |                   |     342.908 |    2916.2 |        3 |
| magika1            |    1000 |                 20 |                   |    1854.2   |     539.3 |        3 |
| magika2-gpu-ml     |    1000 |                 20 |                   |     208.766 |    4790   |        3 |
| magika2-gpu-rules  |    1000 |                 20 |                   |     182.971 |    5465.3 |        3 |
| magika2-ml         |    1000 |                 20 |                   |     588.346 |    1699.7 |        3 |
| magika2-rules      |    1000 |                 20 |                   |     473.755 |    2110.8 |        3 |
| magika2-rules-only |    1000 |                 20 |              20   |      35.518 |   28154.9 |        3 |
| trid               |    1000 |                 20 |                   |    4548.85  |     219.8 |        3 |
| libmagic           |    1000 |                 50 |                   |     273.639 |    3654.5 |        3 |
| magika1            |    1000 |                 50 |                   |    1855.45  |     539   |        3 |
| magika2-gpu-ml     |    1000 |                 50 |                   |     198.024 |    5049.9 |        3 |
| magika2-gpu-rules  |    1000 |                 50 |                   |     136.327 |    7335.3 |        3 |
| magika2-ml         |    1000 |                 50 |                   |     589.123 |    1697.4 |        3 |
| magika2-rules      |    1000 |                 50 |                   |     300.547 |    3327.3 |        3 |
| magika2-rules-only |    1000 |                 50 |              50   |      34.453 |   29025   |        3 |
| trid               |    1000 |                 50 |                   |    3445.15  |     290.3 |        3 |
| libmagic           |    1000 |                    |                   |     323.31  |    3093   |        3 |
| magika1            |    1000 |                    |                   |    1851.52  |     540.1 |        3 |
| magika2-gpu-ml     |    1000 |                    |                   |     203.521 |    4913.5 |        3 |
| magika2-gpu-rules  |    1000 |                    |                   |     159.18  |    6282.2 |        3 |
| magika2-ml         |    1000 |                    |                   |     590.378 |    1693.8 |        3 |
| magika2-rules      |    1000 |                    |                   |     376.885 |    2653.3 |        3 |
| magika2-rules-only |    1000 |                    |              38.2 |      34.923 |   28634.4 |        3 |
| trid               |    1000 |                    |                   |    4846.14  |     206.3 |        3 |

Rule ratios count signature decisions from the rules-only reference; unknowns abstain and no ML control runs. A dash is the natural sample mix or a tool without a rules mode.

Common unambiguous vocabulary: 138 classes, 11446 files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.

| Tool               | Common-vocabulary accuracy   | Mapped coverage   | Macro recall   |
|--------------------|------------------------------|-------------------|----------------|
| magika1            | 98.22%                       | 99.86%            | 98.12%         |
| magika2-ml         | 98.22%                       | 99.86%            | 98.12%         |
| magika2-rules      | 98.26%                       | 99.87%            | 98.19%         |
| magika2-gpu-ml     | 98.22%                       | 99.89%            | 98.12%         |
| magika2-gpu-rules  | 98.26%                       | 99.90%            | 98.19%         |
| magika2-rules-only | 32.37%                       | 32.37%            | 41.35%         |
| libmagic           | 39.57%                       | 42.11%            | 63.18%         |
| trid               | 64.01%                       | 85.95%            | 75.79%         |

