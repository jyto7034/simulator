# WP02 D04 유닛 관계 구간 실패 이력

실패 결과도 성공 영수증과 같은 `evidence/`에 원 로그와 source digest를 보존한다.

## R05 도구 연결

1. `r05_guard_final`은 최종 검증 도구의 `BattleUnitDraft` literal에 새 `equipment_layout` 필드가 없어 컴파일에 실패했다. 실제 소비자인 `runtime_architecture_final_validation/src/final_performance.rs`에 `None`을 명시했다.
2. 별도 보존 baseline 생성기를 시험하기 위해 `runtime_architecture_preservation`을 임시 수정하고 오래된 lock을 offline으로 갱신했다. `preservation_tool_check_final`의 locked 실패와 `preservation_tool_lock_refresh`, `preservation_tool_check_final_002` 성공을 보존한다.
3. `r05_guard_final_002`가 baseline receipt에 고정된 보존 도구 SHA와 임시 수정본 SHA가 다르다고 거부했다. 이 도구는 현재 제품용 실행기가 아니라 승인된 R05 워크로드의 byte identity이므로 `src/main.rs`와 `Cargo.lock`을 보존한 원래 SHA `64debfd6546afb617636caf520fcb9e8ee49acc1ee2377306cf8a19afbd8c631` / `2ffda73f435af0d3fea27376a204e9044c4d6867e66721f34e66a29ac3d7b03c`로 복원했다. baseline receipt나 기대 SHA는 수정하지 않았다.
4. `r05_guard_final_003`과 최종 `r05_guard_final_004`는 도구·입력 검증과 모든 workload를 완료했다. 두 실행은 concurrent allocation peak guard 하나에서만 종료 코드 101을 냈다. 최종 raw 최대값은 9,049,432 bytes로 정책 cap보다 19,244 bytes 높다. 다른 결과·결정성·시간·event allocation guard는 통과했다.

## 형식과 증거 도구

- `changed_region_format_final`은 world 실제 projection 시험과 core memory 계량의 새 줄에서 rustfmt overlap을 발견했다. world 줄을 고친 뒤 `changed_region_format_final_002`가 core의 두 표현을 추가로 발견했다. 두 곳을 고친 최종 `changed_region_format_final_003`은 통과했다.
- `pc_compare_final_002`는 비교 도구에 상대 경로를 넘겼을 때 `relative_to`가 거부했다. 입력 경로를 workspace 절대 경로로 정규화한 `pc_compare_final_003`은 통과했다. PC raw 결과는 바뀌지 않았다.

최종 선택 영수증의 공통 337-file source digest는 `eb9bd8fb3927f8a9f3c5b35908012300e4fad9c4e8d7a03a9e612e44b0dc804a`다.
