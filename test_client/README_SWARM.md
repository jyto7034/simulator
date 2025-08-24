# Swarm Testing Guide

This guide explains how to run swarm tests for the match server using the test client.

## Overview

Swarm tests simulate multiple players connecting to the match server simultaneously to test:
- Server performance under load
- Matchmaking behavior with various player types
- Error handling and recovery
- System stability over time

## Quick Start

### 1. Using the Test Script (Recommended)

```bash
# Run basic swarm test
./test_client/scripts/run_swarm_tests.sh -t basic

# Run load test
./test_client/scripts/run_swarm_tests.sh -t load

# Run minimal smoke test
./test_client/scripts/run_swarm_tests.sh -t minimal

# Run with custom server
./test_client/scripts/run_swarm_tests.sh -t basic -s ws://192.168.1.100:8080
```

### 2. Using Cargo Tests

```bash
# Run individual test functions
cargo test --package test_client test_swarm_basic -- --nocapture
cargo test --package test_client test_swarm_load -- --nocapture
cargo test --package test_client test_swarm_minimal -- --nocapture
```

### 3. Using the Swarm Binary

```bash
# Run with custom config file
cargo run --bin swarm -- --config test_client/configs/swarm_basic.toml
```

## Configuration Files

### Basic Configuration (`configs/swarm_basic.toml`)
- **Duration**: 30 seconds
- **Players**: 10 players (1 shard × 10 players)
- **Behavior**: Mostly normal players with minimal edge cases
- **Use case**: Quick functional testing

### Load Test Configuration (`configs/swarm_load_test.toml`)
- **Duration**: 2 minutes
- **Players**: 100 players (2 shards × 50 players)
- **Behavior**: Diverse mix including slow, spiky, and problematic players
- **Use case**: Performance and stress testing

### Template Configuration (`configs/swarm_template.toml`)
- **Duration**: 60 seconds
- **Players**: 20-40 players (randomized)
- **Behavior**: Randomized behavior ratios within specified ranges
- **Use case**: Randomized testing scenarios

## Player Behaviors

The swarm test includes various player behavior types:

### Normal Behaviors
- **Normal**: Standard matchmaking flow
- **SlowLoader**: Takes longer to load assets
- **SpikyLoader**: Intermittent delays during loading

### Edge Case Behaviors
- **QuitBeforeMatch**: Disconnects before match is found
- **QuitDuringLoading**: Disconnects during asset loading
- **TimeoutLoader**: Never completes loading (causes timeout)

### Invalid Behaviors (Error Testing)
- **UnknownGameMode**: Requests invalid game mode
- **MissingFields**: Sends malformed messages

- **DuplicateEnqueue**: Sends multiple enqueue requests
- **WrongSessionId**: Uses incorrect session IDs

## Configuration Parameters

### Basic Parameters
```toml
duration_secs = 30          # Test duration
shards = 1                  # Number of parallel test groups
players_per_shard = 10      # Players per shard
game_mode = "Normal_1v1"    # Game mode to test
match_server_base = "ws://127.0.0.1:8080"  # Server URL
seed = 12345                # Deterministic seed for reproducibility
```

### Behavior Mix
```toml
[behavior_mix]
slow_ratio = 0.1                    # 10% slow players
slow_delay_seconds = 5              # 5 second delay
spiky_ratio = 0.05                  # 5% spiky players
spiky_delay_ms = 150                # 150ms spikes
timeout_ratio = 0.02                # 2% timeout players
quit_before_ratio = 0.03            # 3% quit before match
quit_during_loading_ratio = 0.03    # 3% quit during loading
invalid_ratio = 0.01                # 1% invalid behavior
```

## Results and Monitoring

### Test Results
- Results are saved to `logs/` directory in JSON format
- Include metrics like success rates, timing, and SLO compliance
- Timestamped files for historical comparison

### Real-time Monitoring
- Tests connect to the server's event stream for real-time monitoring
- Logs show player connection status and behavior outcomes
- Error conditions are captured and reported

### SLO (Service Level Objectives)
The swarm tests evaluate:
- **Match Success Rate**: Percentage of successful matches
- **Average Match Time**: Time from enqueue to match found
- **Loading Success Rate**: Percentage of successful loading phases
- **Error Rate**: Frequency of server errors

## Troubleshooting

### Common Issues

1. **Connection Refused**
   ```
   Error: Connection refused (os error 111)
   ```
   - Ensure the match server is running on the specified URL
   - Check firewall settings

2. **Test Timeout**
   ```
   Error: Swarm test timed out
   ```
   - Server may be overloaded or unresponsive
   - Increase timeout or reduce player count

3. **High Error Rate**
   ```
   Warning: SLO failed - high error rate
   ```
   - Check server logs for errors
   - Verify server configuration and resources

### Debug Tips

1. **Enable Debug Logging**
   ```bash
   RUST_LOG=debug cargo test test_swarm_basic -- --nocapture
   ```

2. **Run Minimal Test First**
   ```bash
   ./test_client/scripts/run_swarm_tests.sh -t minimal
   ```

3. **Check Server Health**
   - Verify match server is running and healthy
   - Check server metrics and logs

## Advanced Usage

### Custom Configurations

Create your own configuration file:

```toml
# my_custom_test.toml
duration_secs = 45
shards = 1
players_per_shard = 20
game_mode = "Custom_2v2"
match_server_base = "ws://my-server:8080"
seed = 98765

[behavior_mix]
# Your custom behavior ratios
slow_ratio = 0.15
# ... other parameters
```

Run with:
```bash
./test_client/scripts/run_swarm_tests.sh -t custom -c my_custom_test.toml
```

### Environment Variables

- `SMOKE_MATCH_BASE`: Override match server URL
- `SMOKE_GAME_MODE`: Override game mode
- `SWARM_SEED`: Override random seed
- `OBSERVER_STREAM_KIND`: Control event streaming (default: "state_violation")

### Integration with CI/CD

```bash
# Example CI script
#!/bin/bash
set -e

# Start match server
./start_match_server.sh &
SERVER_PID=$!

# Wait for server to be ready
sleep 10

# Run swarm tests
./test_client/scripts/run_swarm_tests.sh -t basic

# Cleanup
kill $SERVER_PID
```

## Performance Considerations

- **Memory Usage**: Each player actor consumes memory; monitor for large tests
- **Network Connections**: High player counts may hit connection limits
- **Server Resources**: Ensure adequate CPU/memory on the match server
- **Test Duration**: Longer tests provide better statistical significance

## Best Practices

1. **Start Small**: Begin with minimal tests before scaling up
2. **Use Deterministic Seeds**: For reproducible test results
3. **Monitor Resources**: Watch CPU, memory, and network usage
4. **Save Results**: Keep test results for trend analysis
5. **Test Incrementally**: Gradually increase load to find limits
