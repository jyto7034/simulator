# D04-OWNER-F01 실패·재시도 이력

최종 제품 source는 확장 337개 digest `8aa03d6bd04f8d524fa61b465a9bd7a4171062a1190f615b794e20f915190f04`다. 아래 실패와 선택 오류를 삭제하거나 성공으로 바꾸지 않았다.

| 시점 | 관측 | 처리와 최종 상태 |
| --- | --- | --- |
| 개발 중 첫 `cargo test -p game_core --lib equipment_` | 42 passed / 1 failed. 신규 `equipment_projection_keeps_source_order_item_ids_for_by_type_and_any3`의 Armor-only fixture가 실제 Employee source를 사용해 기존 `WeaponRequired` validation에 걸림. 이 초기 직접 실행은 전용 receipt 도입 전이어서 stdout 원파일이 없다. | ID 순번 계약만 보는 case를 기존 `TestFixture` source로 고쳤다. 실제 Employee inventory 경로는 C06 시험에서 별도로 검증했다. 최종 같은 selector는 `correction_equipment_focused_final`, 43/0. |
| `changed_region_format_final` | exit 2. `run.py` 기본 cwd가 workspace root인데 `check_format.py`를 상대 경로로 호출해 파일을 찾지 못함. | receipt와 log 보존. 올바른 correction cwd의 `changed_region_format_final_002`는 exit 0. |
| `id_bytes_c09_final` | exit 0이지만 `--exact`에 모듈 전체 경로가 아닌 짧은 이름을 주어 0 tests selected. 검증으로 채택하지 않음. | receipt와 log 보존. selector를 고친 `id_bytes_c09_final_002`에서 1/0. |
| `r05_guard_final` | exact workload 완료 후 exit 101. `concurrent_sessions.allocation_retained_peak_guard_pass` assertion만 false. | 정책상 concurrent peak 단일 경고. 최대 9,041,800 bytes로 원 cap 9,030,188보다 11,612 bytes 초과. 다른 hard guard failure는 0. 원자료와 panic log 보존. |

최종 check, focused tests, 전체 1,526개, clippy, PC36은 exit 0이다. R05 이외의 미해결 검증 실패는 없다.
