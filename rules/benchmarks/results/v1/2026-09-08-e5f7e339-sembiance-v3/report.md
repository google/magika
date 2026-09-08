# Cross-tool benchmark 1.1.0

Revision: `e5f7e339ffdf034767b1fed089e90a09b7a83bbb`. Status: complete.

Whole-process wall time, warm OS/tool caches, shell disabled. One file includes startup; large workloads amortize it. No isolated model-initialization or cold-disk claim.

| Tool               |   Files | Accuracy   | Precision   | Mapped coverage   | Macro recall   |   Wrong |   Errors |   Ambiguous |   Unmapped |   Rule hit % | Formats correct   |
|--------------------|---------|------------|-------------|-------------------|----------------|---------|----------|-------------|------------|--------------|-------------------|
| magika1            |    2400 | 58.29%     | 71.27%      | 81.79%            | 56.59%         |     564 |        0 |           0 |          0 |              | 102/167           |
| magika2-ml         |    2400 | 58.29%     | 71.27%      | 81.79%            | 56.59%         |     564 |        0 |           0 |          0 |              | 102/167           |
| magika2-rules      |    2400 | 65.21%     | 76.27%      | 85.50%            | 69.84%         |     487 |        0 |           0 |          0 |              | 125/167           |
| magika2-gpu-ml     |    2400 | 58.29%     | 71.27%      | 81.79%            | 56.59%         |     564 |        0 |           0 |          0 |              | 102/167           |
| magika2-gpu-rules  |    2400 | 65.21%     | 76.27%      | 85.50%            | 69.84%         |     487 |        0 |           0 |          0 |              | 125/167           |
| magika2-rules-only |    2400 | 22.21%     | 100.00%     | 22.21%            | 28.90%         |       0 |        0 |           0 |          0 |      22.2083 | 53/167            |
| libmagic           |    2400 | 33.29%     | 96.61%      | 34.46%            | 38.85%         |      28 |        0 |         545 |        314 |              | 74/167            |
| trid               |    2400 | 42.58%     | 90.68%      | 46.96%            | 55.41%         |     105 |        0 |         402 |        475 |              | 108/167           |

| Tool               |   Files |   Requested rule % |   Observed rule % |   Median ms |   Files/s |   Trials |
|--------------------|---------|--------------------|-------------------|-------------|-----------|----------|
| libmagic           |       1 |                  0 |                   |       3.075 |     325.3 |        3 |
| magika1            |       1 |                  0 |                   |      28.626 |      34.9 |        3 |
| magika2-gpu-ml     |       1 |                  0 |                   |      86.587 |      11.5 |        3 |
| magika2-gpu-rules  |       1 |                  0 |                   |      71.767 |      13.9 |        3 |
| magika2-ml         |       1 |                  0 |                   |       8.213 |     121.8 |        3 |
| magika2-rules      |       1 |                  0 |                   |       8.391 |     119.2 |        3 |
| magika2-rules-only |       1 |                  0 |               0   |       4.22  |     236.9 |        3 |
| trid               |       1 |                  0 |                   |      75.955 |      13.2 |        3 |
| libmagic           |       1 |                100 |                   |       2.604 |     384.1 |        3 |
| magika1            |       1 |                100 |                   |      28.233 |      35.4 |        3 |
| magika2-gpu-ml     |       1 |                100 |                   |      89.125 |      11.2 |        3 |
| magika2-gpu-rules  |       1 |                100 |                   |       7.321 |     136.6 |        3 |
| magika2-ml         |       1 |                100 |                   |       7.958 |     125.7 |        3 |
| magika2-rules      |       1 |                100 |                   |       5.118 |     195.4 |        3 |
| magika2-rules-only |       1 |                100 |             100   |       3.774 |     265   |        3 |
| trid               |       1 |                100 |                   |     115.702 |       8.6 |        3 |
| libmagic           |       1 |                    |                   |       2.234 |     447.6 |        3 |
| magika1            |       1 |                    |                   |      23.245 |      43   |        3 |
| magika2-gpu-ml     |       1 |                    |                   |      79.607 |      12.6 |        3 |
| magika2-gpu-rules  |       1 |                    |                   |       8.485 |     117.9 |        3 |
| magika2-ml         |       1 |                    |                   |       8.218 |     121.7 |        3 |
| magika2-rules      |       1 |                    |                   |       4.547 |     219.9 |        3 |
| magika2-rules-only |       1 |                    |             100   |       3.941 |     253.7 |        3 |
| trid               |       1 |                    |                   |      88.239 |      11.3 |        3 |
| libmagic           |       2 |                  0 |                   |      11.748 |     170.2 |        3 |
| magika1            |       2 |                  0 |                   |      25.64  |      78   |        3 |
| magika2-gpu-ml     |       2 |                  0 |                   |     109.5   |      18.3 |        3 |
| magika2-gpu-rules  |       2 |                  0 |                   |      78.882 |      25.4 |        3 |
| magika2-ml         |       2 |                  0 |                   |      17.229 |     116.1 |        3 |
| magika2-rules      |       2 |                  0 |                   |      14.085 |     142   |        3 |
| magika2-rules-only |       2 |                  0 |               0   |       3.232 |     618.8 |        3 |
| trid               |       2 |                  0 |                   |     113.514 |      17.6 |        3 |
| libmagic           |       2 |                100 |                   |       2.109 |     948.1 |        3 |
| magika1            |       2 |                100 |                   |      25.21  |      79.3 |        3 |
| magika2-gpu-ml     |       2 |                100 |                   |     104.93  |      19.1 |        3 |
| magika2-gpu-rules  |       2 |                100 |                   |       7.527 |     265.7 |        3 |
| magika2-ml         |       2 |                100 |                   |      14.072 |     142.1 |        3 |
| magika2-rules      |       2 |                100 |                   |       5.089 |     393   |        3 |
| magika2-rules-only |       2 |                100 |             100   |       4.288 |     466.4 |        3 |
| trid               |       2 |                100 |                   |      94.072 |      21.3 |        3 |
| libmagic           |       2 |                 50 |                   |       3.104 |     644.3 |        3 |
| magika1            |       2 |                 50 |                   |      22.307 |      89.7 |        3 |
| magika2-gpu-ml     |       2 |                 50 |                   |      75.206 |      26.6 |        3 |
| magika2-gpu-rules  |       2 |                 50 |                   |     110.365 |      18.1 |        3 |
| magika2-ml         |       2 |                 50 |                   |      14.377 |     139.1 |        3 |
| magika2-rules      |       2 |                 50 |                   |      39.228 |      51   |        3 |
| magika2-rules-only |       2 |                 50 |              50   |       4.757 |     420.4 |        3 |
| trid               |       2 |                 50 |                   |      90.407 |      22.1 |        3 |
| libmagic           |       2 |                    |                   |      10.014 |     199.7 |        3 |
| magika1            |       2 |                    |                   |      22.625 |      88.4 |        3 |
| magika2-gpu-ml     |       2 |                    |                   |      90.441 |      22.1 |        3 |
| magika2-gpu-rules  |       2 |                    |                   |      88.194 |      22.7 |        3 |
| magika2-ml         |       2 |                    |                   |      15.541 |     128.7 |        3 |
| magika2-rules      |       2 |                    |                   |      14.275 |     140.1 |        3 |
| magika2-rules-only |       2 |                    |               0   |       3.428 |     583.4 |        3 |
| trid               |       2 |                    |                   |      76.106 |      26.3 |        3 |
| libmagic           |       5 |                  0 |                   |       6.381 |     783.5 |        3 |
| magika1            |       5 |                  0 |                   |      32.364 |     154.5 |        3 |
| magika2-gpu-ml     |       5 |                  0 |                   |     106.568 |      46.9 |        3 |
| magika2-gpu-rules  |       5 |                  0 |                   |      89.164 |      56.1 |        3 |
| magika2-ml         |       5 |                  0 |                   |      13.064 |     382.7 |        3 |
| magika2-rules      |       5 |                  0 |                   |      13.795 |     362.5 |        3 |
| magika2-rules-only |       5 |                  0 |               0   |       4.262 |    1173   |        3 |
| trid               |       5 |                  0 |                   |      98.302 |      50.9 |        3 |
| libmagic           |       5 |                100 |                   |       3.606 |    1386.4 |        3 |
| magika1            |       5 |                100 |                   |      34.043 |     146.9 |        3 |
| magika2-gpu-ml     |       5 |                100 |                   |     105.291 |      47.5 |        3 |
| magika2-gpu-rules  |       5 |                100 |                   |       7.623 |     655.9 |        3 |
| magika2-ml         |       5 |                100 |                   |      14.769 |     338.5 |        3 |
| magika2-rules      |       5 |                100 |                   |       4.214 |    1186.5 |        3 |
| magika2-rules-only |       5 |                100 |             100   |       5.059 |     988.2 |        3 |
| trid               |       5 |                100 |                   |      87.927 |      56.9 |        3 |
| libmagic           |       5 |                 20 |                   |       5.589 |     894.6 |        3 |
| magika1            |       5 |                 20 |                   |      30.867 |     162   |        3 |
| magika2-gpu-ml     |       5 |                 20 |                   |      88.836 |      56.3 |        3 |
| magika2-gpu-rules  |       5 |                 20 |                   |      89.218 |      56   |        3 |
| magika2-ml         |       5 |                 20 |                   |      13.912 |     359.4 |        3 |
| magika2-rules      |       5 |                 20 |                   |      14.64  |     341.5 |        3 |
| magika2-rules-only |       5 |                 20 |              20   |       3.97  |    1259.5 |        3 |
| trid               |       5 |                 20 |                   |      80.456 |      62.1 |        3 |
| libmagic           |       5 |                    |                   |      10.92  |     457.9 |        3 |
| magika1            |       5 |                    |                   |      31.337 |     159.6 |        3 |
| magika2-gpu-ml     |       5 |                    |                   |      87.544 |      57.1 |        3 |
| magika2-gpu-rules  |       5 |                    |                   |     106.429 |      47   |        3 |
| magika2-ml         |       5 |                    |                   |      13.84  |     361.3 |        3 |
| magika2-rules      |       5 |                    |                   |      39.383 |     127   |        3 |
| magika2-rules-only |       5 |                    |              20   |       4.515 |    1107.4 |        3 |
| trid               |       5 |                    |                   |      76.689 |      65.2 |        3 |
| libmagic           |      10 |                  0 |                   |       4.901 |    2040.5 |        3 |
| magika1            |      10 |                  0 |                   |      41.898 |     238.7 |        3 |
| magika2-gpu-ml     |      10 |                  0 |                   |     104.285 |      95.9 |        3 |
| magika2-gpu-rules  |      10 |                  0 |                   |      91.948 |     108.8 |        3 |
| magika2-ml         |      10 |                  0 |                   |      39.581 |     252.6 |        3 |
| magika2-rules      |      10 |                  0 |                   |      16.98  |     588.9 |        3 |
| magika2-rules-only |      10 |                  0 |               0   |       4.684 |    2134.9 |        3 |
| trid               |      10 |                  0 |                   |      85.272 |     117.3 |        3 |
| libmagic           |      10 |                100 |                   |       5.417 |    1846.1 |        3 |
| magika1            |      10 |                100 |                   |      38.34  |     260.8 |        3 |
| magika2-gpu-ml     |      10 |                100 |                   |      89.87  |     111.3 |        3 |
| magika2-gpu-rules  |      10 |                100 |                   |       7.939 |    1259.5 |        3 |
| magika2-ml         |      10 |                100 |                   |      17.242 |     580   |        3 |
| magika2-rules      |      10 |                100 |                   |       6.805 |    1469.6 |        3 |
| magika2-rules-only |      10 |                100 |             100   |       5.474 |    1826.7 |        3 |
| trid               |      10 |                100 |                   |      95.847 |     104.3 |        3 |
| libmagic           |      10 |                 20 |                   |       9.864 |    1013.8 |        3 |
| magika1            |      10 |                 20 |                   |      43.004 |     232.5 |        3 |
| magika2-gpu-ml     |      10 |                 20 |                   |      93.063 |     107.5 |        3 |
| magika2-gpu-rules  |      10 |                 20 |                   |      80.124 |     124.8 |        3 |
| magika2-ml         |      10 |                 20 |                   |      17.353 |     576.3 |        3 |
| magika2-rules      |      10 |                 20 |                   |      15.371 |     650.6 |        3 |
| magika2-rules-only |      10 |                 20 |              20   |       4.156 |    2406.4 |        3 |
| trid               |      10 |                 20 |                   |     105.047 |      95.2 |        3 |
| libmagic           |      10 |                 50 |                   |       4.063 |    2461.1 |        3 |
| magika1            |      10 |                 50 |                   |      37.857 |     264.2 |        3 |
| magika2-gpu-ml     |      10 |                 50 |                   |      81.07  |     123.4 |        3 |
| magika2-gpu-rules  |      10 |                 50 |                   |      87.548 |     114.2 |        3 |
| magika2-ml         |      10 |                 50 |                   |      16.412 |     609.3 |        3 |
| magika2-rules      |      10 |                 50 |                   |      14.769 |     677.1 |        3 |
| magika2-rules-only |      10 |                 50 |              50   |       4.379 |    2283.8 |        3 |
| trid               |      10 |                 50 |                   |     131.703 |      75.9 |        3 |
| libmagic           |      10 |                    |                   |      12.253 |     816.1 |        3 |
| magika1            |      10 |                    |                   |      41.485 |     241.1 |        3 |
| magika2-gpu-ml     |      10 |                    |                   |      91.168 |     109.7 |        3 |
| magika2-gpu-rules  |      10 |                    |                   |      99.911 |     100.1 |        3 |
| magika2-ml         |      10 |                    |                   |      16.184 |     617.9 |        3 |
| magika2-rules      |      10 |                    |                   |      14.992 |     667   |        3 |
| magika2-rules-only |      10 |                    |              20   |       4.109 |    2433.5 |        3 |
| trid               |      10 |                    |                   |      92.982 |     107.5 |        3 |
| libmagic           |      25 |                  0 |                   |      25.82  |     968.2 |        3 |
| magika1            |      25 |                  0 |                   |      66.734 |     374.6 |        3 |
| magika2-gpu-ml     |      25 |                  0 |                   |     110.681 |     225.9 |        3 |
| magika2-gpu-rules  |      25 |                  0 |                   |      94.873 |     263.5 |        3 |
| magika2-ml         |      25 |                  0 |                   |      29.41  |     850   |        3 |
| magika2-rules      |      25 |                  0 |                   |      36.625 |     682.6 |        3 |
| magika2-rules-only |      25 |                  0 |               0   |       5.415 |    4616.9 |        3 |
| trid               |      25 |                  0 |                   |     101.547 |     246.2 |        3 |
| libmagic           |      25 |                100 |                   |       7.506 |    3330.7 |        3 |
| magika1            |      25 |                100 |                   |      77.958 |     320.7 |        3 |
| magika2-gpu-ml     |      25 |                100 |                   |      96.524 |     259   |        3 |
| magika2-gpu-rules  |      25 |                100 |                   |       8.176 |    3057.8 |        3 |
| magika2-ml         |      25 |                100 |                   |      28.797 |     868.1 |        3 |
| magika2-rules      |      25 |                100 |                   |       5.834 |    4285.4 |        3 |
| magika2-rules-only |      25 |                100 |             100   |       5.056 |    4944.5 |        3 |
| trid               |      25 |                100 |                   |     109.946 |     227.4 |        3 |
| libmagic           |      25 |                 20 |                   |      26.973 |     926.8 |        3 |
| magika1            |      25 |                 20 |                   |      71.445 |     349.9 |        3 |
| magika2-gpu-ml     |      25 |                 20 |                   |     117.805 |     212.2 |        3 |
| magika2-gpu-rules  |      25 |                 20 |                   |     104.644 |     238.9 |        3 |
| magika2-ml         |      25 |                 20 |                   |      25.462 |     981.9 |        3 |
| magika2-rules      |      25 |                 20 |                   |      26.345 |     949   |        3 |
| magika2-rules-only |      25 |                 20 |              20   |       5.007 |    4992.8 |        3 |
| trid               |      25 |                 20 |                   |     115.192 |     217   |        3 |
| libmagic           |      25 |                    |                   |       9.955 |    2511.2 |        3 |
| magika1            |      25 |                    |                   |     105.926 |     236   |        3 |
| magika2-gpu-ml     |      25 |                    |                   |     111.521 |     224.2 |        3 |
| magika2-gpu-rules  |      25 |                    |                   |     104.448 |     239.4 |        3 |
| magika2-ml         |      25 |                    |                   |      28.848 |     866.6 |        3 |
| magika2-rules      |      25 |                    |                   |      21.457 |    1165.1 |        3 |
| magika2-rules-only |      25 |                    |              24   |       4.208 |    5940.7 |        3 |
| trid               |      25 |                    |                   |     200.606 |     124.6 |        3 |
| libmagic           |     100 |                  0 |                   |      93.458 |    1070   |        3 |
| magika1            |     100 |                  0 |                   |     241.496 |     414.1 |        3 |
| magika2-gpu-ml     |     100 |                  0 |                   |      96.031 |    1041.3 |        3 |
| magika2-gpu-rules  |     100 |                  0 |                   |     102.506 |     975.6 |        3 |
| magika2-ml         |     100 |                  0 |                   |      69.984 |    1428.9 |        3 |
| magika2-rules      |     100 |                  0 |                   |      83.747 |    1194.1 |        3 |
| magika2-rules-only |     100 |                  0 |               0   |       7.815 |   12796.2 |        3 |
| trid               |     100 |                  0 |                   |     342.809 |     291.7 |        3 |
| libmagic           |     100 |                100 |                   |      16.482 |    6067.3 |        3 |
| magika1            |     100 |                100 |                   |     213.955 |     467.4 |        3 |
| magika2-gpu-ml     |     100 |                100 |                   |     113.397 |     881.9 |        3 |
| magika2-gpu-rules  |     100 |                100 |                   |      12.139 |    8237.9 |        3 |
| magika2-ml         |     100 |                100 |                   |      73.152 |    1367   |        3 |
| magika2-rules      |     100 |                100 |                   |       8.212 |   12176.7 |        3 |
| magika2-rules-only |     100 |                100 |             100   |       9.031 |   11073   |        3 |
| trid               |     100 |                100 |                   |     181.309 |     551.5 |        3 |
| libmagic           |     100 |                 20 |                   |      88.088 |    1135.2 |        3 |
| magika1            |     100 |                 20 |                   |     280.151 |     357   |        3 |
| magika2-gpu-ml     |     100 |                 20 |                   |     103.734 |     964   |        3 |
| magika2-gpu-rules  |     100 |                 20 |                   |     106.074 |     942.7 |        3 |
| magika2-ml         |     100 |                 20 |                   |      69.481 |    1439.2 |        3 |
| magika2-rules      |     100 |                 20 |                   |      55.144 |    1813.4 |        3 |
| magika2-rules-only |     100 |                 20 |              20   |      11.074 |    9030.1 |        3 |
| trid               |     100 |                 20 |                   |     300.988 |     332.2 |        3 |
| libmagic           |     100 |                 50 |                   |      64.82  |    1542.7 |        3 |
| magika1            |     100 |                 50 |                   |     211.825 |     472.1 |        3 |
| magika2-gpu-ml     |     100 |                 50 |                   |     102.731 |     973.4 |        3 |
| magika2-gpu-rules  |     100 |                 50 |                   |     116.562 |     857.9 |        3 |
| magika2-ml         |     100 |                 50 |                   |     100.059 |     999.4 |        3 |
| magika2-rules      |     100 |                 50 |                   |      40.677 |    2458.4 |        3 |
| magika2-rules-only |     100 |                 50 |              50   |      10.451 |    9568.9 |        3 |
| trid               |     100 |                 50 |                   |     262.041 |     381.6 |        3 |
| libmagic           |     100 |                    |                   |      78.628 |    1271.8 |        3 |
| magika1            |     100 |                    |                   |     235.209 |     425.2 |        3 |
| magika2-gpu-ml     |     100 |                    |                   |     124.004 |     806.4 |        3 |
| magika2-gpu-rules  |     100 |                    |                   |     116.486 |     858.5 |        3 |
| magika2-ml         |     100 |                    |                   |      68.775 |    1454   |        3 |
| magika2-rules      |     100 |                    |                   |      51.314 |    1948.8 |        3 |
| magika2-rules-only |     100 |                    |              29   |       7.095 |   14094.2 |        3 |
| trid               |     100 |                    |                   |     220.485 |     453.5 |        3 |
| libmagic           |    1000 |                  0 |                   |     821.107 |    1217.9 |        3 |
| magika1            |    1000 |                  0 |                   |    2453.63  |     407.6 |        3 |
| magika2-gpu-ml     |    1000 |                  0 |                   |     249.439 |    4009   |        3 |
| magika2-gpu-rules  |    1000 |                  0 |                   |     251.36  |    3978.4 |        3 |
| magika2-ml         |    1000 |                  0 |                   |    1824.99  |     547.9 |        3 |
| magika2-rules      |    1000 |                  0 |                   |     630.383 |    1586.3 |        3 |
| magika2-rules-only |    1000 |                  0 |               0   |      51.938 |   19253.6 |        3 |
| trid               |    1000 |                  0 |                   |    1776.31  |     563   |        3 |
| libmagic           |    1000 |                 20 |                   |     709.432 |    1409.6 |        3 |
| magika1            |    1000 |                 20 |                   |    2137.12  |     467.9 |        3 |
| magika2-gpu-ml     |    1000 |                 20 |                   |     236.166 |    4234.3 |        3 |
| magika2-gpu-rules  |    1000 |                 20 |                   |     239.362 |    4177.8 |        3 |
| magika2-ml         |    1000 |                 20 |                   |     628.922 |    1590   |        3 |
| magika2-rules      |    1000 |                 20 |                   |     501.224 |    1995.1 |        3 |
| magika2-rules-only |    1000 |                 20 |              20   |      37.201 |   26880.8 |        3 |
| trid               |    1000 |                 20 |                   |    1531.18  |     653.1 |        3 |
| libmagic           |    1000 |                 50 |                   |     406.557 |    2459.7 |        3 |
| magika1            |    1000 |                 50 |                   |    2386.32  |     419.1 |        3 |
| magika2-gpu-ml     |    1000 |                 50 |                   |     239.679 |    4172.2 |        3 |
| magika2-gpu-rules  |    1000 |                 50 |                   |     172.37  |    5801.5 |        3 |
| magika2-ml         |    1000 |                 50 |                   |     634.611 |    1575.8 |        3 |
| magika2-rules      |    1000 |                 50 |                   |     324.617 |    3080.6 |        3 |
| magika2-rules-only |    1000 |                 50 |              50   |      37.902 |   26383.5 |        3 |
| trid               |    1000 |                 50 |                   |    1594.3   |     627.2 |        3 |
| libmagic           |    1000 |                    |                   |     869.563 |    1150   |        3 |
| magika1            |    1000 |                    |                   |    2143.49  |     466.5 |        3 |
| magika2-gpu-ml     |    1000 |                    |                   |     251.66  |    3973.6 |        3 |
| magika2-gpu-rules  |    1000 |                    |                   |     209.792 |    4766.6 |        3 |
| magika2-ml         |    1000 |                    |                   |     622.316 |    1606.9 |        3 |
| magika2-rules      |    1000 |                    |                   |     504.784 |    1981   |        3 |
| magika2-rules-only |    1000 |                    |              22.9 |      54.008 |   18515.6 |        3 |
| trid               |    1000 |                    |                   |    1774.83  |     563.4 |        3 |

Rule ratios count signature decisions from the rules-only reference; unknowns abstain and no ML control runs. A dash is the natural sample mix or a tool without a rules mode.

Common unambiguous vocabulary: 138 classes, 1071 files. Selected from class metadata and model support, before scoring; this is not proof of each tool's format support.

| Tool               | Common-vocabulary accuracy   | Mapped coverage   | Macro recall   |
|--------------------|------------------------------|-------------------|----------------|
| magika1            | 88.70%                       | 96.08%            | 91.96%         |
| magika2-ml         | 88.70%                       | 96.08%            | 91.96%         |
| magika2-rules      | 88.98%                       | 96.17%            | 92.17%         |
| magika2-gpu-ml     | 88.70%                       | 96.08%            | 91.96%         |
| magika2-gpu-rules  | 88.98%                       | 96.17%            | 92.17%         |
| magika2-rules-only | 25.40%                       | 25.40%            | 28.69%         |
| libmagic           | 53.69%                       | 55.09%            | 59.69%         |
| trid               | 61.62%                       | 67.97%            | 66.63%         |

