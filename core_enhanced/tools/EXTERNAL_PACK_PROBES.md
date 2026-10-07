# External input for public API probes

R02's default release loader requires an explicit external pack selection. The positive
`validated_data_api_audit`, `application_authority_api_audit`, `battle_kernel_api_audit`, and
`core_public_projection_api_audit` consumers now take `TRUSTED_ROOT PACK_RELATIVE_DIRECTORY...`.
Their original behavior/API assertions are retained. For example, from the repository root:

```sh
cargo build -p game_core --release --no-default-features --bin package_official_content
./target/release/package_official_content game_resources /tmp/api-official-pack
cargo run --release --manifest-path core_enhanced/tools/validated_data_api_audit/valid/Cargo.toml -- /tmp api-official-pack
```

The output leaf must be absent. Supply the same two arguments when executing the other positive
consumers. `external_pack_probe.rs` is only their shared input adapter; it does not supply fixture
data, choose a fallback root, or change the assertions owned by each probe.

Negative privacy probes still require an unsuccessful default-release compile for their original
forbidden APIs. Their external input adapters prevent an unrelated removed loader from becoming
the sole error. The `runtime_architecture_final_validation/default_fixture_api_audit` probe
deliberately retains its forbidden embedded-loader and fixture-method calls: both APIs must be
absent in default release. Feature-enabled preservation/performance fixtures retain their explicit
`test-fixtures` capability. These core checks make no server or Unity compatibility claim.

`content_pack_release_probe.py --repository . --output-dir /tmp/new-release-probe` creates an
isolated source/custom/output fixture and executes the registered default-release binaries through
real briefing/starter commands. It requires an absent output directory, checks exact byte and
binary preservation, and leaves every command's output and receipt for inspection.
