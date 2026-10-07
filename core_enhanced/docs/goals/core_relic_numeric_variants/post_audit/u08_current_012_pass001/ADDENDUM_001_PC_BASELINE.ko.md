# 정정001 — PC 비교 기준 식별

2026-10-07T05:28:56Z. 원본 감사 `COMPLETION_AUDIT.md`와 `AUDIT_CLOSURE.json`, 원 검사 코드·결과를 수정하거나 덮어쓰지 않는다. **원본 PC 문단의 “immediate U07M 대비200 bytes”는 감사자의 비교 기준 표기 오류다.** 실제 비교 수치가 적용되는 두 경계를 아래와 같이 정정한다. F01/F02와 Not Complete / Confidence High / open2 판정은 그대로다.

원 검사 `verify_additional.py`는 `PC_IMMEDIATE_ACCOUNTED_U07M_007.json`의 **`before.path`와 `after.path`**를 읽었다. `immediate_u07m.path`를 읽은 검사가 아니었다. 코드의 주석도 잘못된 기준명을 사용했다.

| 비교 | 실제 결과 |
| --- | --- |
| 이전 `core_battle_starter_tactics/.../pc_raw_current_001.json` → U08 `pc_raw_current_007.json` |576행. 비시간 차이는 native retained/owner extent/refusal requested의 정확히200 bytes이며 다른 비시간 필드는 동일. 이 경계가 원 독립 비교의 대상이다. |
| 직전 U07M `core_runtime_measurement_profiles/.../pc_raw_current_002.json` → U08 `pc_raw_current_007.json` |별도 정정 검사로576행을 직접 재귀 비교했다. `measurement.elapsed_ns`576개와 `setup.elapsed_ns`192개만 다르고 **모든 비시간 필드는 정확히 동일, allowed delta0**이다. native retained/owner extent에도 추가200 bytes가 없다. |

따라서 원본 PC 문단은 “더 이전 starter-tactics 기준에서는 기존 계량 차이200 bytes를 설명하며, **직전 U07M 대비 현재 U08의 비시간 비용과 결과는 정확히 동일하다**”로 읽어야 한다. R05의 별도 allocator peak 비교·경고는 이 정정과 다른 측정이며 변경하지 않는다.

정정 검사는 저장된 raw JSON만 읽었다. 제품·시험·성능 실행0, 기존 증거 변경0이다. 새 검사 코드 `verify_pc_baseline_addendum001.py`와 `PC_BASELINE_CORRECTION_001.json`에 두 비교의 전체 경로·bytes·SHA-256·변경 필드 수를 보존했다.

- 이전 starter-tactics raw: SHA-256 `16f3422c09e16f2ebccf6bfdbedc09b58deff3e6f4b4ca03ecfa8d71a31dc9a3`,1,417,024 bytes.
- 직전 U07M raw: SHA-256 `095560d605ad4ccc03fe1fa38ae4c692569e5ad9336d02d5b749facbe59c74b8`,1,417,011 bytes.
- 현재 U08 raw: SHA-256 `852815534cd817ff7d1f7e5d50974a65257d408083ed6705a9f3326c690c7f46`,1,417,011 bytes.
- 원 cost artifact: SHA-256 `71ce0622df9e62432a73960b955835cbcf2d4c75444c57a31ef1d67f09db794a`,384,802 bytes.
- 보존된 원 감사: SHA-256 `e726f3eacd0b79632281800569bc6fae01977e047eff91cbae5f3213fe9c48b7`,15,798 bytes.
