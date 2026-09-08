# Whole-process startup accounting

Twenty measured iterations after one warmup. Parent timestamps are added directly to Hyperfine 1.20.0 around its existing spawn/wait timer; parent JSON is emitted after timing stops. Child timestamps come from Magika. Every traced iteration reconciles exactly using the same clock; arithmetic means below also add to the total.

| Boundary | Mean ms |
|---|---:|
| Before main: process creation, loader and runtime startup | 3.0930 |
| Inside main: rules load, classification, output and local cleanup | 1.5054 |
| After main timer: trace emission, exit and parent wakeup | 0.6456 |
| Total traced process | 5.2439 |

Untraced Hyperfine median: 4.5542 ms. Traced median: 5.2571 ms. These remain separate observations; the traced component table is not retroactively assigned to the earlier 4.640 ms median.

Before-main is not isolated dyld time. After-main includes trace emission. The trace comparison therefore quantifies instrumentation overhead rather than assuming it is free. Raw parent/child timestamps, stdout decisions and Hyperfine iteration timings are retained.
