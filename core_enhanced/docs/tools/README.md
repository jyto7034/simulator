# P2G 문서의 자동 검사

현재 [P2G r2](../policy_to_goal_pipeline_revised_v2.md) §5.4, P2G-40, §8.0/8.3이 요구하는 구조 관계를 검사한다. 이 도구 설명은 정책·단계 규칙이나 Goal Quality Contract를 바꾸지 않는다.

## 자동으로 확인하는 최소 항목

| 범위 | 검사 | 별도 검토가 필요한 것 |
| --- | --- | --- |
| 입력 근거 | exact Ref, 실제 JSON artifact 해석, 중복/모호한 참조·참조 순환, 입력 파일 해시 | 사용자 승인·실제 코드의 currentness·정책 의미 |
| 정책 담당 | Freeze의 실제 Stable Element ID에 대한 selector 해석, primary 담당의 누락/중복 | 작업 경계와 구현 방식의 적절성 |
| 실행 구조 | WP/group partition, dependency endpoint, cyclic SCC·condensation, node의 Phase 배정·순서 | edge의 이유와 실제 동시 전환 필요성 |
| 검증 담당 | Phase별 audit row와 integration row/담당 node·Phase 일치 | 수용 기준을 충분히 검증하는 recipe인지, 중간 상태가 실제 유효한지 |
| 결과 인수 | 동일 입력에서 계산한 Receipt 관계·count·해시와 실제 파일 대조, 불변 target 덮어쓰기 거부 | 구현·게임 테스트·완료 감사 성공 |

`passed`는 위 구조 검사 결과다. source 파일을 읽지 않고 코드 검토 성공을 주장하거나, 검사 통과만으로 정책/구현을 승인하지 않는다. 작성자가 제시한 integration 요구 목록이 의미상 완전한지는 P2G 단계 검토가 소유한다.

## 입력과 호환성

`p2g_inputs.mjs`가 직렬화 해석을 맡고, `validate_p2g40_receipt.mjs`는 한 번 해석한 관계를 생성·검증에 같이 사용한다. `validate_p2g50_goal_set.mjs`도 같은 upstream 해석과 Receipt 검증을 사용한다.

- 새 Freeze: `artifacts: [{ref, payload}]` JSON. 해당 Topic 디렉터리의 JSON artifact bundle에서 참조를 exact-match하고, **Policy Element payload의 Stable Element ID**를 읽는다. revision 이름에서 ID를 추정하지 않는다. 필요한 참조의 누락/중복/손상은 실패하며 하위 디렉터리·저장소 전체를 탐색하지 않는다. unrelated non-artifact JSON은 근거로 포함하지 않는다.
- 새 Plan: 같은 JSON artifact envelope. `payload`는 아래 명시적 relation을 소유한다. 제목·표 순서·ID 접두사는 필요 없다. `validation_receipt_ref`, `validator_method_version`은 envelope의 생성/검증 metadata이며 Plan canonical payload의 역참조가 아니다.
- 기존 Markdown: 기존 template 제목/표와 과거 PER 이름 규칙을 **호환 adapter에서만** 해석한다. 기존 Freeze·Plan·Receipt를 새 형식에 맞추려고 수정하지 않는다. 이 adapter는 모든 임의 Markdown 표현을 지원한다는 뜻이 아니다.
- 새 Receipt: 기존 `p2g40.phase_plan_validation_receipt/v1` 구조와 typed alias 의미를 유지한다. method `1.2.0`은 `structural_input_evidence`에 exact typed 입력 Ref와 실제 해석한 JSON bundle의 경로/해시를 추가한다. 기존 method `1.0.0/1.1.0` Receipt는 기존 입력 경로에서 재검증할 수 있다. JSON 입력에는 `1.2.0`을 사용한다.

Plan의 JSON 직렬화 필드와 기존 canonical member의 대응은 다음과 같다. 새로운 정책 authority 또는 독립 artifact 종류를 만드는 매핑이 아니다.

| JSON payload field | 소유하는 relation |
| --- | --- |
| `Bound Final Policy Freeze Ref`, `Exact Prerequisite Composite Basis` | typed Freeze/Working Revision 입력 기준 |
| `Applicable P2G-40 Obligation Key Set`, `P2G-40 Resolution Record Ref Set`, `Applicable Obligation Supersession Record Ref Set` | 명시적 배열로 나타낸 repair 관계. 해당 없으면 `[]` |
| `work_packages` | `id`, coverage role별 `selectors` 배열, boundary/change/prerequisite/validation/unknown |
| `dependencies` | `from`, `to`, `type`, evidence와 무시 시 깨지는 전제 |
| `atomic_groups`, `standalone` | group `id`/`members`와 standalone WP ID 배열 |
| `cyclic_sccs` | cyclic WP `members`·disposition·근거. cycle 없으면 `[]` |
| `phases` | ordered `id`, `primary` selector, `nodes`, dependency/exit |
| `audits` | owning `phase`와 상태·검사·근거·한계 |
| `integration` | `id`, `source`, `disposition`, `nodes`/`phases`, locator·evidence·prerequisite |
| `Goal Structure Fact`, `Parallelization` | `single_phase`/`multi_phase`, 병렬화 disposition |
| `Shared Contract Handoff`, `Known Unknown/Limitations` | 의미 검토·구현 인계 내용 |

## 실행

```bash
node core_enhanced/docs/tools/validate_p2g40_receipt.mjs --write-receipt \
  --plan <plan.json-or-md> --freeze <freeze.json-or-md> --receipt <receipt.json>
node core_enhanced/docs/tools/validate_p2g40_receipt.mjs \
  --plan <same-plan> --freeze <same-freeze> --receipt <same-receipt>
node --test core_enhanced/docs/tools/p2g_validation.test.mjs
```

새 Receipt는 입력 관계 검사가 통과한 경우에만 쓴다. 같은 결과가 이미 있으면 `unchanged`이며 기존 bytes를 바꾸지 않는다. 다른 immutable target이면 successor Ref/path가 필요하다. 기본 검증은 파일을 수정하지 않는다.

회귀 검사는 OS 임시 디렉터리의 fixture와 저장소의 기존 불변 문서를 사용한다. CLI 검사에는 Node 자식 프로세스 실행이 필요하다. 게임 코드/RON/개인 저장 데이터를 작성하거나 게임을 실행하지 않는다. 과거 Goal Set을 이용한 P2G-50 검사는 도구 호환성 검사이며 새 Topic의 Goal 작성이나 P2G-50 단계 실행이 아니다.
