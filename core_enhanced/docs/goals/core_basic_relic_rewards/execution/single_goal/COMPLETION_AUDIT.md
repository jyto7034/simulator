# U16-2 독립 완료 감사 — independent001

판정: **Complete / High**. 열린 완료 차단 0, 확인된 범위 내 제품 결함 0.

2026-10-15 사용자 요청 `CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md 문서를 지침 삼아 u16-2 독립 감사 작업에 착수해줄래?`를 GEX-20 `stage-run`으로 인수했다. 이 감사 세션은 U16-2 설계·구현·교정에 참여하지 않았다. 서브에이전트 없이 실제 소스, upstream 계약, scoped diff, 테스트 기대값과 원시 결과를 직접 검토했다. 구현 target·시험·fixture·동결된 구현 기록은 수정하지 않았으며 감사 결과만 별도 위치에 작성했다. 후속 구현 권한은 포함하지 않는다.

## 대상과 현재성

GS-CBRR-001 / GOAL-CBRR-01 / PHASE-CBRR-01. HA-CBRR-001이 지정한 FPF-CBRR-001·PP-CBRR-002·Goal Quality Contract v2, GEX2026-10-04-gex-r8, 현재 연속 실행 지시서와 로드맵 §1.9를 적용했다. [원본 Goal Set](../../P2G_50_GOAL_SET.ko.md)의 single_phase 수용17·차원10·통합5·실행노드2 전부가 감사 범위다.

[고정 대상](target/u16_2_current_001/AUDIT_TARGET_RECORD.json): `u16_2_current_001`, HEAD `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`, source953/evidence774/delta390. source archive SHA256 `b3d011f7d2ae5621346188958ba79f1519825778e2d02df599f959a99e94d6d5`.

[입구 비교](audit/independent001/ENTRY_CURRENTNESS.json), [membership·archive identity 확인](audit/independent001/ENTRY_MEMBERSHIP.json), [출구 비교](audit/independent001/EXIT_CURRENTNESS.json)에서 고정 파일의 크기·해시와 생산 소스 디렉터리 membership을 확인했다. 판정은 `current / unchanged`; 누락·불일치0이다. 감사 산출물과 mutable HANDOFF/현황 색인은 target의 선언된 제외 항목이다. 사용자 dirty 작업을 되돌리거나 이번 성과로 귀속하지 않았다.

[원시 증거 독립 검산](audit/independent001/EVIDENCE_RECHECK_002.json)은 BEFORE586/SOURCE953의 bytes·membership, 정확한390개 delta와 각 preimage/postimage 해시를 확인했다. 기존 [격리 정역복원](target/u16_2_current_001/RESTORE.json)을 이 일치성에 근거해 재사용했다. 복원 도구를 다시 실행하거나 사용자 tree에 patch를 적용하지 않았다.

## 수용 조건 판정

아래 소스 위치는 `core_enhanced/src/game/` 기준이다. 각 행의 원시 selector 대응은 위 EVIDENCE_RECHECK_002와 [구현 당시 coverage](COVERAGE.json)에 있으며, 이번 판정은 구현자의 `satisfied` 표시와 분리했다.

| 조건 | 직접 검토한 계약·소스와 증거 | 감사 판정 |
| --- | --- | --- |
| A01 | `world/facility_rooms.rs`의 기존 고정 metadata 소비와 세 유형 지원, 입장 전 thresholds/사본 조건. `u13_facility` 원시 및 이번 재실행으로 예약·배정·미지원 유형 거절 확인. | Pass |
| A02 | `world/relic_reward.rs::sample_relic_rewards`는 양수 가중 pool에서 제거하며 최대3종 선택. `u162_choice_distinct_shortage_zero_skip_and_once`가0/1/2/3/4 공급과 Normal1개 지급을 관찰. 보유/장착 중복 규칙으로 후보를 제외하지 않음. | Pass |
| A03 | 고정 node seed와 세션 후보 소유, 선택은 현재 UUID만 변경. 공통 Exit가 노드를 완료하며 grant 없음. 원시host choice와 이번 choice_complete에서 중복 수령 거절·IO 재시도 후1회 지급 확인. | Pass |
| A04 | `relic_investment_preview`가 다음 누적 임계치와 paid_dust 차액을 계산하고 전액 가능 여부를 선검사. 투자3/6/9, 부족2, 최고등급 거절의 상태 불변 assertion을 검토·재실행. | Pass |
| A05 | 다음 등급 공급 존재 검사 후 결제; 실패는 기존 후보·선택·자산 불변. view는 현재 rewards의 정의/효과와 다음 비용만 제공. 빈 상위 공급 시 기존 등급 수령까지 검사. | Pass |
| A06 | 성공한 투자마다 rewards 교체·selected UUID 제거·등급/누적 갱신. 이전 UUID 거절, 최고등급18 지불 후1개 지급, 이번 dust_complete002 및 dust_skip에서 완료·무환불 확인. | Pass |
| A07 | `fusion_input_grade`는 사본 존재·미장착·지원 등급 확인. 서로 다른 같은종류 사본 허용, 정상 Corrupt 명령으로 만든 입력 허용. source와 집중 검사, 원시 host_edges의 실제 공개 경로 확인. | Pass |
| A08 | `relic_reward.rs::fusion_probabilities`의 여섯 표를 정책 수작업 기대값과 대조. UUID 정렬과 입력별 seed 정규화, roll 범위/누적 비교상0% 배제 확인. 여섯 조합·역순 입력 검사 재실행. | Pass |
| A09 | 두 투입 종류를 제외한 모든 양수확률 등급 존재를 확정 전 검사. 확률표는 변경하지 않음. missing_positive_grade 사례에서 공개 사유·선택과 원본 불변·퇴장까지 확인. | Pass |
| A10 | `finish_basic_relic_reward`가 기존 EconomyGrantState·transaction 안에서 두 입력 제거, Normal1개 grant와 노드완료를 묶음. 원시 host 및 이번 fusion_complete에서 실제 IO 무변경/동일 명령 재시도/중복거절 확인. | Pass |
| A11 | 기존 미완료 checkpoint가 입장 전 자산·map을 보존; node seed와 정규화 입력이 재입장 결과를 고정. 같은process view/선택 유지, 원시 fresh-process 미완료 재입장과 입력역순 결과 일치 확인. | Pass |
| A12 | 원시 choice/dust/fusion 완료Continue·SavePoint의 자산·노드와 투자후퇴장 잔여15를 독립 비교. 이번 choice_checkpoint와 완결 host 재현 결과도 원본 domain 결과와 일치. | Pass |
| A13 | 정상 prepared host의 actual paired publish에서 OS21 주입 전 명령 자격 충족. 원시8회 및 이번5회 IO_FAILURE와 성공 재시도를 확인. intermediate investment가 발행하지 않을 수 있다는 기존 교정은 완료 경계 검증을 유지함. | Pass |
| A14 | LiveRon→typed policy validation의 참조·중복·양수가중치·u32합계·증가양수 비용·Reward category를 직접 검토. 정상1/작성오류10의 실제 패키저 exit와 변경입력 원문 확인. 빈/부족 공급은 정상 선택 검사로 확인. | Pass |
| A15 | metadata→map_content→Reward session→정상84명령 vocabulary/owner→공통 grant/완료→DTO/host 연결 확인. 공식 빈 공급은 사유/미이용 종료이며 일반 pool로 대체하지 않음. 외부 작성 fixture는 공식 콘텐츠 채택과 구별됨. | Pass |
| A16 | 정상/300후보·추가64사본 비용6process312창과 공유 lifecycle6process72연산의 raw 성공·allocator live bytes 정확 회수 확인. 기본10M authoring 거절과 별도64M 유한 측정 승인을 구별; native16MiB/배포 기본 불변. | Pass |
| A17 | 실제24파일 product/test/data delta와 공유 loader/reward/session/clone/store 영향을 검토. Core고유2196·통합72·선행native4/server31/socket1 근거와 실패 closure를 재집계. baseline·최종 archive와 정역복원 검증을 재사용. | Pass |

## 차원·통합·구조 검토

10개 차원 모두 applicable이며 N/A는 없다. success는 세 활동 지급·완료, invalid_input은 작성오류와 사본 자격, failure_rollback은 빈공급/부족/실제IO, retry_idempotency는 중복명령·재시도·Continue에 직접 대응한다. ordering_concurrency는 직렬 Core command owner의 두 UUID 정규화와 staged/paired commit을 확인했으며 별도의 병렬 게임 명령 지원을 주장하지 않는다. boundary_empty_max는0~3후보·등급상한·가중합계·0% 경계를, disconnect_restart_persistence는 미완료/완료/SavePoint를, live_data는 실제 RON packaging을, producer_consumer는 실제host의 공개 command/query를, legacy_fallback_absence는 기존 reward 흐름과 대체/재분배 부재를 검토했다.

5개 IVA(CHOICE/DUST/FUSION/STORE/AUTHOR)는 해당 수용 행들의 실제 공개 경로와 원시 증거를 갖는다. AG-CBRR-RUN과 WP-CBRR-VERIFY의 코드·검증·동결 책임을 확인했다. single_phase이므로 별도 GEX-30로 미룬 수용 조건은 없다.

새 상태는 기존 RewardSession의 optional inline owner와 최대3개의 기존 RewardOption이다. immutable policy는 validated layout, 사본·장착은 RelicInventory, 가루는 crafting resources, 완료·미완료 복원은 기존 node/checkpoint, durable publication은 기존 host가 소유한다. 후보 string/effect extent, clone/preflight, view allocation과 임시 카탈로그 색인의 수명을 실제 소스에서 확인했다. 일반/전투 Reward는 activity None의 기존 흐름을 유지한다.

Implementation Quality Guardrails를24파일 scoped diff와 실제 호출에서 검토했다. fixture identity에 따른 production 분기, silent normalization/recovery, consumer의 확률 재추론, 이중 자산권한, 미래 전용 layer, 임의 재시도·시간 완화, 미승인 대체 지급을 발견하지 못했다. legacy pool 시험 두 개의 metadata 수정은 원래 pool 검증 목적을 유지하며, 현재 공식 활동의 별도 시험을 남긴다. G11 successor는 source receipt/profile/trace/limits를 유지하면서 추가 layout default의 실제 semantic identity를 반영한다. 원시 실패를 삭제하거나 ignored로 바꾸지 않았다. runtime/data-loading 문서의 현재 공급·명령·조회·복원 의미도 실제 구현과 일치한다.

## 실행과 재사용

- 이번 실행: [focused001.log](audit/independent001/focused001.log), exit0, **17 passed / failed0 / ignored0 / filtered2113**. U16-2 8개와 시설9개. cargo build와 실제 test 종료를 구별했다.
- 이번 실행: [HOST_REPLAY.json](audit/independent001/HOST_REPLAY.json), **5 fresh processes / 27 commands / OS21 5회와 재시도**, exit0. 보관된 실제 preimage를 새 `/tmp/u162-independent-host-*` 사본에 복원해 동일한 고정 host·admission·지원 명령을 사용했다. choice_complete, dust_complete002, fusion_complete, choice_checkpoint, dust_skip의 최종 inventory/Run은 원시 결과와 정확히 같다. 사용자 저장 파일과 원본 artifact는 변경하지 않았다.
- 재사용: 전체 lib의2114 성공과 후속 교정9·통합72·기존 socket1을 합친 **고유2196**. 원시 실패10은 후속9개/소켓1로 모두 연결된다. 기존 ignored6은 PASS로 세지 않는다. 한 번의 all-targets green 주장이 아니다. 이번17은 이 수에 중복 가산하지 않는다.
- 재사용: 이전 native4/server31, 현재 정상/확장host23process107명령/IO8, 작성정상1/거절10, 비용과 archive/restore. 원시 command·rejection·IO_FAILURE·성공 재시도·결과 불변과 selector 실재를 기계 검산하고 source 영향 근거를 직접 검토했다.
- 선행 U16-1 independent001의 historical Complete/High를 보존했다. 실제 구현 전 baseline524의 product members가 그 고정 소스와 동일함을 확인했고 U16-2가 바꾼 공통 경계는 현재 회귀/host/비용 증거로 다시 판단했다. 별도의 선행 전면 재감사를 수행했다고 표기하지 않는다.
- 고정 host/worker/packager4개 receipt가 실제 binary 해시와 일치한다. prepared build 이후 차이는 cfg(test) G11 locator 한 문자열임을 exact preimage와 대조했다. 실제 승인은 복사한 binary fingerprint에 귀속하며 새 build에 승계하지 않는다.

감사 도구의 실패도 보존했다. 첫 host 시도는 출력 부모 디렉터리가 없어 host 초기 create_dir에서 exit1(실제 명령0)이었고, 부모를 준비한 host002에서만 유효 실행했다. 첫 evidence checker는 `ignored, 사유`의 suffix를 정규식이 인식하지 못해 실패했다. 원시6행과 summary를 대조해 parser를 교정했고 모든 원시 실패/교정 전 도구를 남겼다. 이 두 경우는 제품 finding이나 기능 PASS로 세지 않는다.

## 잔여·제외와 다음 경계

지원 제작 입력·정상 명령·실제IO 실패는 `범위 안 — 지원 경로`로 검토했다. 저장/checksum/메모리/내부상태의 임의변조만으로 발생하는 방어는 `범위 밖 — 외부 임의 변조`이며 이번 감사의 새 필수 시험이나 차단 사유로 추가하지 않았다. 별도 조사로 해결하지 못한 완료 차단 unknown은 없다.

공식 유물 catalog는 여전히 비어 있다. 대표 효과는 지원 외부pack에서 검증했으며 U21 콘텐츠 완성을 주장하지 않는다. 비용3/9/18·가중치1은 위임 초기값이다. 300후보 stress pack은 기본 authoring10M에서 거절되며,64M은 별도 유한 측정 설정이다. 비용은 해당 환경 표본이고 최종 기기 SLO가 아니다. private issuer key는 보관 대상이 아니므로 새 환경은 자기 issuer로 calibration/adoption해야 한다. 기존6개 ignored는 계속 한계로 남긴다.

이 판정은 **U16-2만 완료**한다. U16-3/4, 전체 U16, Unity, 공식 효과 확장과 최종 SLO는 포함하지 않는다. 사용자가 지정한 독립 감사 단계에서 종료하며 다음 구현을 자동 시작하지 않는다.

재현: 저장소 루트에서 `PATH=/home/blast/.cargo/bin:$PATH CARGO_HOME=/tmp/wp02-cargo-home CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u12-vault-cost-target cargo test -p game_core --lib --offline --locked -- u162_ u13_facility --test-threads=8`. 증거 검산은 `python3 core_enhanced/docs/goals/core_basic_relic_rewards/execution/single_goal/audit/independent001/recheck_evidence.py <새-json-파일명>`으로 실행한다. host 재현은 위 HOST_REPLAY의 argv와 고정 replay를 사용하되 store와 출력은 새 경로를 사용한다.
