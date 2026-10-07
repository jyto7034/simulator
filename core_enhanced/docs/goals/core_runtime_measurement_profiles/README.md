# Runtime measurement profiles

U07-M은 정상 CoreApplication과 실제 호스트에서 유한한 개발 측정과 재현을 제공한다. 배포 기본 retained admission은8MiB이며 기존 기본 RON·effective hash·공개 RuntimeFactLimits 검증을 보존한다.16/32MiB는 유한한 개발 측정창으로 선택할 수 있는 engineering 값이다. 최종 배포값, 콘텐츠 전체의 최대 메모리 또는 Unity 플레이 적정성을 뜻하지 않는다.

구현·자체 검증의 현재 위치와 독립 완료 여부는 [PLAN](execution/single_goal/PLAN.md)을 따른다. [정책](WORKING_POLICY.ko.md), [설계](CODE_REVIEW_001.ko.md), [실행 원본](execution/single_goal/EXPERIMENTS.md)을 구분한다.

## Trusted profile

완전한 `config/runtime_safety.ron` 사본에서 기존 `native_runtime`를 유지하고 최상위에 다음 optional field를 추가한다.

```ron
development_measurement: Some((
    revision: 1,
    emergency_retained_bytes: 16777216,
)),
```

32MiB를 선택하려면 `33554432`를 사용한다. revision1만 지원하며 emergency는 원래 native retained warning보다 커야 하고32MiB 이하여야 한다. 원래 native DTO는 여전히8MiB 이내로 검증한다. 누락한 field는 canonical serialization에서 생략되어 기본 identity가 같다. unknown/duplicate/zero/overflow/과대 값은 거절하며 조용한 fallback은 없다.

호스트는 `runtime_safety_path` 또는 `APP__RUNTIME_SAFETY_PATH`로 시작 전에 trusted 파일을 선택한다. 콘텐츠·저장·일반 게임 명령은 그 파일을 선택하거나 cap을 올리지 못한다. live 소유자는 immutable GameData를 사용한다. fresh Continue는 현재 호스트 profile로 기존 pre-entry checkpoint를 다시 검증하며 새로운 live-battle disk codec을 만들지 않는다. 같은 actor의 reconnect와 rollback은 실제 소유자의 기존 profile을 보존한다.

`RuntimeFactLimits`의11필드/layout과 공개8MiB `validate()`/`with_runtime_fact_limits`는 유지한다. 개발 profile의 owner도 명시한 작은 override를 지킨다. private trusted entry만 선택된 retained ceiling을 검증한다. work·clone·객체 수·입력·expression scratch·overflow 보호를 높이지 않는다.

## Host collection

```sh
cd game_server
GAME_OBSERVABILITY_PATH=/absolute/path/to/enabled-observability.ron \
APP__RUNTIME_SAFETY_PATH=/absolute/path/to/trusted-runtime-safety.ron \
  cargo run --features runtime-measurement --bin game_server
```

`game_server` cwd에서 상대 `config/development.toml`을 읽으며 `cargo run`이 현재 target에 빌드한 같은 바이너리를 실행한다. 설정 파일 경로는 절대 경로를 지정한다. 기본 profile은 `APP__RUNTIME_SAFETY_PATH`를 생략해 기존 host 설정을 사용한다. measurement feature는 observation을 포함하지만 enabled 파일이 없으면 HTTP/worker는 시작하지 않는다. 라이브러리는 global allocator를 선언하지 않는다. 실제 host/measurement binary는 System wrapper 하나를 설치하며 wrapper 없는 library/example의 heap live/peak는 `null`/instrumented=false다.

기존 loopback `/state`는 최대256개 session,300개 raw operation,300개 selection ring과 손실 카운터를 제공한다. queue는1024, drain batch는128이다. weak lease가 사라지면 current session을 회수하고 유한 raw 원본은 남긴다. queue/registry/출력 포화는 관측 손실이며 이미 실행된 gameplay를 fault로 바꾸지 않는다. identity와 site는 metadata이고 metric label을 늘리지 않는다.

## Public playback

```sh
cargo run -p game_server --features runtime-measurement --bin measure_runtime -- \
  /path/to/manifest.json /path/to/new-output.jsonl
```

manifest는 schema1, case_id, external pack/selection, source provenance, 관측 모드, repeat/session, steps, 출력 상한을 가진다. 각 session은 player/seed/trusted runtime_safety_path와 **expected_effective_profile_sha256**을 지정한다. 실제 profile hash는 현재 `author_content validate ... --runtime-safety ...` 결과에서 읽는다. 모든 profile gate는 CoreApplication 생성·콘텐츠 열기·출력 생성 전에 실행된다. 기존 출력은 덮어쓰지 않는다.

실제 검증 manifest는 [normal default](evidence/playback_inputs_002/normal_default_none.json)와 [낮은 strict profile](evidence/lifecycle_samples_003/low_strict.json)에 원래 바이트로 남아 있다. 다음 최소 예시에서 경로·source provenance는 자기 실행 입력으로 지정하고 hash는 현재 validate 결과를 사용한다. 기본 hash는 아래의 원래 값이다.

```json
{
  "schema_version": 1,
  "case_id": "ordinary_default",
  "source_sha256": "f2144e9a4c5ddeb41245799294a75b41de9d843018cb2166f711b7f07821cda8",
  "pack_root": "/path/to/external-pack",
  "selection_path": "/path/to/selection.json",
  "expected_content_sha256": null,
  "observation": "normal",
  "repeats": 1,
  "sessions": [{
    "player": "00000000-0000-0000-0000-000000000019",
    "seed": 23,
    "runtime_safety_path": null,
    "expected_effective_profile_sha256": "a7b932aa614c11c8adf2b0e43c35c89f841056d877b78800e935d4d0c61f4a49"
  }],
  "steps": [{"kind": "enter_ordinary_combat"}, {"kind": "elapsed", "delta_ms": 10}],
  "max_samples": 4096,
  "max_export_bytes": 16777216,
  "stop_on_rejection": true
}
```

관측 모드는 `none`, 매 step drain하는 `normal`, receiver를 제거한 `detached`, 마지막까지 drain을 미루는 `full`이다. full은 deferred drain이며1024개를 넘는 실제 publication이 있을 때만 queue 포화를 입증한다. 동일 입력/seed/profile의 공개 결과는 네 모드에서 같다.

명령/admin/tick/snapshot/prepare-discard/reconnect/rollback/checkpoint는 공개 CoreApplication을 사용한다. 자동 ordinary combat 진입도 기존 공개 frontmatter와 명령을 사용한다. 중간 typed 거절은 kind/code/details digest와 실제 public operation 수를 보존한다. live owner의 Continue처럼 현재 API가 지원하지 않는 상태를 성공으로 바꾸지 않는다. 성공 Continue는 실제 호스트의 디스크 checkpoint→재시작→fresh application 표본으로 구분한다.

TrialStep 입력을 재사용하는 `script`는 앞선 **공개 응답**에서만 값을 결속한다. 응답 history는 session당256records/1MiB encoded JSON이고, Value 컨테이너의 실제 heap을1MiB라고 주장하지 않는다. 이미 회수되었거나 미보존된 응답은 assertion 판정불가다. 보존된 document 안에서 pointer가 없는 경우만 Exists(false)로 판정한다. resolved command와 원래 입력을 유한하게 기록하며 새 게임 권한을 만들지 않는다.

manifest≤1MiB, repeat≤64, live session≤16, 보수적으로 계산한 전체 public call≤10000, retained export sample≤4096, JSONL≤16MiB다. 한 record≤64KiB이고 footer64KiB를 예약한다. 최종 상태 최대1024개는 개별 capped sample과 rolling digest로 처리한다. 상한/손실/summary missing을 별도로 표시한다. session은 같은 프로세스에서 interleave하며 parallel session thread라고 부르지 않는다.

## Metrics and limits

| 값 | 범위와 해석 |
|---|---|
| seed attempted/accepted | 처음 native scope의 owner 시작값; 실패한 seed는 accepted가 아니다. |
| accepted growth/total | 허용된 양의 retained admission; 해제·nested·candidate discard·rollback으로 같은 operation 예산을 환급하지 않는다. |
| cumulative/owner max request | 누적 admission 요청과 별도 owner extent 요청; overflow는 unavailable flag다. |
| first soft/hard exceed | static site·request kind·requested·limit; soft는 warning 초과이며 hard fault를 무시한 결과가 아니다. |
| committed owner extent | 성공한 battle/pending resource 계산; prepared/failed candidate의 peak와 다르다. unmeasured/no owner/측정된 값은 구분한다. |
| legacy admitted_bytes | 역사적으로 거절된 첫 seed attempted를 포함할 수 있다. 순수 accepted로 재명명하지 않는다. |
| requested allocator live/peak/window delta | 프로세스 전체 System requested bytes; observer와 다른 thread/session을 포함하며 allocator 내부 fragmentation을 제외한다. |
| RSS/VmHWM | Linux bounded `/proc/self/status`의 실제 process RSS와 lifetime HWM. window peak가 아니며 실패/미지원은 명시 missing이다. |

이 값들을 더하거나 서로 대체하지 않는다. native retained가 없는 operation은0allocation이 아니라 native scope 미계량이다. non-battle persistence/resource rollback capture heap은 기존 native operation 계량 밖이며 process allocator에 포함한다. 이 준비 작업은 그 기존 보호/계량 범위를 새로 넓히지 않는다.

allocator hook은 시작부터 연속 추적하고 allocation/log/lock을 실행하지 않는다. 창 이전 allocation의 free도 포함한다. 창 baseline/end를 peak에 포함하며 hook을 live 변경 전부터 게시까지 등록한다. just-closed 창의 게시가 다음 창에 섞이지 않도록 successor begin을 막는다. 종료 시 hook이 남거나 opening 경계가 불확실하면 peak `null`/busy를 표시한다. active window 조회는 새 창을 열지 않는다. concurrent atomic 표본이며 엄밀한 정지 heap snapshot이 아니다.

분포의 mean/p95/max는 실제 **retained exported samples**에서만 계산한다. queue drop, export drop, raw count, truncated/summary missing을 함께 읽는다. p95나 짧은 측정 최대가 이론적 최대를 입증하지 않는다. source_sha256_supplied는 호출자 provenance이고 compile-time 정보가 없으면 compiled 값은 null이다. 실행 receipt의 exact source manifest·input·toolchain·실행 바이너리 hash를 함께 인수한다.

## Evidence scope and follow-up

현재 실제 표본은 공식 normal 전투, 원래 유효 U07 actor/trap/wall fixture, public independent-area 해제, 별도 TTL lifecycle, 네 관측 모드 포화, 동시 두 profile, 실제 WebSocket/actor/Continue/lease 회수다. fixture의 legacy starter/reward/custom actor/HP3 wall과 별도 TTL1000ms는 작성 입력 선택이며 공식 balance 변경이 아니다. 낮은 nominal 실험 profile의 budget 표본은 배포8MiB 또는8MiB 초과 실측과 분리한다.

기본 PC는 즉시 선행 U07의576행/36조건31성공5거절과 non-timing exact를 직접 비교한다. U06+200 owner 설명·17negative control은 보조 역사 guard다. R05는 원래 숫자·workload·guard와 승인된 concurrent-peak-only 예외를 보존한다. 기능 worker와 PC/R05 측정은 겹치지 않는다. 도구 준비를 실제 실행 완료로 세지 않는다.

다음 수치 조정은 새 profile/revision, 원시 전후 값·이유·영향·독립 인수를 남긴다. U08의 후속 소유자 작업, U11 identity, U30-C Core 표본, U30-P Unity 플레이, U31 최종 인수에 필요한 근거를 제공하며 전체 콘텐츠/Unity/최종 배포값 적정성은 그 후속 소유자에게 남긴다. 독립 Complete/High/open0 전에는 U08을 시작하지 않는다.

별도 낮은 nominal256KiB 표본은 preflight를 통과한 같은 actor first12 명령을 사용한다. native seed15701/queued_fact_source request262165에서 strict는accepted262029 뒤 거절했고 dev16은soft만 기록해accepted634913까지 진행했다. dev16 maxowner35461은 operation cumulative634913과 다르다. 앞선 entry-only256KiB 표본은초과없음(max12935),8KiB 표본은 loader request8323에서거절이므로 operation실행 후 거절로세지않는다. 이 결과는배포8MiB초과나최종한도적정성을입증하지않는다.

JSONL은header/profilegate provenance, capped input/resolved public step/result, nativeoperation, session committed/prepared metadata, finalstate sample 및footer를구분한다. footer에는모든public call/accepted/rejected/drivererror와queue/exportdrop·retainedsample분포가있다. 일반typed거절(kind/code/detailsdigest)은drivererror와다르며중단정책을함께읽는다. outputwritefailure나record초과는summary_missing/truncated/drop으로보고하며0실행·누락을PASS로바꾸지않는다. 정상종료실제host footer와CLI rollingfinaldigest는각자의측정scope이며정지snapshot/완전heapdump가아니다.
