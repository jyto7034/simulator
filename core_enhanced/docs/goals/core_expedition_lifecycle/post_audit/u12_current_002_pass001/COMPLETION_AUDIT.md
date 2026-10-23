# U12 독립 완료 감사 — u12_current_002 / pass001

**판정: Not Complete. 확정 completion-blocking finding: 1건(F-U12-001).**

정상 승인 저장 경로가 미지원 profile 필드를 정상 backup 복구로 우회하고 후속 명령에서 원본을 덮어쓴다. `Complete / High / open0`를 부여하지 않는다. 결함의 재현과 원인 판단에 대한 신뢰도는 높지만, 이를 Goal 전체의 High 완료 판정으로 표시하지 않는다.

## 권한과 감사 분리

사용자의 “너가 독립 감사를 수행해줘” 요청으로 GS-CEXL-001 / GOAL-CEXL-01의 direct GEX-20 stage-run을 수행했다. [goal_execution_pipeline(1).md](../../../../goal_execution_pipeline(1).md)의 §6.7, GEX-20과 Goal의 reviewer requirement를 적용했다. 이 대화의 감사자는 해당 Goal의 계획·제품·시험 작성에 참여하지 않았고, 진입 전 작업은 기록 확인과 HANDOFF/이력 갱신이었다. 모델 정체성을 별도 runtime 검증했다고 주장하지 않는다. 자격·권한·범위는 [ENTRY.json](ENTRY.json)에 남겼다.

감사 중 제품·데이터·시험·동결 target은 수정하지 않았다. 신규 출력은 이 감사 디렉터리에만 귀속된다. 재현 명령은 보존된 실행파일과 정상 admission을 사용해 감사 전용 저장 복사본에 실행했다. 원래 저장·issuer 키를 수정하거나 복사해 승인을 우회하지 않았다. 기존 유효 issuer 문맥을 정상 API로 다시 열었으며 신규 calibration을 수행했다고 주장하지 않는다.

## F-U12-001 — 미지원 profile 필드가 backup 복구를 통과해 원본을 잃음

- 중요도: P1, U12 완료 차단.
- 직접 위반: `A-13-2`의 미지원 원본 보호 및 Goal의 strict schema/store/compatibility 계약. `A-PS02-2`의 전체 미지원 입력 보호 범위도 이 finding을 해소하기 전 닫지 않는다.
- 명시 계약: [core_runtime_contract.md](../../../../core_runtime_contract.md)의 V4 profile/unknown-field 원본 보호 문장. 파일의 1004행은 “이전 envelope/unknown fields는 원본 보호 대상이다”라고 명시한다.
- correction owner: U12 GOAL-CEXL-01 GEX-10의 profile reader/원본 보호 및 paired publication 소유자. U11의 과거 verdict를 이 현재 U12 결함의 대체 승인으로 쓰지 않는다.

재현에는 동결된 `normal_return_cycle002/evidence/second_departed/save`의 정상 profile/current/backup/active 4파일을 복사했다. current와 backup을 따로 시험하면서 한 profile 사본에 제거된 `standard_tutorial_completed: true`만 추가하고 unsigned JSON의 SHA-256을 다시 계산했다. 원래 정상 checksum의 동일 계산 일치도 먼저 확인했다. 다른 사본, active binding과 required configuration은 동일하게 유지했다.

| 감사 전용 사례 | 정상 Continue | 후속 AbandonRun | 미지원 원본 보존 |
| --- | --- | --- | --- |
| 정상 4파일 대조 | exit 0, 파일 불변 | 실행하지 않음 | 해당 없음 |
| current profile에 제거 필드, 정상 backup | exit 0, 파일 불변 | exit 0 | 실패: current/backup/profile/active 모두 새 정산으로 변경 |
| backup profile에 제거 필드, 정상 current | exit 0, 파일 불변 | exit 0 | 실패: current/backup/profile/active 모두 새 정산으로 변경 |

두 음성 사례 모두 admission을 정상 통과했고 실제 Continue와 `AbandonRun`에 도달했다. 단순 checksum 오류나 승인 부재로 일찍 거절된 시험이 아니다. 예상은 미지원 schema의 명시 거절과 원본 불변이다. 실제는 정상 사본을 이용한 묵시적 복구 후 unsupported 사본 덮어쓰기다. 이번 복사본의 vault 손실량을 일반화하지 않으며, 확인한 결함은 미지원 저장을 거절하지 않고 원본을 상실시킨다는 것이다.

원인 경로는 다음과 같다.

1. `game_server/src/persistence/player_profile_store.rs:135`의 strict deserialize는 unknown field를 `InvalidEnvelope`로 분류한다.
2. `player_profile_store/application_intent.rs:139–148`의 approved preflight는 `UnsupportedVersion`만 즉시 반환하고 다른 decode 오류를 무시한다. 정상 사본 하나가 있으면 통과한다.
3. 같은 파일 `protect_profile_originals:193–198`도 이 오류를 보호 대상으로 반환하지 않는다.
4. `player_profile_store.rs:368–372`는 current 오류를 정상 backup으로 복구한다. backup만 미지원이면 정상 current를 그대로 읽는다. 이후 정상 paired publication이 두 profile 원본을 덮어쓸 수 있다.

기존 `u12_normal_legacy_profile_active_and_journal_headers_protect_originals`는 이전 version header를 시험한다. duplicate-vault 검사는 **두 profile 사본 모두** 잘못된 입력으로 바꾼다. 이 성공 근거들은 “한 사본만 미지원 + 다른 사본 정상” 경계를 검증하지 않으므로 이번 반례를 닫지 못한다.

정확 argv·실행파일/config/seed hash·출력·전후 4파일 hash와 보존한 미지원 원본은 [original_protection_probe/RESULT.json](original_protection_probe/RESULT.json) 및 각 사례의 `*.receipt.json`, `unsupported_original.json`에 있다. 실행 스크립트는 [probe_original_protection.py](probe_original_protection.py)다. 동일 출력 경로에는 재실행하지 않고 후속 재현은 새 evidence/store 경로를 사용한다.

## 검토한 범위와 기존 근거

고정된 15 policy element의 38수용 문장을 freeze 원문과 대조했고, 19 integration seam·10 validation dimension·3 execution node의 추적을 확인했다. 원문 일치와 191개 선택된 artifact 참조, 33개 기존 실행의 named test/raw receipt 검사는 [MACHINE_EVIDENCE.json](MACHINE_EVIDENCE.json)에 있다. build/관리명령/게임 진행/계량을 test PASS로 합산하지 않았다. 이전 실패 batch의 성공 case만 분리했고 실패 원문은 유지했다.

실제 검토는 원정 identity/대여/현재 사본 귀환·종료, profile와 knowledge의 서로 다른 수명, paid recruitment, entry receipt/Gate/소진 선택, 실제 treatment/weapon/fragment/HQ recovery eligibility, Continue/SavePoint의 동일 instance와 최신 profile 보존, profile→active CAS 및 accepted journal, strict codec/legacy rejection, 승인 저장 producer/consumer와 DTO/문서를 포함한다. U11 독립 감사의 exact closure와 현재 U12 영향표를 읽었으며 U03/U04/U07M/U11의 관련 검증을 구분했다. 과거 감사 결론을 현재 Goal의 판정으로 복사하지 않았다.

이전 실행과 최종 source의 차이는 [EVIDENCE_SOURCE_DELTAS.json](EVIDENCE_SOURCE_DELTAS.json), 원본 locator는 [DELTA_PREIMAGE_LOCATORS.json](DELTA_PREIMAGE_LOCATORS.json)에 보존했다. 다수의 마지막 변경은 test/oracle 또는 공개 진행기 분기였고, 현재 persistence body의 반례는 최종 default-release 실행파일로 새로 실행했다.

**이번은 결함을 확인한 음성 종료다.** [CONDITION_REVIEW.json](CONDITION_REVIEW.json)은 모든 수용·차원·통합·node를 추적하되, 나머지 항목을 일괄 PASS나 독립 완료로 선언하지 않는다. 전체 최종 비용 표본/일부 미완료 source-oracle 세부 검토를 포함한 긍정 종료 심사는 correction 뒤 재감사에서 닫아야 한다. 확정 finding 1건이라는 수치는 다른 결함이 없다는 보증이 아니다.

## target 현재성과 복원

- target: `execution/single_goal/target/u12_current_002/MANIFEST.json`
- MANIFEST SHA-256: `3eda79477ee69d83442721e6c5f591240eee51e6677f9ebea9080cacd8351ecd`
- source digest: `e1b51302c484c16dcc1837c60ac385398460bb854f2c0708996b5b0b3fc6371e`

[INTEGRITY_ENTRY.json](INTEGRITY_ENTRY.json)에서 current source 4,728·기록 3,692·참조 39·외부 artifact 41·target artifact 14의 bytes/hash와 재구성한 source membership가 일치했다. [INTEGRITY_ARCHIVES.json](INTEGRITY_ARCHIVES.json)에서는 before 4,693/after 4,728/records 3,692/references 39개 tar member의 실제 bytes·membership를 대조했다. 기존 정·역 복원 실행의 도구와 raw receipt도 검토했다. 감사자가 작업 tree에 패치를 적용하거나 freezer를 재실행하지 않았다. 종료 대조 [INTEGRITY_EXIT.json](INTEGRITY_EXIT.json)도 동일 범위 불일치0으로 통과했으며, 최종 결속은 [AUDIT_CLOSURE.json](AUDIT_CLOSURE.json)에 기록한다.

## 반환 경계

GEX-20을 `Not Complete`로 닫고 **U12 GEX-10 교정**으로 반환한다. 미지원 schema 거절/원본 보호와 허용된 손상 backup 복구를 함께 검증하고, current·backup 각각의 미지원 사본 + 정상 상대 사본, 정상 양성, fresh reopen·writer/재시도 원본 불변, paired profile/active 원자성 및 관련 U04/U11 경계를 재검증해야 한다. 필요한 실제 변경 범위로 successor target과 영향 근거를 고정한 뒤 fresh 재감사를 수행한다. 현재 frozen target/과거 실패를 고쳐 쓰지 않는다.

이번 감사 요청에서 제품 교정을 수행하지 않았고 U12를 완료 처리하지 않는다. U13에는 착수하지 않는다.
