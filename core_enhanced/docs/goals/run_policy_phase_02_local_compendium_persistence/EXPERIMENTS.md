# Local Encounter Compendium Persistence Experiments

Status: complete

Implementation complete: true

Created: 2026-07-27

## 2026-07-27 — Child Goal 초기화와 readiness audit

### 확인

- P01 completion review는 `Complete / High`이고 Master registry에서 P02가 active다.
- 신규 child 경로가 존재하지 않아 충돌 없이 생성했다.
- 승인 local profile policy와 monolith decision의 defeated/placeholder 9~10절을 다시 읽었다.
- campaign 중간 build의 기존 배포 profile 호환 요구는 없다. alias/default/dual reader를 만들지 않는다.

### 최초 current-state 발견

- Core `EncounterCompendiumProfileState`는 발견 abnormality/corrosion set 두 개만 가지며, import도 현재
  catalog membership을 요구해 dormant ID를 거부한다.
- producer는 실제 opponent spawn을 수집하고 node-consuming 결과에서 union하지만 defeated set과
  primary/non-primary `UnitDied` 의미가 없다.
- snapshot은 발견된 항목만 ID 정렬로 투영하며 authored 전체 순서, placeholder와 완료 표식이 없다.
- game_server load-balance actor가 transport `player_id`별 process-local `HashMap`을 durable authority처럼
  사용한다. profile file, checksum, lock, backup, bootstrap과 dirty retry가 없다.
- Unity reader는 발견된 exact metadata list만 읽고 profile persistence status/recovery/fatal UI가 없다.
- P01 active-run export test는 compendium이 payload에 없음을 이미 보호한다.

### Baseline

- P01 직후 `cargo test --workspace --all-targets`: Core library 762, integration 3/4/23/13/4,
  server 40, 실패 0.
- 원본 Unity MCP: EditMode 246/246, PlayMode 61 pass/0 fail/4 explicit skip.

### 다음 작업

- Core profile/set API와 battle result identity의 정확한 mutation boundary 설계
- live catalog authored order와 Unity current compendium display surface 감사
- P01 store를 복사하지 않고 공유 가능한 file lifecycle과 별도 aggregate 경계 결정

## 2026-07-27 — Core profile, producer와 projection 구현

### 변경

- profile을 abnormality/corrosion 각각의 `discovered`/`defeated` strict set 네 개로 확장하고
  `defeated ⊆ discovered`를 import/merge/save 경계에서 검증한다.
- persistence import는 형식·부분집합만 검증해 dormant ID를 보존하고, 새 gameplay commit은 현재
  catalog membership을 별도로 검증한다.
- 실제 battle spawn/result identity에서 발견을 추출한다. 승리 시 primary를 발견·격파하고 승리한
  attempt에서 실제 `UnitDied`인 non-primary만 격파한다. 패배는 발견만 commit한다.
- snapshot은 live catalog authored 순서의 전체 항목을 투영하고 미발견 metadata를 숨긴다.
- active-run restore와 transaction rollback은 local profile을 rewind하지 않고 현재 singleton profile을
  보존한다.

### 검증

- subset/malformed/current-catalog commit/dormant round-trip/idempotent union test 통과
- primary·non-primary 실제 사망, survivor와 defeat negative test 통과
- authored 전체 projection, placeholder metadata 은닉과 dormant 비투영 test 통과
- active-run rollback/profile monotonic independence test 통과

## 2026-07-27 — 별도 profile store와 server lifecycle 구현

### 변경

- `encounter_compendium.json`, rolling backup, temp, backup temp와 별도 lock을 사용하는 v1
  checksum envelope/store를 구현했다.
- current 우선 load, valid backup 복구, both-missing empty bootstrap, dual-corrupt/unsupported-version
  blocking, fsync/검증 뒤 atomic replace와 명시적 reset을 구현했다.
- OS application-data root 아래 상대 persistence 경로를 해석하고 절대 경로는 유지한다.
- load-balance의 transport player UUID별 `HashMap`을 제거하고 actor가 공유하는 local-install singleton
  profile runtime을 주입했다.
- profile publish 실패는 gameplay/active-run commit을 rollback하지 않고 최신 in-memory union을 dirty로
  유지한다. retry, bounded stop flush, backup recovery 1회 알림과 fatal reset/reload를 구현했다.
- active-run과 profile store가 저수준 replace/directory-sync helper만 공유하고 file/envelope/lock/state
  transaction은 분리돼 있다.

### 실제 파일·actor 검증

- missing/publish/load/backup recovery/reset/checksum/unsupported/dual corrupt/lock/publish failure/dirty
  retry 실제 filesystem test 통과
- 서로 다른 transport player가 같은 singleton profile을 보는 actor test 통과
- active-run write failure와 profile write failure가 상대 commit을 rollback하지 않는 test 통과
- fatal bootstrap gameplay gate, clean title reset 확인, backup recovery 알림 1회 test 통과

## 2026-07-27 — Unity consumer와 recovery UX 구현

### 변경

- strict compendium entry/status reader를 `discovered`, `defeated`, optional metadata와 exact persistence
  state로 확장했다.
- Main Menu에 discovered/defeated summary, authored entries와 `???` placeholder를 표시한다.
- recovered/dirty/fatal 상태를 표시하고 raw system path 대신 안정된 진단 code를 사용한다.
- fatal 상태는 Retry, 확인 modal을 거친 Reset과 local Quit만 제공한다. 정상 상태에서도 명시적 전체
  Reset을 확인 후 실행할 수 있다.

### 원본 Unity Editor MCP 검증

- focused EditMode: 8 passed, 0 failed
- focused PlayMode: 2 passed, 0 failed
- full EditMode job `58427561d3ee4dc3a7541c9c907d62cc`: 251 passed, 0 failed
- full PlayMode job `062d827a39334b24ac94d28b64ee4aee`: 62 passed, 0 failed,
  explicit live-server opt-in 4 skipped
- MCP server 재기동 뒤 `ark@b49d32f6f2f70bbe`를 다시 pin하고 asset refresh/domain reload를
  완료했다. Editor ready, compiling false, Console error 0건을 확인했다.

## 2026-07-27 — 최종 broad validation과 legacy 감사

| 명령/runner | 실제 결과 |
| --- | --- |
| `CARGO_INCREMENTAL=0 cargo test --workspace --all-targets --quiet` | Core 765, integrations 3/4/23/13/4, server 51, 실패 0 |
| Unity full EditMode | 251 pass, 0 fail |
| Unity full PlayMode | 62 pass, 0 fail, 4 explicit live opt-in skip |
| Unity refresh/compile/console | ready, compiling false, error 0 |

- canonical runtime/data/server/Unity 문서를 새 profile boundary와 exact projection/status로 갱신했다.
- official runtime/server/Unity/canonical에서 player-keyed profile authority, 구 discovered-only shape,
  발견 항목만 ID 정렬 projection과 “도감 UI 없음” 계약을 검색해 material 잔존 0건을 확인했다.
- 과거 Goal/감사 문서가 historical current-state를 설명하는 표기는 변경하지 않았다.

### 완료 판단

- `COMPLETION_REVIEW.md`: `Complete / High`
- Master P02 exit gate를 충족했다.
