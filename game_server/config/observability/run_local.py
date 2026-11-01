#!/usr/bin/env python3
"""Launch local standard Prometheus/Grafana using explicit installed binaries."""
import argparse
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import urllib.request


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--prometheus', type=Path, required=True)
    parser.add_argument('--grafana-home', type=Path, required=True)
    parser.add_argument('--state-dir', type=Path, required=True,
                        help='new directory for this local service run')
    args = parser.parse_args()
    config = Path(__file__).resolve().parent
    prometheus = args.prometheus.resolve()
    grafana_home = args.grafana_home.resolve()
    state = args.state_dir.resolve()
    for path in (prometheus, grafana_home / 'bin/grafana'):
        if not path.is_file():
            parser.error(f'missing binary: {path}')
    state.mkdir(parents=True, exist_ok=False)
    env = os.environ.copy()
    env.update(CRSO_DASHBOARDS_PATH=str(config / 'grafana/dashboards'),
               GF_PATHS_PROVISIONING=str(config / 'grafana/provisioning'),
               GF_PATHS_DATA=str(state / 'grafana-data'),
               GF_PATHS_LOGS=str(state / 'grafana-logs'),
               GF_PATHS_PLUGINS=str(state / 'grafana-plugins'))
    commands = [
        [str(prometheus), '--web.listen-address=127.0.0.1:9090',
         '--config.file=' + str(config / 'prometheus.yml'),
         '--storage.tsdb.path=' + str(state / 'prometheus-data'),
         '--storage.tsdb.retention.time=24h'],
        [str(grafana_home / 'bin/grafana'), 'server', '--homepath=' + str(grafana_home),
         '--config=' + str(config / 'grafana/grafana.ini')],
    ]
    processes, logs = [], []
    try:
        for name, command in zip(('prometheus', 'grafana'), commands):
            log = (state / (name + '.log')).open('xb')
            logs.append(log)
            processes.append(subprocess.Popen(command, env=env, stdout=log,
                                              stderr=subprocess.STDOUT))
        deadline = time.monotonic() + 45
        for url in ('http://127.0.0.1:9090/-/ready', 'http://127.0.0.1:3000/api/health'):
            while True:
                if any(process.poll() is not None for process in processes):
                    raise RuntimeError(f'service exited; inspect {state}')
                try:
                    with urllib.request.urlopen(url, timeout=1) as response:
                        if response.status == 200:
                            break
                except OSError:
                    pass
                if time.monotonic() >= deadline:
                    raise RuntimeError(f'service readiness timed out; inspect {state}')
                time.sleep(0.1)
        print('Grafana: http://127.0.0.1:3000/d/core-runtime', flush=True)
        print('Prometheus: http://127.0.0.1:9090 — local retention 24h', flush=True)
        print(f'Logs and data: {state}', flush=True)
        while all(process.poll() is None for process in processes):
            time.sleep(0.5)
        raise RuntimeError(f'service exited; inspect {state}')
    except KeyboardInterrupt:
        return 0
    finally:
        for process in reversed(processes):
            if process.poll() is None:
                process.send_signal(signal.SIGTERM)
        for process in reversed(processes):
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        for log in logs:
            log.close()


if __name__ == '__main__':
    sys.exit(main())
