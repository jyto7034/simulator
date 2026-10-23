# U11 완료 인수와 후속 작업

기록 시각: 2026-10-12T08:48:58.489356+09:00. Root는 비참여 독립 감사자의 판정과 실제 출구 현재성을 인수했다. 제품·시험 변경이나 추가 실행은 수행하지 않았다.

## 독립 완료

GS-CMVA-001 / GOAL-CMVA-01은 **Complete / High / 열린 finding 0**이다. 감사자는 이번 Goal의 계획·제품·시험·실행에 참여하지 않은 `/root/u11_independent_audit001`(GPT-6 Astra high)다.

- [독립 감사 종료](../../../../goals/core_mod_validation_apply/post_audit/u11_current_001_pass001/AUDIT_CLOSURE.json): SHA-256 `d68ee5bcb871b05dee9660227a98879aa4a6f38b0d335c0b6eb9d4aba732b7d2`.
- [전체 감사 보고서](../../../../goals/core_mod_validation_apply/post_audit/u11_current_001_pass001/COMPLETION_AUDIT.md) 및 [출구 현재성](../../../../goals/core_mod_validation_apply/post_audit/u11_current_001_pass001/INTEGRITY_EXIT.json).
- [정확한 동결 대상](../../../../goals/core_mod_validation_apply/execution/single_goal/target/u11_current_001/MANIFEST.json): SHA-256 `fb6d1175c268e6ca87a0288d53fb57e509537757c5ef40fa986cca57a996c76c`.

15개 정책 요소의 30개 수용 문장, 20개 통합 seam, 10개 필수 검증 차원, 3개 실행 node 및 선행 316개 행을 독립 검토했다. 출구에서 복원 범위 4,732개 파일·기록 13,860개·대상 산출물 15개의 bytes/hash 불일치는 0이다. Runtime scanner의 4,678개 파일과 복원 합집합 4,732개는 서로 다른 범위이며, runtime source는 `638eab584d31b2a98890a44bd9292126cf6cb1f0c03a812dd36060d535b89746`이다.

## 실제 완료한 계약

외부 mod/pack·의존성·최종 조합을 불변 후보로 해석하고, 정상 실험실의 실제 기능 coverage와 plain/native 계측·교정 기준을 승인 권한으로 연결한다. 주체·revision·source/build/profile/suite/criterion에 결속한 cache는 정상 local/server Apply가 소비하며, 진행 중 A와 향후 B 세대를 분리한다. 필요한 승인 구성이 없는 저장은 writer-capable 복구 전에 원본을 보호하고, 구성을 복구하면 같은 원본을 Continue한다.

실제 public launcher/server, cache restart·취소·위조·stale 거절, B-after-C 새 저장·fresh reader, accepted-v2 crash 지점, 정상 actor의 다음 HostTick expression fault와 첫 원인·다른 actor 격리까지 감사 근거로 인수했다. 24개 named 그룹의 선택 PASS 32개와 별도 관측·관리·공개 command 그룹 49개를 구분한다. 과거 전체 시험의 재실행을 주장하지 않는다.

[동결·격리 복원 실제 실행](../u11_freeze_20261012_002/FREEZE_COMMAND_RESULT_001.json)은 exit0/reaped, 1,581.29초다. [복원 검증](../../../../goals/core_mod_validation_apply/execution/single_goal/target/u11_current_001/RESTORE_VERIFICATION.json)은 원본 559개에서 최종 합집합으로의 정방향·역방향 파일 구성과 bytes 일치를 확인했다. 작업공간에 역패치를 적용하지 않았다. 첫 동결 실패와 수정 전 도구도 보존했다.

## 유지하는 한계와 근거

개발용 기준 `cmva-dev-cadence-v3-20261012`는 phase별 6,000,000,000ns·heap 1,342,177,280bytes·RSS 1,610,612,736bytes·native requested 233,000,000이다. Default gameplay safety cap 변경이나 모든 환경의 성능 보장이 아니다. 실제 resource-only base native 최대 230,987,820과 조합 실패 234,667,723의 범위를 보존한다. 500/1000 직원 workload는 로스터·자산 확장과 실제 8명 배치이며 500/1000명 동시 전투가 아니다.

U10의 원 PC strict 실패·R05 peak/환경 한계와 U11의 실패·미완료·중단·0실행 원본을 유지한다. Generated 문서의 감사 전 pending 표기는 동결 당시 이력이며, 실제 복원과 이번 외부 독립 종료가 해당 미래 gate를 닫는다. 동결 PLAN/NOTES/EXPERIMENTS/STATE·소스·시험·증거·target·감사 원본은 수정하지 않는다.

## 다음 승인 작업

기존 연속 실행 지시에 따라 U12 튜토리얼·원정 시작/귀환·개인 상태 수명의 정책 및 현재 코드 기반 P2G로 이어간다. 고정 3층 튜토리얼 포팅, 첫 실행 원정, 선발·대여·귀환/실패·첫 귀환 story 상태, 신뢰 제거와 기억/최근 반응/고정 성격 보존을 다룬다. U12 구현·검증은 아직 시작하지 않았다. 전체 시설 지도 U13, Unity U26-B, 개별 최종보스 고유 구현 제외를 유지한다. 단계 전환을 위한 추가 승인 대기는 없다.
