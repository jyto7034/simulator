# 승인된 모드로 로컬 게임과 서버 실행하기

2026-10-12 실제 [literal entry 검증](../goals/core_mod_validation_apply/execution/single_goal/test_parts/runtime001/evidence/reviewed_adoption_functional002/24__public_entry__literal_smoke.json)은 launcher validate/query/apply·정상 starter run/Continue와 server WebSocket run·재시작 Continue·exact checkpoint 보존·invalid content의 persistence 이전 거절을 확인했다. 이 가이드의 실행 예는 그 검증 범위를 따른다.

일반 실행은 `AdmissionSession`이 실제 Lab 결과와 private issuer의 승인을 확인한 뒤 시작한다.
`author_content validate` 또는 `content_pack::load`의 정적 publication만으로 일반 호스트를 열 수는 없다.
계약은 [Data Loading Contract](../data_loading_contract.md)와 [Core Runtime Contract](../core_runtime_contract.md)가 소유한다.

## 1. 같은 빌드의 plain·instrumented worker 준비

저장소 루트에서 지원 빌드 도구를 사용한다. `--out`은 매번 존재하지 않는 새 경로이며,
plain과 instrumented의 target directory를 분리한다.

```bash
python3 tools/cmva_build.py --mode plain --kind bins \
  --out /tmp/cmva-build/plain001 --target-dir /tmp/cmva-plain
python3 tools/cmva_build.py --mode instrumented --kind bins --worker-only \
  --out /tmp/cmva-build/instrumented001 --target-dir /tmp/cmva-instrumented
```

현재 지원 행렬은 root workspace의 실제 lock·dependency graph·feature·compiler·target·release profile을
proof에 기록한다. 일반 호스트는 default-release이고, instrumented worker만 `RUNTIME_OBSERVATION`을 가진다.
다른 effective dependency graph, source/compiler 또는 unbound build는 승인을 발급할 수 없다.
WorkerPair는 실제 executable handshake와 hash를 확인한다. `cargo build --release`만 실행한 unbound binary는
진단 용도이며 승인용 빌드를 대신하지 않는다.

## 2. 명시적인 admission 설정

콘텐츠는 pack 디렉터리, trusted base selection과 mod registry로 준비한다. Registry는 mod ID별 pack 묶음·
의존 관계·최종 선택을 소유한다. 각 mod의 base+dependency 검증과 최종 조합의 실제 실행은 별개다.
실패한 mod의 종속 항목은 held로 남고 독립적인 mod는 계속 검증한다. 줄어든 최종 선택도 원 authored
winner를 유지하므로 held winner를 다른 pack으로 자동 대체하지 않는다.

호스트가 읽는 JSON은 다음 필드를 명시한다. 예시는 자신의 실제 파일과 worker 경로로 채운다.

```json
{
  "schema_version": 1,
  "content_root": "/srv/cmva/content",
  "registry_path": "/srv/cmva/registry.json",
  "base_selection_path": "/srv/cmva/base-selection.json",
  "runtime_safety_path": "/srv/cmva/runtime-safety.ron",
  "suite_path": "/srv/cmva/suite.json",
  "individual_suite_paths": {},
  "issuer_root": "/srv/cmva/private/launcher-issuer",
  "plain_worker": "/tmp/cmva-plain/x86_64-unknown-linux-gnu/release/mod_launcher",
  "instrumented_worker": "/tmp/cmva-instrumented/x86_64-unknown-linux-gnu/release/mod_launcher",
  "environment": "<reviewed measurement environment identity>",
  "profile_revision": "<exact trusted suite/profile revision>",
  "repeats": 3,
  "timeout_ms": 300000,
  "max_output_bytes": 16777216
}
```

Suite는 변경한 definition/site의 자동 coverage 의무를 줄일 수 없다. 실제 normal preparation·effect·
termination witness가 필요하며, 지원하지 않는 kind/operation이나 필요한 target/trigger가 없으면 `Incomplete`다.
지원 표는 `mod_launcher --cmva-coverage-support`로 조회한다. 이는 모든 mod의 호환 보장이 아니다.

Private issuer/cache와 Lab work root는 실제 POSIX filesystem의 현재 사용자 소유 `0700` directory여야 한다.
Symlink·다른 owner·느슨한 mode를 거부한다. Windows mount에서 `chmod` 명령의 성공만으로 권한을 추정하지 않는다.
Private key와 전체 authority store를 배포하거나 결과 JSON으로 승인을 재구성하지 않는다.

## 3. 실제 측정 검토 후 criterion 채택

해당 호스트 executable 자체에서 측정한다. `mod_launcher`, `game_server`, 다른 소비자 executable은
각각 자신의 executable identity가 포함된 context를 사용하므로 승인 자료를 서로 이식하지 않는다.
아래 evidence directory는 존재하지만 비어 있어야 한다. Authority·work·evidence는 실제 POSIX
filesystem에 준비하고 `stat`으로 mode와 현재 owner를 확인한다. 다음은 새 경로를 만드는 예다.

```bash
umask 077
mkdir -m 700 /srv/cmva/private
mkdir -m 700 /srv/cmva/private/launcher-issuer \
  /srv/cmva/private/launcher-raw /srv/cmva/private/launcher-adoption
stat -c '%a %u %n' /srv/cmva/private /srv/cmva/private/launcher-issuer \
  /srv/cmva/private/launcher-raw /srv/cmva/private/launcher-adoption
id -u
```

각 mode는700, owner는 `id -u`와 같아야 한다. 이미 사용한 evidence directory를 비우거나 덮어쓰지 않고
새 경로를 만든다. `reviewed-limits.json`은 검토한 development 결정의 다음 네 key/value를 가진다.

```json
{
  "max_elapsed_ns": 6000000000,
  "max_requested_heap_bytes": 1342177280,
  "max_rss_bytes": 1610612736,
  "max_native_work": 233000000
}
```

```bash
mod_launcher --cmva-calibrate admission-launcher.json /srv/cmva/private/launcher-raw
mod_launcher --cmva-adopt-calibration admission-launcher.json \
  /srv/cmva/private/launcher-adoption cmva-dev-cadence-v3-20261012 reviewed-limits.json
```

첫 명령은 실제 plain/instrumented 반복·parity·coverage·ordered phase·heap/RSS/native work를 기록한다.
측정이 complete여도 criterion을 아직 채택하지 않은 결과는 `Incomplete`다. 검토한 유한 limits를 제공하는
두 번째 명령은 같은 실제 context를 새로 측정한 뒤 private issuer에 criterion을 봉인한다.

현재 CMVA 개발 결정은 phase/sample마다 elapsed `6000000000ns`, requested heap `1342177280bytes`,
RSS `1610612736bytes`, native work `233000000`이다. 이는 native operation의 hard cap을 늘리지 않는다.
Development profile `config/runtime_safety_cmva_authoring_dev_v2.ron`은 authoring work `100000000`,
clone slots `1000000`을 명시한 별도 설정이다. Embedded 기본값·native `10000000`/`16MiB`와
expression 한도를 유지하며, 임의 규모의 콘텐츠 완주를 보장하지 않는다. 실제 선택한 RON bytes/hash를 context에 결속한다.

## 4. Validate·Query·Apply와 완성된 starter 흐름

```bash
mod_launcher validate admission-launcher.json
mod_launcher query admission-launcher.json
mod_launcher apply admission-launcher-b.json
mod_launcher run admission-launcher.json /srv/cmva/saves commands.json
mod_launcher continue admission-launcher.json /srv/cmva/saves
```

`validate`는 individual/held/final 조합 결과를 출력한다. `query`/`apply`는 그 invocation의 승인된
future selection을 출력한다. 이후 invocation에서 B를 쓰려면 같은 B admission 설정을 전달한다.
현재 A run/actor는 A를 계속 pin하며 B Apply는 다음 새 run/actor에 적용된다. 승인 없는 DTO와 stale capability는
Apply를 대신할 수 없다.

`run`은 최대256개 명령의 JSON 배열을 실행한다. Starter 확정 전에는 이어하기가 없다. 먼저
`[{"type":"start_new_game","game_mode":"Standard","replace_active_run":false}]`를 별도 discovery root에서
실행해 실제 VN session/scene을 확인한다. 검증한 단일 무선택 briefing fixture에서는 다음 세 명령으로
starter 확정까지 실행했다. 별도 실제 save root를 사용하고 session/scene은 discovery 결과의 실제 값을 넣는다.

```json
[
  {"type":"start_new_game","game_mode":"Standard","replace_active_run":false},
  {"type":"advance_visual_novel","session_id":"<returned session>","current_scene_id":"<returned scene>"},
  {"type":"select_starter_employees","employee_definition_ids":["field_medic","breach_guard","records_clerk"]}
]
```

다른 VN 흐름은 반환된 현재 context의 advance/choice 권한을 따라 필요한 명령을 반복한 뒤 starter를 선택한다.
한 번의 advance가 모든 briefing을 끝내지는 않는다. 실제 server smoke의 네 command는 Start·Advance·Starter와
재시작 뒤 ContinueRun을 합산한 수다.
확정 뒤 `continue`는 저장한 run seed·map·required configuration을 복원한다. 재시작에는 저장한 required
configuration과 일치하는 승인 설정을 전달한다. 저장한 run을 교체할 때는 Start가 허용되는 새
`NotStarted` host/session에서 `replace_active_run:true`를 명시한다. 이미 진행 중인 game의 Start는 이 flag가
true여도 `context_irrelevant`로 거절된다. `recovery_pending` 결과가 나오면 새 mutation 전에 같은 accepted intent의
복구를 마친다. 요구 mod가 없거나 incompatible save이면 원 파일·backup·journal을 보호하고 거절한다.

## 5. 정상 game_server 시작

서버 executable에서 별도의 `--cmva-calibrate`와 `--cmva-adopt-calibration`을 수행한다. 인수 형식은
위 launcher와 같고 admission 파일·issuer·evidence는 server owner의 것이다. Launcher의 approval/criterion을 복사하지 않는다.

실제 서버가 읽는 `config/development.toml` 또는 `config/production.toml`의 top level에 다음을 추가한다.

```toml
mod_admission_path = "/srv/cmva/admission-server.json"

[persistence]
active_run_directory = "/srv/cmva/saves/active"
player_profile_directory = "/srv/cmva/saves/profile"
```

기존 server/session/retry/logging/content 설정은 함께 유지한다. `RUN_MODE`가 선택한 파일이 존재하는
working directory에서 default-release `game_server`를 실행한다. Bootstrap은 승인 이후에 persistence와
LoadBalanceActor/AppState를 열고 `/game` WebSocket을 제공한다. `mod_admission_path` 누락·invalid content·
미승인 조합이면 정상 시작을 거절하며 static loader나 embedded content로 우회하지 않는다.

Active/profile root는 명시적으로 같은 directory 또는 별도 directory를 사용할 수 있다. Readonly preflight는
두 root의 current/backup/journal을 먼저 검사하며 directory·lock 생성, profile load와 backup repair보다 앞선다.
재시작 Continue도 같은 approved host 경계를 사용한다. 정확한 WebSocket envelope와 command 권한은
외부 Unity transport canonical을 따른다.
