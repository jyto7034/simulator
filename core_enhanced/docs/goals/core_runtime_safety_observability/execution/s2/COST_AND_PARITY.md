# Paired observation cost and gameplay parity

35 release process runs (7 modes ×5 alternating-order rounds) have exact gameplay SHA256 `67a63efdd38dc1f4f33f3d67d46d141fd73f42d17d021bd83a7fbb27b5669958`. All result/error/event payloads, final Run/reconnect projections and retreat output match; only operation_wall_ns and the separate observation object are excluded. Each run advances1100×10ms and the same2000ms concentrated call. All5 full-queue runs dropped84 observation summaries; no gameplay result differed. Scrape services were stopped, and the failed-exporter mode holds its actual loopback port occupied. Stalled mode holds an actual crossbeam epoch guard while the worker runs. Off/on binaries, commands and raw JSONL SHA256s are bound in RESULT.json and driver_binaries002.json.

Median microseconds (normal tick n=5500 per mode, other rows n=5 per mode):

| Mode | normal_tick | concentrated_tick | pause_battle | resume_battle | authority_rejection | native_rejection |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| off | 157.87 | 143.65 | 379.54 | 124.69 | 158.50 | 89.69 |
| none | 157.22 | 144.37 | 337.27 | 123.92 | 532.87 | 226.49 |
| detached | 158.15 | 144.48 | 340.25 | 125.04 | 562.00 | 226.09 |
| queue_full | 162.24 | 144.49 | 359.41 | 124.73 | 553.11 | 226.23 |
| on | 163.34 | 147.90 | 348.46 | 132.62 | 546.18 | 223.60 |
| stalled | 163.66 | 145.44 | 339.80 | 128.96 | 553.79 | 222.69 |
| exporter_failed | 162.13 | 144.40 | 356.00 | 131.61 | 547.40 | 231.14 |

Raw per-operation timings, p95/max and all5 per-process samples are retained. Overall process time also includes loader/output work and is not the operation metric. This is a local measurement, not a new latency gate or a statistical zero-overhead claim. Feature-enabled refusal paths are slower in this sample even with no observer; the report preserves that difference rather than attributing all timing variation solely to collector work. The standard exporter runs on separate owned threads and observation losses are expected in overloaded modes. Current S1/R06/R07 cost guards are verified separately.
