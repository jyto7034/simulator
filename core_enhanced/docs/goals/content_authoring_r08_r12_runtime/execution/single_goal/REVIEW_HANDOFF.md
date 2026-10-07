# R08~R12 최종 통합 구현 인계

R08~R12 기능 구현과 자체 검증 자료를 최종 통합 target으로 동결했다. **55조건 중54 자체 검증, X09 환경 제한1 / 독립 감사 요청 대기**다. Unix socket 권한 제한이 남아 있으므로 무조건 전체 수용 PASS나 Complete/High를 선언하지 않는다. 정식 Single Goal의 후속 검토는 GEX-20이며 Master/GEX-30은 없다.

## 대상과 경계

source `bd89f9c42191217a333d648d35f495e802d13e6fb1420caa9c08da3deb731690` / 387 files. [manifest](target/r08_r12_integrated_001/MANIFEST.json), [외부 manifest 포인터](FINAL_TARGET.json), [수용표](COVERAGE.json), [currentness](evidence/VALIDATION_CURRENTNESS.json), [정역 복원](target/r08_r12_integrated_001/RESTORE_VERIFICATION.json). SOURCE_BEFORE/AFTER·RECORDS·REFERENCES archives와 추가/삭제를 포함한 FORWARD.patch가 target에 있다. isolated patch --fuzz=0의 정/역 membership와 bytes가 모두 일치했다. manifest 자기 자신과 이후 감사는 자기 해시에서 제외한다.

원 baseline은 선행 감사의 dirty/untracked source367개와 roadmap1개, 총368개를 실제 bytes로 보존했다. 기존 변경 전체를 이번 작업으로 귀속하지 않는다. 내부 WP source별 기록은 EXPERIMENTS.md와 unique evidence receipts/archives에 남겼고 과거 실패·동결 target을 덮어쓰지 않았다.

## 구현과 수용 대응

| 범위 | 기존 T 조건 | 상세 조건 |
| --- | --- | --- |
| WP08 | T17, T26 | A08-01, A08-02, A08-03, A08-04, A08-05, A08-06, A08-07 |
| WP09 | T19, T20, T29 | A09-01, A09-02, A09-03, A09-04, A09-05, A09-06, A09-07, A09-08, A09-09, A09-10 |
| WP10 | T08, T23, T26, T27 | A10-01, A10-02, A10-03, A10-04, A10-05, A10-06, A10-07, A10-08, A10-09 |
| WP11 | T09, T27 | A11-01, A11-02, A11-03, A11-04, A11-05, A11-06, A11-07, A11-08, A11-09 |
| WP12 | T10, T19, T27 | A12-01, A12-02, A12-03, A12-04, A12-05, A12-06, A12-07, A12-08, A12-09, A12-10 |

각 조건의 실제 selector·입력·oracle·default/feature receipt는 COVERAGE.json에 있다. X01~X10은 공개 보스·직원 실제 사본·복수 직원 보호·침식 교차·저장·rollback·조회·한도·전체 검사·byte 복원의 통합 조건이다.

- R08: opaque action handle, 명시 caster/target/facing, Released/EffectsSettled, 실패 route, timeout/state-exit 취소와 이미 방출한 payload 보존. 외부 보스 RON의 projectile→area 후 다음 marker action 시각으로 완료 경계를 검증했다.
- R09: exact active fragment 자격/공명/cast-start, 대상 집합 침식 mandatory death와 지정 부가 효과 억제, unpaused host freeze/잔여 event 재개, 직원 대가·Breakdown 및 node 회복/만료/asset 회수. 보스 flow+channel+공유 보호막 공존 중 정상 World 결과/save/fresh Core까지 검증했다.
- R10: Sequence/Parallel(All)/Branch/Repeat/Wait/Action의 유한 compiled flow, 실제 HP 결과 분기, authored 주소와 occurrence, target/delivery kernel, native 한도·overflow·취소·원자 재시도. 기존 legacy step도 동일 compiled 계획을 사용한다.
- R11: source별 가역 modifier와 family membership, 실제 periodic basic·작업·태세·시전 제한 channel, exact activation 재입력의 RemainingTime/Goodbye, 원무기 주기·속성·identity를 유지하는 area/projectile overlay. aura 만료 후 이미 발사한 탄환의120HP snapshot과 중지 후 basic 재개를 직접 검증했다.
- R12: 개인→공유 shield, capacity/잔량/재시전/expiry, HP1/일회 생존, 별도 HP cost/recharge, Slow/Root/Taunt/Pull, 첫 직접 strike parry/OnHit 억제·counter·miss 경감, 실제 흑조 사본 passive 경감/반사. 기본 흡혈은 strike의 실제 HP 합산 후 내림/cap 한 번이고 보호 흡수·과잉 피해·Goodbye·반사는 제외한다.

## 검증 결과와 한계

최종003 default/test-fixtures all-target에서 Unix socket fixture를 정확히 제외한 두 실행은 각각1676 passed/0 failed다. 원명령은 각각 library1619 passed/1 failed로 종료했고 실패는 `game::data::content_pack::snapshot::tests::unix::invalid_utf8_directories_and_socket_inputs_return_errors`의 OS `Operation not permitted`다. 제품 시험을 skip하도록 바꾸지 않았다. 독립 감사 또는 허용 환경에서 이1개를 다시 확인해야 한다.

check/all-target, event_log_checks, release bins, clippy, fmt PASS. Clippy는 선행 명령의 never_loop/eq_op allow만 유지하며 경고0이라는 주장은 하지 않는다. 공개 release CLI는 R06/R07 Snapshot·Live/R10 Snapshot·Live/R12 Hello·보호·Released·EffectsSettled 총9팩의 validate/describe/explain/simulate off/on45명령이 모두 성공했다. 모든 public_result와 마지막4 amount 문서가 trace off/on에서 일치한다. 과거 R06/R07 팩은 필수 corrosion 정책 누락을 정확히 거부한다. 과거 원본을 보존한 별도 복사본에 현재 run policy의 corrosion 한 줄만 추가했으며 scenario/skill/copy/recipe/나머지 정책은 byte 단위 동일하다. [변경 증명](evidence/CLI_PREDECESSOR_MIGRATION_001.json)에 연결한다. 원자료는 그대로 보존하고 기존 OBS001의 TraumaRecoveryPreview 직원 표시 순서만 UUID 정렬했다. [CLI summary](evidence/CLI_FINAL_SUMMARY_002.json).

PC36은576행/36조건에서31success·5typed refusal, HP/fact/완료/error-code 차이0이다. [PC summary](evidence/PC_SUMMARY_005.json). 첫 측정에서 일반 fact/source의 빈 신규 필드가 한도 초과를 유발한 것을 발견해 희소 상세/출처 저장으로 교정했다. 실제 할당 전 admission·retained·CloneShape를 유지했고 workload/cap을 축소하지 않았다. 중간 실패는 final001~003에 남겼다.

로그는 신규 event variant 때문에 형식32다. 이전 동결 source의 원 workload에서 실제65개 로그를 추출해 P01 원 hash와 모두 대조했다. 그 byte의 leading version31만32로 바꾼 [별도 기준](evidence/P01_LOG_V32_BASELINE_001.json)과 [증명](evidence/LOG_VERSION_MIGRATION_001.json)을 사용한다. event payload·순서·bytes 길이·반복·원 cap은 변경하지 않았다. PC 원본 v32 hash와 v31 envelope hash를 함께 보존하며 로그의 그 외 byte 차이는 허용하지 않는다. 실제 R05 runner source는 변경하지 않았다.

R05 event-heavy peak263431 bytes, concurrent peak9372846 bytes(cap9030188), 초과342658만 기존 합의 경고다. allocation operations/retained/event peak/repeatability/parity hard guard 실패0. [R05 summary](evidence/R05_SUMMARY_005.json). latency는 이 환경의 관찰값이며 비교 가능한 성능 보증으로 승격하지 않는다.

## R06/R07 영향과 저장

[선행 감사 종료](../../reviews/r06_r07_20260930_001/AUDIT_CLOSURE.json)의 Complete/High는 source714195327ec0df7facc60d1aafe721e6e26c1296494b39cff593232c45365e6f에 한정된다. 현재 공유 action/flow/resource/damage/save 변경까지 과거 판정을 재사용하지 않는다. [현재 영향표](R06_R07_IMPACT_COVERAGE.json)의 R06 31 acceptance/21 integration 및 R07 F01~F12 selectors를 실제 최종 소스에서 다시 실행했다. 새 독립 영향 감사는 대기다.

현재 durable 형식 V9는 직원 BreakdownCounter를 필수로 보존한다. V7/V8 자동 로드를 거부하고 원 identity/checksum을 확인한 명시 변환으로 새 bytes를 반환한다. 원 V7 bytes/hash와 predecessor archives를 보존했다. V8 legacy-shape fixture는 생성한 시험 데이터이며 역사적 원본이라고 주장하지 않는다. exact owned copy/form/equipment/remaining node/result를 새 Core 복원에서 검증했다.

## 정책·작성 제약·후속 감사

[기술 판단](EXPERIMENT_NOTES.md), WP10/11/12_DETAIL, [정식 P2G](../../P2G_POLICY_AND_REVIEW.json)를 따른다. trauma80/freeze1000ms/Breakdown2node와 테스트 수치는 위임 잠정 정책 또는 fixture 값이며 사용자 확정 밸런스로 소급하지 않는다. Flow의 finite depth/node/repeat bound, channel/self aura, overlay 지원 delivery, parry의 즉시 counter leaf와 명시 attachment, 비용의 leave-one 기본/명시 lethal opt-in은 publication/runtime에서 일관되게 적용한다. 허용되지 않는 조합은 진단하고 암묵적 fallback을 하지 않는다.

이번 primitive는 공식 콘텐츠 전체 저작·밸런스 완료를 뜻하지 않는다. 일반 함수/재귀, R13 전체, 미래 원정/임대·귀환, GUI·Unity/game_server 확장, stress/metrics/20분 시험은 범위 밖이다. 현재 attempt·node 완료·save 경계는 미래 원정 전환 시 영향 감사가 필요하다.

후속 GEX-20은55조건과 R06/R07 공유 경계, 새 source 비용 계산·원자성, 버전만 이행한 성능 기준, X09 환경 시험을 확인해야 한다. 구현자는 독립 PASS를 발행하지 않았고 자동 감사나 다음 로드맵 단계로 진행하지 않는다.
