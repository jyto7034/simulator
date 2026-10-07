# C03 시도 이력

## 시작

C04/C05 검토 PASS와 기존 교정 지시서·두 템플릿을 확인했다. predecessor currentness/격리 복원 통과 후 before342개를 보존했다.
ordered executor의 관측 on은 compiled literal plan, off는 authored literal을 command builder에 전달한다. Damage는 unsigned_abs/u32, Heal은 signed i32 유지다. 기존 시험은 catalog7인 step의 literal만-7로 바꿔 검증했으므로 이번에는 경계값마다 실제 RON/catalog를 만든다.
HP clamp로 command 입력을 구분할 수 없는 경우가 있어 actual process_skill_commands_in_scope의 소비 직전 값을 cfg(test) thread-local observer로 수집한다. 감시는 scope 종료·panic에 해제하고 제품 빌드에는 포함하지 않는다.

## 최초 경계값 준비 실패

c03_boundaries_001: 0 pass/2 fail. 두 시험 모두 첫 i32::MIN의 RON parse에서 ExpectedInteger로 거부되었다. 실행 command에 도달하지 않았다. 실패 당시 source/log/영수증을 보존했다.
추가 코드 대조 결과 Damage 음수는 기존 공개 SkillDatabase 계약이 거부하고 Heal signed i32는 typed set_effect_amount가 허용한다. 따라서 모든 경계를 RON으로 공개할 수 있다는 최초 fixture 가정을 폐기한다. 공개 거부는 유지·확인하고, Heal은 기존 typed authoring setter→정상 builder/catalog로 구성한다. Damage 음수의 runtime 호환 경계는 테스트에서만 실제 catalog compiler로 구성하는 staged fixture로 구분한다.

## typed/staged fixture 재검증

c03_boundaries_002: 2 pass/0 fail. Damage/Heal 각각 MIN,-7,0,7,MAX의10개 입력 × 관측 없음/일반 관측만/수치 관측까지의3설정 =30회 실제 소비를 확인했다. actual command 입력, HP clamp, 완료 fact 및 event/queue/gameplay가 동일하다. 수치 관측10회는 record/공통 evaluator/두 query encoder도 독립 기대값과 일치한다.
GameDataBase의 새 메서드는 cfg(test) 안에서만 음수 Damage catalog를 real compile_skills로 재구성한다. 테스트는 공개 typed setter의 거부와 미변경을 먼저 확인했다. Heal MIN 등 허용값은 기존 setter→정상 builder 경로다. 변경 source는3개와 신규 test1개이며 전체 검증을 시작한다.

## 최종 검증

최종 source에서 check, authoring112, value_expr101, 전체1552(기본 병렬 약125초), Clippy, 변경 파일format이 통과했다. 집중 로그30행도 기대한10개 경계×3설정으로 확인했다. BOUNDARY_MATRIX.json과 FINAL_VALIDATION.json에 원자료를 대조했다.
새 cfg(test) 부분을 제거하면 변경3개가 before와 byte 동일하고 나머지339개도 동일하다. 선행 원자료/도구/환경 대조를 EVIDENCE_REUSE.json에 기록하여 PC36/R05를 재사용한다. 새 측정값은 없다.
기존 freeze를 변경3/추가1, C03/2개 신규시험, test-only 정규화 대조에 맞춰 조정했다. 최종 freeze/verify 결과는 target의 복원 기록 및 루트 영수증이 소유한다. C03-OBS01과 C06–C15는 남겨두고 단계검토로 인계한다.
