# R05 F-R05-001 교정 pass

2026-09-12 · GEX10 active · R05 전체 Not Complete.

[독립 감사001](COMPLETION_AUDIT_001.md)이 생성자에서 charged resource validation 오류를 expect로 처리하는 F-R05-001을 확인했다. Root는 전체 보고서와 봉인13출력, 종료12,969개 currentness를 확인한 뒤 감사 종료를 인수했다. 감사 중 source 변경은 없다.

Owner는 기존 GS-CARL-P2G50-001 / GOAL-CARL-01 / PHASE-CARL-01, AG-CARL-01의 WP03 publication과 WP04 검증이다. 승인된 R05 안의 기술 교정이며 정책/Phase/Handoff 범위는 변하지 않는다.

## 교정과 검증 계획

- 먼저 새 durable regression을 추가하고 교정 전 production source에서 실제 panic을 재현한다. 생성자 catalog/reference/cancellation 경계와 공개 로더 FinalValidation→GeneratedSetup 경계를 구별한다.
- Private GameDataBase constructor와 validated publication을 fallible로 전환해 원래 charge/strict validation/순서를 보존하며 typed 오류를 전파한다. 광범위 catch, 한도 완화, 검증 생략으로 바꾸지 않는다.
- 동일 regression에서 typed code/phase/site, 기존 generation/receipt/input 보존과 정상 retry를 확인한다. 직접 영향 loader/packaging/public tests 및 필요한 final baseline/비용을 현재 source에서 확인하고, runtime 의미 불변의 재사용은 scoped source 근거와 구별해 기록한다.
- old target002와 102–116, R01–R04 actual reports, R05 Not Complete와 모든 실패는 보존한다. 교정 후 source attribution/현재성/선행 영향의 successor를 고정하고 R01–R04 필요 감사 및 R05 fresh 감사를 복원한다.
- 완료 판정은 아직 없으며 R06 이상은 시작하지 않는다.

## 실제 기록

- 117: Cargo package 이름을 core_enhanced로 잘못 지정해 package resolution 실패, 실행 시험0.
- 118: regression의 unwrap_err가 GameDataBase: Debug를 요구해 test-only compile 실패, 실행 시험0. .err().expect로 시험 assertion만 교정했다. production 변경은 아직 없다.
- 119: 같은 생성자 regression의 교정 전 실제 실행 진행. 위 두 실패는 재현 성공이나 gameplay 실패로 세지 않는다.

- [119](evidence/resource_constructor_reproduction_119.json): 1 test 실행, 예상된 교정 전 FAIL. 실제 resource catalog expect panic을 stderr로 확인했다.
- [120](evidence/resource_publication_reproduction_120.json): 실제 public loader의 1 test 실행, 예상된 교정 전 FAIL. FinalValidation reference scan의 expect panic을 확인했다. 원래 공식 empty catalog에서도 발생한다.
- Production 교정: private GameDataBase::new/from_validated_live_catalogs가 Result를 반환하고 source relocation을 거쳐 publication까지 전달한다. 기존 strict validations/charge/순서와 public API signature는 유지한다. cfg fixture builder만 기존 fixture-assert 역할로 반환 오류를 expect하며 production loader는 이를 호출하지 않는다.
- [121](evidence/resource_constructor_correction_121_input.json): 생성자 catalog/reference와 control-entry 이후 cancellation 경계의 교정 후 시험 진행. 정상 retry를 함께 확인한다.

- [121 결과](evidence/resource_constructor_correction_121.json): 1 test/3경계 PASS. catalog work1/request2, reference_step work1/request2는 authoring_work_limit/FinalValidation, scope-entry 이후 cancellation은 authoring_cancelled/resource_catalog_validation이며 정상 로드가 복구됐다.
- [122 결과](evidence/resource_publication_correction_122.json): 1 test/empty와 nonempty2 실제 external pack PASS. 공개 예산 이분 탐색으로 empty521937→requested521938, nonempty522601→requested522602에서 FinalValidation/resource_reference_effect typed refusal을 반복 확인했다. 한 단위 높은 예산은 GeneratedSetup까지 진행했다. 기존 generation/receipt/source와 정상 retry·일반 loader가 보존됐다. 코드의 charge 수를 복제하거나 test-only hook을 넣지 않았다.
- [123](evidence/resource_core_alltargets_correction_123_input.json): local-socket 포함 기존 필수 all-targets 회귀를 승인된 실행 경계에서 진행 중.

- [123 결과](evidence/resource_core_alltargets_correction_123.json): 실제1412 passed/0failed/0ignored. lib1360 + integration4/4/25/15/4, 0-test binary는 미집계. source change0.
- 124–137 validation batch 진입: old default binary8개를 exact SHA로 별도 보존하고 release/public builds, 원래 guards, typed15/legacy13/diagnostic10/negative13/limits8/order 및 plain/allocator 비용을 교정 source에서 측정한다. `/tmp/r05_correction_validation_001.py`는 실패 시 중단하며 positive audit를 만들지 않는다.

## 원래 성능 guard126 실패와 추가 교정

- 124/125 default release/public build PASS, source change0. [126](evidence/resource_legacy_guards_correction_126.json)은 public plain/allocator·kernel plain·native budget를 통과했으나 kernel allocator concurrent peak가9,042,341 > 원래cap9,030,188로 FAIL했다. batch는 여기서 중단했으며127–137은 아직 실행하지 않았다.
- 이전102와126의 concurrent samples는 allocation_count364268/deallocation_count364202/reallocation_count5900 및 retained4,196,928이 같다. 동시 peak는 실행 중 공존량에 따라 달라진다. 이를 성공 재시도 선택이나 cap 완화 근거로 쓰지 않고 실제 footprint를 더 줄인다.
- Source 조사에서 RuntimeFactSource.strike는 serde(skip)·crate-private Option<StrikeOrdinal>이며 producer assignment와 초기화/observation clear 외 production read가 전혀 없다. 실제 admitted Strike의 progress/ordinal·StrikeScope.targets와 DamageFact.strike가 이미 owner/consumer다. 이 중복 transient annotation과 전용 미사용 ordinal accessor만 제거해 매 source 복제 footprint를 줄인다.
- 기존 Strike-before-formula/overflow/segment-boundary regression은 유지하고 transient 단일 필드 확인을 전체 RuntimeFactSource 복원 equality로 강화한다. R04 사건 identity·serialization·formula-before/after 순서와 resource authority를 바꾸지 않는다. 수정 후 focused test·original guards와 영향 전체 회귀/공개/비용을 새 기록으로 복원한다.
- INPUT002/123/124/125/126은 첫 생성자 교정 단계의 실제 증거로 보존한다. 현재 positive target은 없고 추가 교정 뒤 새 source snapshot이 필요하다.

- [127](evidence/resource_source_compaction_127.json): 기존 runtime_strike_admission_precedes_formula_limits_and_rolls_back 1 test PASS. ordinal overflow·formula capacity refusal·actual last ordinal·2mutations·전체 source equality/StrikeScope cleanup 검증을 유지했다. production 소비자 없이 중복된 annotation만 제거했다.
- 이번 추가 교정은 battle/core/commands, damage_runtime, queue, runtime_facts와 battle/runtime_facts의5파일이다. damage_runtime은 원래 R05 변경49목록 밖의 직접 producer이므로 전체 dirty-baseline 귀속은50modified+16added로 갱신해야 한다. 기존 R04 Strike producer의 해당 단일 write 제거이며 신규 기능·정책 의미 변경은 아니다.
- [128](evidence/resource_legacy_guards_compact_128_input.json): 원래 public/kernel plain·allocator/native guards 전체를 새로운 stable candidate에서 실행 중.126 실패와 첫 교정 snapshot002는 그대로 보존한다.

- [128 결과](evidence/resource_legacy_guards_compact_128.json): original public/kernel plain·allocator/native guard 전부 PASS, concurrent5samples 최고9,006,984 <9,030,188.
- [129 고정3회 확인](evidence/CONCURRENT_CONFIRMATION_001/RESULT.json): 실행 전3회로 정한 동일 immutable kernel_alloc의15추가samples 모두 PASS. 회별max8,992,449/9,007,529/8,989,671,128을 합친20samples max9,007,529. 모든 결과·명령을 보존하며 최저/성공 실행 선택이 아니다. 관찰 입력의 성능 근거이며 임의 OS scheduling의 수학적 상한이라고 주장하지 않는다.
- `/root/r05_source_compaction_read`는 §8의 read-only 구현 보조다. archive002와 exact delta·derived equality·source-default allocation elision·observation Arc sharing·Strike authority·retained/CloneShape를 검토해 추가결함을 찾지 않았다. DamageSourceSnapshot은 fact_source를 equality에서 제외하고 QueuedEvent는 event만 비교한다. 관찰은 원래 strike를 지운 뒤 source sharing에 사용했다. 중복 annotation만 있던 source의 allocation elision/예산 절감은 가능하지만 남은 metadata/capability를 지우지 않는다. public derived Debug/Eq에서 내부 annotation이 제거되는 표현 차이는 직렬화 계약 변경이 아니다. 이 보조는 최종 독립 감사가 아니며 source 수정/시험 실행 없음.
- [130](evidence/resource_core_alltargets_final_130_input.json): 최종 compaction candidate의 전체 core 회귀 실행 중. 이후131–143의 default release/public/consumer/비용 현재성, 새 source003·수용 map·exact target/필수 감사가 남는다.

## 최종 교정 검증 종료와 감사 인계

- [130 결과](evidence/resource_core_alltargets_final_130.json): source003에서 실제1412 passed/0failed/0ignored, lib1360 + integration4/4/25/15/4. 두 새 regression과 강화한 기존 Strike test를 포함하며 source change0이다.
- [최종 validation](evidence/CORRECTION_VALIDATION_001.json):131core release/132공개 release builds,133typed15cases,134기존소비13groups,135generated diagnostics10commands,136negative13cases,137limits8cases,138order·actual atomic shortage 모두 PASS. 모든 명령의 원시 log SHA와 source 무변경을 확인했다.
- [최종 비용003](evidence/COST_SUMMARY_003.json):139/140plain 합계1025samples,142/143allocator 합계615samples. typed read99회 무할당, 입력별 독립 generation load/drop12쌍을 확인했다. 실제 requested bytes와 별도 plain timing이며 RSS나 Complete 즉시 전체 회수의 증거로 쓰지 않는다.
- 256자원 일반 long-clock은 unchanged clone limit의 atomic refusal을 유지한다. 별도 명시된 bounded-clock scenario는16steps 실제 Victory까지 완료했다. 원래 호출 성공이나 자동 분할이라고 주장하지 않는다.
- [source003](evidence/IMPLEMENTATION_INPUT_003.json)·[archive003](evidence/SOURCE_AFTER_003.tar.gz)·[귀속003](evidence/ATTRIBUTION_003.json)은349현재입력, 기존 dirty기준50modified+16added를 고정한다. 초기 input001에서8파일 교정,341불변이다. SOURCE_AFTER003 SHA256은 aed28f87349fe623b1a811f32efdc5ea7f8f2717b9b3f58e43309f0d869b56d0이다.
- GEX10 구현·검증은 여기서 종료하며 source/test/fixture/binary mutation을 멈춘다. original33 acceptance·12IVA·10dimensions·C01–08·I01–08 및 R05 card/AT05의 현재 증거 successor를 별도로 작성한다. 새 currentness/정확한 targets, R01→R02→R03→R04→R05 독립 감사와 parent 전체 인수는 필수 미완료다. 최초 Not Complete,119/120panic,126guard failure와 중간 snapshot은 모두 보존한다.
