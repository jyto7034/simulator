# 추가 통합 실패17개 교정 완료

2026-10-14 사용자 후속 교정 요청을 수행했다. **원17개의 검증 목적을 유지한 현재16개 테스트가 모두 통과했다.** 기본 등록·장착 smoke1개는 고유 assertion을 공유 준비와 성공 이전 테스트에 옮긴 뒤 제거했다. 변경한 통합2target 전체는 **40통과·0실패·0ignore·0filter**다.

## 변경 결과

| 대상 | 교정 | 보존한 검증 |
|---|---|---|
| U08 공통 시작 | `Standard`→`Expedition`, 직원 선택에 필수 `loan_copy_ids: []` 제공 | 공개 수치6·이벤트3·판매3·이전/생명주기3의 본래 oracle |
| 외부 등록·장착 smoke | 파일 존재/pack 수를 `content()`로, 최초 단일 사본·owner·slot0을 성공 이전 테스트로 통합 | 기존 slot2→다른 직원 slot1 이전 및 Normal/Corrupt 기여·역조회 해제 유지 |
| 포기 후 새 런 | 활성 런 거절과 포기 후 성공 시작 모두 유효한 `replace_active_run: false` 사용, 실제 profile 설치 후 Continue | 유물·슬롯·촉매4·분진23 초기화, 오래된 copy 거절, 새 런 복원 |
| 미완료 이벤트 Continue | 현재 원정 FacilityLayout으로 실제 진입 가능한 이벤트 예약, 새 앱 두 곳에 해당 profile 설치 | 진입전 checkpoint 바이트, 임시 타락/이력 되감기, 사전 유료 타락·촉매2·분진11 보존, 재진입·완료·복원 |
| 시작·영입 장비 명세 | 시작 Tank `containment_trainee`를 shield/last_line으로 정합화 | 12명 정확한 세트·baseline 없음·일반 보상풀 분리, 영입 Dealer greatsword/breakthrough 유지 |
| 규칙표 | 시작 탱커2의 현행 방패 세트를 반영하고 영입 후보와 일반 이전 규칙을 구별 | 제품 데이터나 장비 이전 계약 변경 없음 |

실제 변경은 테스트 소스4파일·폐기된 시설 fixture2파일 제거·규칙 문서1파일이다. 제품 실행 로직과 공식 RON 데이터는 변경하지 않았다. 최초 preimages와 [최종 delta](correction002.diff)를 보존했다. [원17→현재16 개별 대응표](CORRECTION_MATRIX.json)에 목적·보존 assertion·실제 통과 근거가 있다.

구현 위치: [공통 준비](../../../tests/u08_relic_public/support.rs), [이전·재시작·Continue](../../../tests/u08_relic_public/lifecycle.rs), [수치 plan 테스트](../../../tests/u08_relic_public/prospective.rs), [시작 장비 명세](../../../tests/ron_loading.rs), [규칙표](../../game_rules/equipment-and-fragments.md).

## 생명주기 경로와 중복 통합 검토

미완료 이벤트는 현재 공식 `facility_6A` 템플릿을 사용하는 깊이1 방과 ENTRY 연결을 작성한다. 시작점 S에 인접한1번 슬롯에 지원되는 `u08_public_double_corruption` Event를 예약하고 실제 `select_map_node`→`confirm_enter_node`로 들어간다. 작성 입력을 RON으로 직렬화한 뒤 정상 content loader의 검증·generation 발급을 거친다. 나머지 콘텐츠는 바이트 동일성을 확인한다. 이 시험의 entry cost는 작성 가능한0으로 명시해 유물 비용 복원에 집중하며 현재 시설 난이도와 일반 슬롯 배치는 유지한다.

구형 Standard 템플릿/프로필을 통째로 덮어쓰던 fixture 두 파일은 사용처와 함께 제거했다. Tutorial 전환이나 admin Event 진입으로 durable 저장 경로를 대체하지 않았다. 실제 소유 앱의 PlayerProfile snapshot을 새 앱의 bootstrap port에 설치하고 checkpoint를 복원한다. 미완료 이벤트와 완료 이벤트의 Continue를 모두 통과했다.

포기 전 시작 거절은 이제 금지된 true 플래그 때문에 조기에 실패할 수 없다. 같은 false 요청이 활성 런에서는 거절되고 Abandon 후에는 성공하므로 의도한 활성 런 경계를 검사한다. 실패 전후 public snapshots와 checkpoint의 불변 검증은 유지했다.

통합한 smoke의 파일 존재·pack2·초기 copy1/owner/slot0 검증은 남아 있다. 성공 이전 검사는 여전히 Normal/Corrupt 각각의 정확한 사본·자원 보존, 기존 직원100/새 직원180 또는210, penalty 귀속과 copy만 지정한 unequip를 확인한다. 빈 목록 JSON 키 생략, 잘못된 공개 명령, 기타 수치 plan assertion은 제거하지 않았다.

## 실행 및 재사용 근거

| 근거 | 결과 | 해석 |
|---|---|---|
| [correction001](correction001.result.json) | 빌드 실패, 테스트 미실행 | 새 fixture의 typed ID에 `From<&str>`를 가정한 작성 오류. 공개 `::new` 생성자로 수정; 원문 보존 |
| [correction002](correction002.result.json) | exit0, ron_loading25/25·u08_relic_public15/15 | 최종 입력 전체40통과, 생명주기2의 후반 assertion까지 실행 |
| 직전 integration_021/023의 비영향6target | 32통과 재사용 | 해당 테스트·공유 common·제품·콘텐츠 변경 없음 |
| 직전 lib2097·계측23·외부CLI12 | 비영향 근거 재사용 | 이번에 재실행한 것으로 세지 않음 |

직접 실행한40과 비영향32의 통합 커버리지 합집합은72개다. 이전 통합73개에서 smoke1개를 통합한 결과이며, 전체8target을 한 명령으로 다시 실행했다는 뜻은 아니다. 기존 lib의 G11은 직전 보고서에 명시된 현재 manifest 환경 조건을 따르고 기존 ignore6도 그대로다. 이번 결과를 조건 없는 전체 저장소 green이나 기존 범위 한계의 해소로 확대하지 않는다.

[실제 stdout](correction002.stdout.log), [실행 색인](EXECUTION_INDEX.json), [요약](SUMMARY.json), [시험 입력](correction002.inputs.json), [직전 교정35파일 불변 확인](REUSE_INPUT_VERIFICATION.json)을 남겼다. 직전 통합 원문은 [021](../core_baseline_failure_correction_20261014_001/integration_021.stdout.log)·[023](../core_baseline_failure_correction_20261014_001/integration_remaining_023.stdout.log), 이전 전체 검증 조건은 [기존138 교정 보고서](../core_baseline_failure_correction_20261014_001/REPORT.ko.md)를 따른다. 현재 세 테스트 모듈의 rustfmt check와 추가 행 공백 검사도 통과했다. 테스트 실행 뒤 소스 해시가 최종 입력과 같음을 확인했다.

재현 명령은 저장소 루트 기준이다.

```bash
PATH=/home/blast/.cargo/bin:$PATH CARGO_HOME=/tmp/wp02-cargo-home \
  cargo test -p game_core --test ron_loading --test u08_relic_public \
  --offline --locked --target-dir /mnt/f/work/simulator/.cache/u12-vault-cost-target \
  --no-fail-fast
```

이번 요청 범위의 미해결 실패는 없다. 기존138 교정과 과거 U14 동결·감사 기록은 보존했으며 U15는 착수하지 않았다.
