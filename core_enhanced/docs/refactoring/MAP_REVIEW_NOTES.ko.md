# 시스템 지도 조사 기록

## 작업 범위와 기준선

- 요청: `refactoring/README.md`의 현재 상태와 다음 행동을 확인해 이어서 수행하고 결과·재개 위치를 갱신한다.
- 선택 작업: 현재 Core 전체 시스템 지도, 상세 조사 순서와 첫 상세 조사 대상 제안.
- 종료점: 지도와 근거·미확인 경계·다음 행동 기록. 제품 구현이나 시스템 상세 감사 완료를 주장하지 않는다.
- 기준선: [입력 목록과 해시](evidence/map_001/baseline.json), [기존 Git 변경](evidence/map_001/git-status-before.txt).
- 현재 working tree의 변경·신규·삭제 파일을 사용자 작업으로 보존한다. HEAD만으로 실제 조사 입력을 식별하지 않는다.
- `docs/goals/**`의 본문은 조사 입력에서 제외한다. 테스트·빌드는 이 지도 작성의 필수 검증이 아니며 실행하지 않았다.

## 확인한 경계와 조사 메모

### 실행 루트와 공유 상태

- `src/lib.rs`는 application, authoring, game, mod_validation, runtime_safety와 선택적 runtime_observation을 노출한다.
- `CoreApplication`은 `GameCore`를 소유한다. `GameCore`는 `GameCoreState`와 검증된 게시 데이터를 소유한다.
- `world/state.rs`는 원정·전투 세션·checkpoint·전투 진입 snapshot을 함께 표현하므로 단일 시스템으로만 배정하지 않는다.
- `world/application.rs`는 명령별 ApplicationOwner와 준비·commit 경계를 제공한다. 상세 조사 시 실제 분기별 의미를 추적해야 한다.

### 빌드와 제외 영역의 실제 연결

- workspace `Cargo.toml`과 Core `build.rs`에서 `docs/goals/**` 아래의 별도 probe crate 참조를 확인했다.
- 이 경로는 과거 문서를 근거로 삼은 것이 아니라 현재 manifest·build 코드에서 확인한 외부 검증 의존이다.
- 사용자 제외 범위를 유지해 probe 본문은 읽지 않는다. 일반 Core 테스트와 workspace 전체 빌드·최종 검증의 범위를 동일하게 취급하지 않는다.
- build identity는 Core 외에도 server·공식 데이터·runtime safety 설정 등을 입력으로 사용한다. 실제 재빌드 영향은 Core 소스 파일 변경에만 한정되지 않는다.

### 전투 종료와 사용자 결과 확인

- `world/combat.rs::handle_explicit_combat_node`에서 원정 상태를 받아 전투 구성·세션 진입을 준비하는 흐름을 확인했다.
- `transition_finished_live_battle_to_result`는 live session을 정리하고 `settle_combat_result`를 호출한 뒤,
  `ConfirmedCombatResult`에 acknowledgement·continuation phase 등을 담아 CombatResult phase로 이동한다.
- `handle_complete_combat_result`는 저장 receipt를 소비하여 다음 phase를 정하고 acknowledgement를 반환한다.
  사용자 확인 버튼이 첫 정산 지점이라는 해석은 현재 코드와 맞지 않는다.
- 이 관찰은 처리 순서를 확인한 것이며 실패·재호출·rollback·저장 후 재개 정확성을 검증한 것은 아니다.
  해당 연결이 첫 상세 조사 S07의 중심이다.

### 공유 자원과 제작 실행

- `game/resource/**`의 typed resource 수명과 `game/resources/**`의 경제·장착·세션 타입을 구분해 배정했다.
- `authoring/trial.rs`의 TrialSession은 CoreApplication을 보유하고 실제 실행을 사용한다.
- 유물의 소유·검증/게시·전투 전달·효과 실행은 S05·S02/S11·S07·S10에 걸친 계약으로 기록했다.
- `world/state.rs` 같은 공유 파일과 공통 테스트 묶음은 파일별 배정만으로 내부 책임 조사가 끝나지 않는다.

### 외부 저장 소비자

- 서버 actor handler의 query·continue·준비된 변경 분기를 확인하고 `publish_prepared_application_bound` 연결을 읽었다.
- `player_profile_store/application_intent.rs`의 accepted 계약은 승인 이후 준비된 Core를 한 번 commit하도록 요구한다.
- 승인 구성 전용 persistence의 전체 구현, crash recovery, Unity 소비자, 공식 데이터 전체는 조사하지 않았다.
- 외부 경계 파일은 [외부 입력 목록](evidence/map_001/external-inputs.json)에 별도로 식별했다.

### 제외 범위와 테스트 연결의 추가 확인

- `mod_validation.rs`와 `content_pack/tests.rs`는 goals 아래 Rust 코드를 직접 포함한다.
- authoring structural/causal 테스트, checkpoint/revival 테스트, lifecycle cost 테스트도 goals fixture·입력을 참조한다.
- 참조 위치만 검색했으며 제외한 본문은 열지 않았다. 자세한 목록은 [지도 §5](SYSTEM_MAP.ko.md#5-포함제외미확인과-검증-지원)에 있다.
- 따라서 후속 검증 계획은 Core package test도 외부 입력을 사용할 수 있음을 반영해야 한다.

## 진행 상태와 확인 결과

- 전체 지도와 조사 순서 작성 완료: [SYSTEM_MAP.ko.md](SYSTEM_MAP.ko.md).
- [파일별 담당 목록](evidence/map_001/inventory.tsv): 457개 입력 배정, 미배정 0개.
  배정은 조사 책임이며 모든 파일 본문·테스트 의미를 감사했다는 뜻이 아니다.
- `rg --files --hidden --no-ignore`로 src/tests 목록을 다시 수집하고 manifest·lock·build를 합쳐 기준선과 비교했다.
  [확인 결과](evidence/map_001/verification.json): 추가·삭제·내용 변경 0개. 기존 제품 변경을 보존했다.
- 문서의 상대 파일 링크·주요 함수명·목록 배정·문서 끝 공백을 정적으로 확인했다. 제품 변경이 없으므로 빌드·테스트는 실행하지 않았다.
- 작업 중 `combat_setup/assembly.rs`라는 존재하지 않는 경로를 검색해 오류가 있었고 실제 파일 목록으로 바로잡았다.
  지도에는 실제 combat_setup 하위 모듈만 사용했다. 전투 실행 함수명도 `sim.rs` 선언과 대조해 바로잡았다.
- 다음 재개: 지도 §6의 S07 상세 조사. 원정→전투→종료 정산→receipt→결과 확인→후속 보상의 상태·commit·실패·재개 경계를 조사한다.
  개선 후보는 아직 확정하지 않았으며 다음 산출물은 상세 조사와 개선 계획이다.

## 후속 진행 연결

2026-10-16 후속 요청으로 [S07 상세 조사·계획](S07_COMBAT_BOUNDARY_REVIEW.ko.md)을 작성했다.
보상 수량·Continue 불일치 B1을 재현하고 R1 공통 후처리 통합 후보를 확인했다. 현재 재개 위치는 README §1을 따른다.
위 지도 단계의 무변경·미실행 기록은 당시 범위의 사실이며 후속 probe 실행 결과는 S07 기록이 소유한다.
