# F-U164-IA001 교정 결과

2026-10-15 사용자 교정 요청에 따른 GEX-10 기술교정이다. **누적 수량 결함의 교정과 영향 검증을 완료했다. U16-4 전체는 Not Complete다.** 기존 독립감사·실패원문은 보존하며 이 기록을 새 독립 완료감사로 부르지 않는다. 감사에 참여했던 실행자가 이번 교정을 수행했으므로 이후 비참여 감사는 별도 실행자가 맡아야 한다.

## 변경과 근거

`StressCombatPolicy::validate`가 엔케팔린·장비 가루·재료ID별로 모든 라운드와 반복 효과를 누적해 `checked_add`와 고정2배의 표현 가능성을 검사한다. `GrantEquipmentMaterial("equipment_dust")`는 `GrantEquipmentDust`와 같은 잔고에 더한다. 서로 다른 재료·지갑·유물 사본은 합치지 않는다.

금액은 음수가 없으므로 모든 prefix의 누적량이 최대2,147,483,647이면 정상 및2배 지급량이 u32에 들어간다. 동일효과1개의배율 검사로 대체하지 않았다. XP는 `Employee::add_experience_with_policy`에서 각 지급 후 레벨업 비용을 소비하므로 잔고 합산과 구분했다. 기존 개별XP checked_mul(2)와 실제25/50XP 지급 대조를 유지했다. 기존 보유량과의 runtime 합산 한도·실패 원자성 및 공통 정산 코드는 변경하지 않았다.

실제 canonical 변경은 제품 `src/game/stress_combat.rs`, 영구 회귀 `src/game/world/tests/u164_stress.rs`, 계약 설명 `docs/data_loading_contract.md`의3파일이다. `before/`, `CHANGES.patch`, `AFTER.tar.gz`, `INTAKE.json`, `GREEN_INPUT.json`, `RESULT.json`이 전후 입력과 결과를 연결한다. 최종3파일은 시험 입력과 같고 보관 archive도 digest를 검증했다. 이는 교정 인계용 scoped 보관이며 미완료 Goal의 최종 target/정역복원을 대신하지 않는다. 동결 Goal·수용 조건은 바꾸지 않았다.

## 검증

| 실행 | 결과 | 의미 |
| --- | --- | --- |
| red001 | 1pass/2fail, exit101 | 교정 전 새검사가 초과통화와 가루alias 혼합의 잘못된 수용을 실제 검출. 서로다른owner/XP분리 양성은통과 |
| green001 | 16pass/0fail/0ignore, exit0 | 기존9+영구회귀3+감사양성replay/추가경계4 |
| loading001 | 25pass/0fail/0ignore, exit0 | 공식 RON 로딩·참조·시나리오·보상owner 회귀 |
| package-reject001 | exit1, FinalValidation/cumulative enkephalin | 감사 때 승인됐던 동일10억×5 입력을 공개 패키징 단계에서 거절 |
| package-boundary001 | exit0 | 누적2,147,483,647 경계의 정상 작성 pack 승인 |

새 영구 회귀는 누적경계/+1, 무억제에서만 초과하는5억×5, 감사원형10억×5, 같은라운드 반복효과, 가루alias·같은재료 합산/서로다른재료 분리를 확인한다. 단순 오류유무뿐 아니라 누적검사 사유를 확인한다. 기존 개별 후반 보상 overflow 검사도 유지했다.

별도 사본의 `POSITIVE_REPLAY.rs`는 과거 감사의 다중wave·공개소진/Continue·pending뒤후퇴 양성3개를 같은 기대값으로 재실행한다. 추가경계1개는 정상 작성 초기잔고0에서1–4라운드 각429,496,729/5라운드429,496,731을 실제 전투로 완료한다. 중간 무지급, 정상2,147,483,647 및 무억제4,294,967,294 지급, 노드소비·Core checkpoint복원을 확인했다. 허용경계를 불필요하게 낮추거나 보상을 포화/잘라내기로 지급하지 않는다. 사본은 `/tmp/u164-correction001`이며 기존 감사사본을 수정하지 않았다.

이16+25 결과는 기존2222 전체 재실행·서버 전체PASS·제품 비용측정이 아니다. 실제host/freshprocess/SavePoint/pairedIO와 전체회수 증거는 아직 없다. 이전 감사의 오류재현 test/진단은 그대로 보존했고, 새 테스트가 같은 잘못된 작성입력을 시작 전에 거절한다. 오류재현진단을 정상기능PASS로 재해석하지 않는다.

## 재현과 남은 경계

실행 workspace `/tmp/u164-correction001`, `CARGO_HOME=/tmp/wp02-cargo-home`, `CARGO_TARGET_DIR=/mnt/f/work/simulator/.cache/u164-review-dev-target`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, Rust1.98.1이다.

```bash
cargo test -p game_core --lib u164_ --offline --locked
cargo test -p game_core --test ron_loading --offline --locked
```

사본 재구성은 기존 감사의 AUDIT_INPUT/REPLAY_SUPPORT를 새 빈 디렉터리에 복원하고 이번 AFTER.tar.gz를 덮어쓴 뒤, 사본 u164_stress.rs 끝에 POSITIVE_REPLAY.rs를 붙이면 된다. 원본 저장소에 기록용 patch를 재적용하지 않는다. Cargo 환경과 의존성 cache는 위 값을 따른다.

공개 패키징은 현재 결과에 기록한 바이너리로 `independent_audit001/cumulative_reward_source`와 이번 `boundary_source`를 각각 새 출력 디렉터리에 전달한다. 기존 `rejected_pack`/`boundary_pack` 및 로그를 덮어쓰지 않는다. 원시 명령 형태는 `package_official_content SOURCE_ROOT NEW_PACK_DIRECTORY`다.

F-U164-IA001은 구현자 기준 교정·영향검증 완료이며 별도 currentness/재감사 판정을 아직 인수하지 않았다. 정상host/저장/파츠·유물·소비품/직원손실/동시각패배/비용·최종동결과 복원 등 기존 Goal 잔여는 PLAN대로 유지한다. 이 교정에서 후속 U17/U21/Unity 작업을 시작하지 않았다. 모든 실행 세션은 종료·회수했다.
