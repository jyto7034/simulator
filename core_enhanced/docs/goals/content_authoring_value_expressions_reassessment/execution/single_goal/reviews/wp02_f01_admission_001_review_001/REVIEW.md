# WP02 F01 admission 001 단계 재검토

판정: **CORRECTION_REQUIRED** — F01의 Live frame 메모리 선청구에 확인된 잔여 결함 1건.

검토일: 2026-09-26. 대상은 `corrections/wp02_f01_f03_cap_001/target/f01_admission_001/MANIFEST.json`, SHA-256 `8ac6a2c6197ce19e1e841d52128758b14b8079762a11536620131884fa344370`이다. F01 부분 교정만 검토하며, F02/F03/CAP-01/CAP-02의 종료나 다음 WP 진입을 승인하지 않는다.

## F01-R01 — Live frame의 첫 BTreeMap 할당보다 작은 크기를 허용함

우선순위: P1. 위치: `core_enhanced/src/game/battle/core/value_expr_runtime.rs:591`의 크기 계산 및 606행의 admission.

Live frame은 참조마다 `size_of::<(ValueRef, TypedInteger)>() + 256 + dynamic_bytes * 4`를 청구한 후 `frame.insert`를 수행한다. 그러나 BTreeMap은 첫 삽입에도 여러 항목을 수용하는 노드를 할당한다. 이 환경에서 문자열 없는 Source.Attack Read 하나는 352 bytes만 청구하지만 첫 노드 할당은 1,072 bytes다.

기존 owner extent를 먼저 seed하고 잔여 retained 예산을 정확히 청구 예정량으로 설정한 독립 allocator probe 두 개에서 다음을 재현했다.

| 경계 | 허용한 신규 할당량 | 실제 allocation 요청 합계 | 최대 단일 요청 | 결과 |
| --- | ---: | ---: | ---: | --- |
| Live frame 생성만 | 352 | 1,072 | 1,072 | frame 생성 성공 |
| Live frame + evaluator | 1,120 | 1,712 | 1,072 | Damage(10) 반환 성공 |

두 번째 예산은 frame 352 + 정상 평가가 보고한 evaluator retained 768이다. 첫 번째 시험은 **단일 할당부터 잔여량보다 크므로**, 요청 합계와 실제 peak를 혼동해서 생긴 finding이 아니다. 메모리 수치는 해당 release 빌드에서 allocator에 전달된 요청 크기이며 RSS가 아니다.

현재 계약은 신규 할당의 보수적 사전 admission이다. 충분하지 않은 크기를 청구하고 성공하므로 F01의 메모리 경계가 닫히지 않았다. 기존 '잔여 1 byte' 시험은 조기 거부만 확인하고, 추정값이 과소인데도 허용되는 경계를 검증하지 못한다.

증거: [probes.rs](probes.rs), [실패 로그](frame_allocation_probe.log), [실행 receipt](frame_allocation_probe.json). 두 테스트 모두 최종 allocation 상한 assertion에서 실패했으며, 그 전에 frame 생성/정상 계산 성공을 확인했다.

### 수정 방향

- 실제 컨테이너 할당 단위를 덮는 보수적 경계 또는 할당 용량을 통제할 수 있는 bounded frame 저장 구조를 사용한다. 첫 노드뿐 아니라 성장/분할, 동적 key payload도 포함한다.
- 관측된 1,072 bytes를 플랫폼 공통 상수로 단순 하드코딩하거나 전역 예산을 올리지 않는다. 구조·지원 플랫폼에 근거한 상한을 제시한다.
- Snapshot의 유사한 `capture_extent` 추정도 확인하되, 이미 소유한 frame을 평가 때 다시 청구하지 않는 원칙은 유지한다. 이번 직접 재현 범위는 Live다.
- 수정 후에는 부족한 예산이면 작업 전에 거부하거나, 성공한 경우 해당 신규 할당이 허용량 안에 들어오는지 확인한다. 첫 삽입·성장 경계·문자열 key·반복 평가 회귀를 포함한다.
- F02/F03/CAP 교정으로 범위를 확대하지 않고 별도 correction target으로 재검토받는다.

## 확인된 개선과 PC 거부 증가의 해석

코드상 evaluator의 local work/retained 검사 후 host admission이 실제 작업 앞에 연결됐고, 기존 adapter의 평가 후 일괄 work 청구는 제거됐다. 취소 poll도 host에 연결됐다. 기존 F01 회귀 3개와 ValueExpr 회귀 36개는 직접 재실행해 모두 통과했다.

구현자 PC 비교의 36조건 중 30완료/6거부 → 25완료/11거부는 보존된 결과를 검토한 것이며, 이번에 36조건을 다시 측정한 것은 아니다. 추가 거부 5개는 128 Read 반복 평가에서 이전에 빠져 있던 누적 메모리 청구가 적용된 결과와 일치한다. 제거된 사후 청구가 그대로 남아 work를 이중 청구한다는 근거는 없다.

다만 이것이 누적 할당 정책의 게임 적합성을 증명하지는 않는다. 반복 중 126회 완료는 합성 fixture의 결과이며, 적 수나 타격 수의 제품 한도가 아니다. 현재 정책은 해제된 임시 메모리도 operation 동안 누적 청구하므로 peak 메모리와 다르다. 정확한 admission 교정과 불필요한 재할당/재생성 개선 후, 목표 전투 규모에서 별도로 적합성을 판단해야 한다. 이번 finding을 '안전 검증이 과도하니 제거해도 된다'는 결론으로 해석하지 않는다.

## 실행 범위와 자료 무결성

- 동결 archive에서 `/tmp/r06-f01-audit.oZdgjJ`에 소스를 복원했다. 322개 원본 파일을 manifest와 대조하고, 테스트 모듈 hook 1개 외에는 일치함을 확인한다.
- hook은 격리 복사본의 기존 runtime tests 안에 reviewer probe 모듈을 연결할 뿐 제품 로직을 변경하지 않는다. 별도 target 디렉터리의 offline release 빌드를 사용했다.
- workspace 해석을 위해 현재 `game_server/Cargo.toml`, `game_server/src/main.rs`를 보충했다. game_server는 컴파일하지 않았다. 두 지원 파일의 해시와 목적은 `verification.json`에 기록한다.
- 직접 실행: 신규 경계 probe 2개 실패(위 finding), 기존 admission 회귀 3/3 통과, ValueExpr 36/36 통과. 명령과 실행 시각은 각 JSON receipt에 있다.
- 구현자의 전체 1,462 tests, all-target check, PC 36조건, work=1 allocation 비교 및 별도 평가 도중 취소 probe는 기존 증거를 검토했으며 전체 재실행하지 않았다. R05 별도 allocation guard도 이번에 재실행하지 않았다.
- 제품 코드·기존 baseline·동결 target·기존 증거는 수정하지 않았다. 새 reviewer 증거만 이 디렉터리에 보존했다. 종료 시 제공된 read-only verifier를 재실행한 결과와 reviewer 자료 해시는 `verification.json`에 기록한다.
- 검토자는 앞선 교정 방향에 참여한 동일 대화의 reviewer이며, 이번 구현자의 자체 검증과 구분되는 코드 검토·추가 probe를 수행했다. blind 독립 검토라고 주장하지 않는다.

종료 상태: F01 추가 교정 필요 / 동결 target 유지 / 후속 구현 및 WP 진행 미승인.
