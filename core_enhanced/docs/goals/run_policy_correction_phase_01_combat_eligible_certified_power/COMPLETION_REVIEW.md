# P01 Combat-Eligible Certified Power Completion Review

Review date: 2026-07-28

Verdict: **Complete / High**

Implementation complete: true

Review basis: Child의 완료 주장과 별개로 current formula, shared eligibility authority, 세 소비자,
model/save rejection, live RON, qualitative projection, canonical과 broad runner를 다시 대조했다.

## Audit Table

| Item | Source of truth | Verdict | Fit |
| --- | --- | --- | --- |
| Certified roster membership | `combat_power.rs`, `Employee::is_combat_eligible()` | Complete | High |
| Current threat / Floor initialization / run peak | map projection와 world run-power call sites | Complete | High |
| Model version / old-save rejection | live RunPolicy, active-run continue, difficulty resolver | Complete | High |
| Readiness와 hidden numeric boundary | `MapViewDto` regression, canonical | Complete | High |
| Cross-workspace regression | workspace all-targets | Complete | High |

## 1. Formula Membership And Condition Meaning

Expected behavior:

- `alive && physical_damage <= 4 && trauma < 100`인 전체 roster만 current certified power에 들어간다.
- Eligible 경계 안의 current HP, physical damage와 trauma magnitude는 coefficient가 아니다.

Direct evidence:

- Formula는 배치 가능성의 기존 단일 authority인 `Employee::is_combat_eligible()`를 호출한다.
- Boundary matrix가 physical damage 0/4/5/6과 trauma 0/99/100을 직접 고정한다.
- Eligible condition/HP 불민감 test는 physical 4, trauma 99에서도 persistent loadout power가 같음을
  검증한다.
- Critical, Breakdown, Dead는 matrix의 member set에서 제외된다.

Verdict: Complete / High.

## 2. Consumer Convergence And Historical Peak

Expected behavior:

- Current threat, first Floor initialization과 actual combat-entry peak가 같은 formula/version을 사용한다.
- Current roster가 약해져도 historical run peak는 감소하지 않는다.

Direct evidence:

- 세 call site가 모두 `certified_combat_power_for_roster()`를 사용하고 별도 potential/deployed-subset
  authority가 없다.
- Existing run-power regression이 starter manifestation과 실제 combat entry에서만 peak를 올리는 경계를
  검증하고 통과한다.
- Active-run round-trip과 server backup high-water tests가 peak의 비되감기 의미를 계속 검증한다.
- 전원 Breakdown map regression은 current denominator 0을 `VeryHigh`로 표시하면서 manifested fixed
  difficulty를 재작성하지 않음을 증명한다.

Verdict: Complete / High.

## 3. Versioning And Compatibility

Expected behavior:

- Formula 의미 변경은 새 model version을 사용하고 구 authority/save를 fail closed한다.

Direct evidence:

- Live `combat_difficulty.power_model_version`은 2다.
- Active-run regression이 stale version continue를 `InvalidAction`으로 거부한 뒤 current version import가
  성공함을 같은 fixture에서 검증한다.
- Manifested difficulty resolver는 live version/profile mismatch를 계속 거부하며 live 의미의 fixture만 2로
  갱신됐다.
- Migration, alias, dual reader, fallback과 coefficient 변경은 없다.

Verdict: Complete / High.

## 4. Projection And Canonical

- `MapViewDto`는 qualitative band/threat/readiness만 내보내며 exact certified total, target, model version은
  여전히 wire에 포함하지 않는다.
- `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md`가 exact membership, magnitude 의미,
  zero-power threat와 version 2에 수렴한다.
- DTO shape/Unity consumer가 바뀌지 않아 Unity 구현 변경과 Unity MCP validation은 적용 대상이 아니다.

Verdict: Complete / High.

## Validation

- Pre-fix regression: eligibility matrix 0 passed, 1 failed; Critical 포함 결함 재현.
- Post-fix focused: matrix 1/1, combat power 4/4, map threat 1/1, active-run version 1/1, run peak 1/1,
  live RON 23/23.
- Core broad: 789 passed, 0 failed.
- Server broad: 53 passed, 0 failed.
- Workspace all targets: Core 789, integration groups 3/4/23/13/4, server 53; failures 0.
- `cargo fmt --all -- --check`와 scoped `git diff --check`: pass.

## Final Verdict

- Primary verdict: **Complete**
- Long-term fit: **High**
- Improvement class: **none**
- Remaining risk: none material to P01. P02 must preserve version-2 certified input while moving Floor anchor
  ownership into checkpoint payload and replacing the active-run save schema.
