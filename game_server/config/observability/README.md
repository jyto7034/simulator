# Local runtime observation

Build the host with `cargo build -p game_server --features runtime-observation`.
Copy `game_server/config/observability.ron` to a local settings file, set `enabled: true`,
and supply its path through `GAME_OBSERVABILITY_PATH` when starting the host.
Without the feature or an enabled file, observation is off. Observation settings are
separate from `config/runtime_safety.ron` and do not change runtime safety allowances
or content/profile identity. HTTP listeners are loopback only; defaults are metrics
`127.0.0.1:9109`, detail `127.0.0.1:9110`, snapshot200ms.

With installed Prometheus and Grafana binaries, run:

```sh
python3 game_server/config/observability/run_local.py \
  --prometheus /path/to/prometheus \
  --grafana-home /path/to/grafana \
  --state-dir .cache/my-new-observation-run
```

The state directory must be new. The helper starts its own local services and stops
them on Ctrl-C; logs and retained data remain in that directory. Prometheus binds
127.0.0.1:9090 with1s scraping and24h retention. Grafana binds127.0.0.1:3000,
provisions the Prometheus datasource and the [Core runtime dashboard](http://127.0.0.1:3000/d/core-runtime),
and permits local anonymous viewing. Dashboard refresh defaults to1s. Dashboard
file provisioning polls every11s to work on filesystems without native watch events.
These are development services, with no remote deployment or account setup.

To select an active battle, read `http://127.0.0.1:9110/state` for session IDs, then:

```sh
curl -H 'Content-Type: application/json' -d '{"session":1}' http://127.0.0.1:9110/selection
# Clear the selected slot:
curl -H 'Content-Type: application/json' -d '{"session":null}' http://127.0.0.1:9110/selection
```

Selection is one slot, not a label per player. Metadata includes the battle UUID,
wall selection timestamp, generation and a300-row ring. Selection/end boundaries
are explicit; the simulation-time panel plots points against battle milliseconds.
Choose the corresponding wall interval when comparing selections. Old samples
remain in Prometheus history after a battle ends; current aggregate/selection
gauges are cleaned independently of observation queue delivery.

The dashboard distinguishes attempted operation work/duration/admission and
committed state. Accounted retained extent and cumulative admitted bytes are
accounting quantities, not RSS. A rejected or discarded draft can record work and
brief peaks without changing committed gauges. Histogram samples can be lost when
the bounded queue is full or a lease has expired: check loss counters and collector
freshness alongside distributions. Operation/rejection/commit counters and current
state also have fixed atomic publication outside the queue. Exporter failure does
not change Core results; a stale dashboard does not indicate paused gameplay.

The [metric and storage contract](../../../core_enhanced/docs/goals/core_runtime_safety_observability/execution/s2/corrections/n01_metric_docs_001/METRICS_AND_STORAGE.md)
describes names, units, finite labels,1024-slot queue,128-summary batches, idle
histogram maintenance, retirement gating and lifecycle cleanup. The example
`observe_runtime` drives official external content through public Core APIs for
local verification; it does not change a saved game.

A published battle runtime fault ends its current observation without adding a
successful commit. The first failed operation retains its work/peak summary while
the owner stays alive; active aggregates and selected gauges clear, and the ring
records an `ended` boundary. Reconnect preserves the same faulted owner. The host
stops its timer and save writers; explicit Quit followed by a new owner and the
existing valid-checkpoint Continue is the recovery path. See the
[Core fault lifecycle contract](../../../core_enhanced/docs/core_runtime_contract.md#faulted-published-battle).
The `fault_runtime` example drives an actual native failure and collector cleanup
through public Core APIs, including detached and saturated observation queues.


## Trusted measurement profiles and process memory

Build `game_server` or `measure_runtime` with `--features runtime-measurement` to
include bounded native retention details and actual process allocator/RSS samples.
This feature includes observation; an enabled `GAME_OBSERVABILITY_PATH` file still
controls the local worker and HTTP endpoints. Default builds and deployment8MiB
remain unchanged. Select a trusted runtime-safety file before host startup through
`APP__RUNTIME_SAFETY_PATH`. Optional development revision1 allows a finite16/32MiB
retained emergency ceiling while keeping the original nominal warning and every
other guard. Content, saves and ordinary game commands cannot select this cap.

`/state` includes fixed session identity/profile metadata, committed owner extent,
bounded raw native operation summaries, process allocator counters and Linux RSS.
The session registry is256 and raw ring300; queue1024/batch128 remain. Native seed
attempted/accepted, accepted growth, cumulative request, owner extent and first
warning/refusal site are distinct. Missing native scope is not zero allocation.
The original failed-seed `admitted_bytes` meaning remains; use the new accepted
field for accepted admission. Hard refusal and failed candidates do not become
committed extent. Tiny explicit owner overrides remain effective even in dev mode.

Heap live/peak are whole-process **requested allocator bytes**, including observer
and other threads/sessions; they exclude allocator fragmentation. LinuxVmRSS and
lifetimeVmHWM are separate OS values. Never add admission, heap and RSS together or
attribute process heap to a session. Libraries do not install a global wrapper;
uninstrumented or uncertain values are null with an explicit status. Active-window
queries are read-only. A retiring/opening concurrent hook can make window peak
unavailable; this does not fault gameplay. Normal signal/server-future completion
prints one process window footer after dropping the observer and retains the
original shutdown Result. Abrupt process termination cannot guarantee a footer.

Use the bounded `measure_runtime manifest.json new-output.jsonl` binary for normal
public CoreApplication playback. Its expected effective profile gate runs before
Core creation, content opening or output creation. Output is create-new with
finite step/session/repeat/sample/file bounds and explicit loss/summary state.
Same-actor reconnect and rollback keep their actual owner; fresh Continue validates
the existing pre-entry checkpoint under the current trusted host profile.
See [the measurement guide](../../../core_enhanced/docs/goals/core_runtime_measurement_profiles/README.md)
for profile syntax, commands, metric scopes, concurrent window rules, actual
fixture provenance and later U30-C/U30-P tuning responsibilities.16/32MiB are
engineering measurement windows, not a final deployment budget or Unity result.
