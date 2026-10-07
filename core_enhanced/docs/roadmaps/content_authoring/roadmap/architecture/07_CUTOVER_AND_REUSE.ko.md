# A07 — 기존 코드 재사용·교체와 단계별 전환 설계

아래는 **현재 경로에서 새 책임으로 옮길 권장 지도**다. 각 파일 전체의 분석을 끝내고 정확한 patch를 지정한 표는 아니다. 구현 AI가 D1에서 생산자·소비자를 조사해 재사용/확장/교체를 최종 판정한다.

## 1. 전환 지도의 선택

| 기존 경계 | 새 책임 | 기본 판정 | 반드시 확인할 소비자·증거 |
| --- | --- | --- | --- |
| `data/live_ron.rs` 고정 source inventory/embedded | 외부 SourceSnapshot·PackResolver | 입력 확보·inventory 교체, 검증 자산 재사용 | startup·공개 construction·RON loader 테스트 |
| `data/mod.rs::load_live_sources` | ValidatedGeneration publication | validate-before-publish 경계 재사용 | 생성 preview·setup·new run pin |
| `ability.rs`·`data/skill_data.rs` | typed AST·EffectSite/ActionSite·compiled flow | 작성 단위와 compiler 구조 교체/확장 | cast preview·validation·execution·descriptor |
| `sim.rs::build_skill_step_commands` | compiled op→effect service request | enum/단계-wide 해석 경계 교체, leaf damage/heal 호출 재사용 | 기존 basic/skill source·처치·공명·result |
| `skill_runtime/{cast,projectile,area}` | ActionHandle·progress/result·delivery owner | 전달 kernel 재사용, 완료·수명 포트 확장 | first hit/terminal·no-hit·withdraw·stale event |
| `stats.rs`·`stat_pipeline.rs` | typed expressions·ModifierLedger·SpellPower | 산술 재사용 가능한 부분 유지, stat authority 확장 | 무기 적용·수치 중복·display·power·save |
| `core/types.rs`·`ids.rs`·buff key | StateOwner×Lifetime·stable source/copy binding | identity/lifetime 구분과 handle 도입 | 재배치·사망 cleanup·proc 사용 기록 |
| `skill_fragment.rs`·loadout | OwnedFragment aggregate | 종류별 progress 장부 교체 | grant·equip·bloom·선택·분쇄·checkpoint |
| `triggers.rs`·`damage_runtime.rs`·`death_runtime.rs` | typed facts/receipts·reaction queue·suppression | 계산·bookkeeping 재사용, 사건·출처 경계 확장 | OnAttack/OnHit·OnKill·wave·최종 결과 |
| `data/boss_runtime_data.rs`·boss runtime | 확장된 task/port·blackboard·selector | 기존 계층형 제어·검증 유지, action/signal 연결 | action gate·phase timing·cancel·schema future rejection |
| `summon_runtime`·`spawn_kernel` | 구분된 actor/trap/object 생성 | 원자적 생성·ID·배치 검증 재사용 | 수명·진화·reward/wave 권한 |
| `world/state`·`combat`·`active_run` | 사본·attempt overlay·result commit·fingerprint | 기존 경계 재사용, 신규 상태 동반 전환 | retreat/retry/load/reconnect·profile 보존 |

짧은 경로는 `core_enhanced/src/game/` 기준이다. 새 추상화를 만들기 전에 기존 기능이 이미 소유하고 있는 부분을 조사한다. 동일 책임의 두 시스템을 영구 병행시키지 않는다.

## 2. 최신 경로 하나로 전환하는 권장 순서 후보

CA01은 최소 typed Effect/Action 구문과 현재 kernel에 연결할 실제 피해·회복 slice를 만든다. CA02는 그 최신 schema로 최소 공식 pack·필수 카탈로그 참조·startup를 함께 전환한다. 모든 향후 콘텐츠 효과를 구현할 때까지 구형 parser를 유지하는 방식은 사용하지 않는다.

이 시점의 최소 playable profile은 시험·개발 부트스트랩임을 표시한다. 아직 구현하지 않은 고급 기능은 registry에서 unsupported로 진단한다. 구형 placeholder RON을 새 공식 콘텐츠의 완성으로 이름만 바꿔 출판하지 않는다.

이후 각 기능이 추가될 때 해당 RON·descriptor·compiler·runtime·진단·test를 함께 늘린다. development 중 schema가 바뀌면 현재 지원하는 파일·fixture를 최신으로 전환하고 과거 alias를 남기지 않는다. 코드가 실제로 채택한 버전 값을 사용하며 문서에서 임의 다음 version 숫자를 확정하지 않는다.

기존 kernel의 입력으로 변환하는 typed 포트는 레거시 호환 계층이 아니다. 그것이 최신 compiled op를 기존 계산·전달 함수에 연결하고, 구형 문법/의미를 별도로 실행하지 않는다면 올바른 재사용이다. “adapter”라는 이름만 보고 전부 삭제하지 않는다.

## 3. 테스트 전환

기존 assertion마다 보존할 의미, 승인 정책 때문에 바뀌는 의미, 삭제할 과거 정책을 구분한다. 유지해야 할 regression은 새 경로에서 다시 테스트한다. 고정 카탈로그 수와 ID는 공식 데이터 coverage용 검사로 둘 수 있지만, 범용 loader의 신규 콘텐츠 차단 조건으로 쓰지 않는다.

private builder로 만든 작은 kernel fixture와 정상 외부 loading/Application 포트의 인수 시험을 구분한다. 둘 다 가치가 있지만 전자만으로 제작 기반이 검증된 것은 아니다. 새 CLI/helper가 콘텐츠 ID를 알고 특수 행동을 대신하면 불합격이다.

## 4. 저장·preview·전투력의 전환 동반

새 사본·형태·modifier가 생기는 시점에 owner 문서와 snapshot/save shape·preview·진단 영향도 확인한다. CA19는 전체 검증이며 그 이전에 잘못된 저장을 출판하는 것을 허용하지 않는다.

범위 preview는 공통 geometry·compiled scope에서 계산하고 효과별로 설명할 수 있게 한다. 가시화 때문에 runtime state를 바꾸거나 실제 스킬을 시험 발동하지 않는다. Certified 계산은 최종 활성 계획의 승인된 모델을 사용하고 미정 가중을 임의로 추론하지 않는다.

## 5. 완료된 단계 사이의 일관성

DG에서 현재 K revision을 확인한다. 변경된 shared type 때문에 선행 단계의 테스트가 무효가 되면 해당 receipt를 갱신한다. 코드 변경과 최신 schema 데이터 전환을 원자적인 통합 단위로 완료하되 실제 commit 수까지 강제하지 않는다.

전체 프로젝트의 untracked/dirty 파일을 깨끗하게 만드는 것이 목표가 아니다. 사용자 변경을 보존하고 자신의 수정만 기록한다. 서버·Unity의 과거 불일치를 이 전환의 선행 blocker로 만들지 않고 해당 비범위 영향을 명시한다.

## 6. 순서와 전환의 확인 수준

위 CA별 전환은 제안이다. 현재 코드에서 모든 reader/writer와 단계 중간 상태를 검증한 실행 계획이 아니다. 기존 kernel을 사용할 때도 어느 입력·상태·결과 계약이 맞는지 확인한다. 선택 Topic의 P2G-20/40이 [의존성 기준](../DEPENDENCIES.ko.md)에 따라 최소 실행 slice·동시 전환·검증 선행을 확정한다.

파일별 책임 지도나 같은 날짜의 문서라는 이유로 current 코드·배포·테스트 상태를 추정하지 않는다. 이번 개정은 코드의 실제 순서를 새로 조사하거나 edge를 재배열하지 않았다.
