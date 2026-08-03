# Goal 완료 검토 가이드

이 문서는 완료된 master/subgoal 항목이 장기 방향에 맞게 실제 구현됐는지 확인하는 재사용 가능한
검토 절차를 정의한다.

Gameplay 정책의 source of truth가 아니다. 완료된 goal 체크박스, post-policy 구현 goal, 대규모
refactor master goal을 감사할 때 사용한다.

## 문서 책임

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | 감사 가이드 |
| 소유 범위 | 완료 검토 입력, 증거 수집 절차, verdict 분류, cross-component 검토, 검증 기준, 검토 출력 형식 |
| 소유하지 않는 범위 | gameplay/runtime/data/wire 정책, goal 구현 계획, 현재 프로젝트 상태, runtime과 live data의 최종 권위 |
| 함께 확인할 문서 | [README.md](README.md), [engineering_workflow.md](engineering_workflow.md), 영향받는 canonical 문서, 검토 대상 goal |

## 목적

완료 검토는 단순히 “체크박스가 선택됐는가?”보다 다음을 확인해야 한다.

- 문서화한 정책이 runtime 동작으로 구현됐는가?
- 구현이 프로젝트의 장기 refactor 방향과 일치하는가?
- legacy fallback, compatibility layer, dual schema, 숨은 source-of-truth 분리가 남아 있지 않은가?
- 테스트가 사용자-visible 동작과 외부 계약을 보호하는가?
- 남은 위험이 후속 작업을 위해 명시적으로 분류됐는가?

## Source Of Truth 적용

프로젝트 전체의 증거 우선순위와 문서 등급은
[README.md](README.md#source-of-truth-순서)가 소유한다. 완료 검토에서는 그 순서를 다음처럼
적용한다.

- 실제 runtime code와 live RON/data에서 시작한다.
- 검토 항목이 Unity 경계를 넘으면 Unity-facing 계약을 확인한다.
- 의도한 동작의 해석에는 최신 canonical 정책 문서를 사용한다.
- goal 문서와 과거 기록은 현재 정책이 아니라 주장과 증거 포인터로 취급한다.

Goal 문서가 완료를 주장해도 runtime code 또는 live data가 다르게 말하면, code/data 동작으로
입증할 때까지 해당 항목을 미완료로 판정한다.

## 검토 입력

완료된 각 master goal 항목에서 다음을 수집한다.

- master goal 항목 원문
- 연결된 subgoal 디렉터리
- 확인된 정책 결정
- 기대하는 사용자-visible 동작
- 영향받는 runtime module
- 영향받는 live RON/data 파일
- 영향받는 Unity/server DTO 또는 transport message
- 동작을 고정했다고 주장하는 테스트와 검증 명령

연결이 하나라도 빠져 있으면 추측하지 않고 누락 자체를 검토 finding으로 기록한다.

## 항목별 검토 절차

완료된 각 항목에 다음 절차를 적용한다.

1. 의도한 정책 또는 refactor 결과를 한 문단으로 다시 쓴다.
2. 구현 후 기대하는 source of truth를 식별한다.
3. 해당 source를 생성, 변경, 검증, 직렬화, 소비하는 runtime code path를 읽는다.
4. 관련 live RON/data와 schema/loader validation을 읽는다.
5. 외부 계약에 닿는 항목이면 Unity-facing DTO, command payload, WebSocket message, snapshot,
   timeline/log output을 확인한다.
6. 구 이름, 구 enum variant, 제거된 field, fallback path, ignored test, debug-only bypass,
   compatibility shim을 검색한다.
7. 테스트가 private 구현 모양보다 동작을 고정하는지 판단한다.
8. 항목에 맞는 focused validation을 실행한다.
9. 증거와 남은 위험을 포함한 verdict를 기록한다.

## 장기 방향 검토

일반 refactor, legacy 제거와 추상화 판단 기준은
[engineering_workflow.md](engineering_workflow.md)가 소유한다. 이 절은 그 기준을 감사 질문으로
적용하며 별도의 engineering 정책을 정의하지 않는다.

각 완료 항목이 국소 패치가 아니라 실제 장기 설계인지도 확인한다.

- source-of-truth 중복을 줄였는가?
- 동기화해야 하는 중복 상태를 새로 만들지 않았는가?
- obsolete code/data/test를 compatibility layer 뒤에 보존하지 않고 제거했는가?
- 알려진 테스트 하나만 만족하는 special-case patch를 피했는가?
- 이름, 타입, module 위치가 domain 책임과 일치하는가?
- parallel feature를 추가하는 대신 기존 함수를 조합할 수 있었는가?
- 더 단순한 구조가 가능해졌는데도 남겨두지 않았는가?
- runtime, data loading, validation, Unity DTO, test 사이의 책임이 더 명확해졌는가?
- 미래 content/policy 변경을 또 다른 migration 없이 표현할 수 있는가?
- 테스트가 현재 helper 배치보다 durable contract를 보호하는가?

다음을 기록한다.

- Long-term fit: high / medium / low
- Improvement class: none / follow-up refactor / immediate correction recommended
- Reason: 구체적인 code/data/test 증거

## Verdict 분류

항목마다 다음 primary verdict 중 정확히 하나를 사용한다.

- `Complete`: code/data/contract에 구현됐고 의미 있는 검증으로 보호되며 material legacy path가 없다.
- `Partially complete`: main path는 동작하지만 edge path, data loader, DTO, validation 또는 test gap이 남아 있다.
- `Document-only complete`: goal 문서는 완료로 표시하지만 runtime/data 동작이 입증하지 못한다.
- `Policy decision needed`: 현재 정책으로 결정되지 않은 gameplay/UX/schema/balance 판단이 필요하다.
- `Not applicable`: 이후 확정된 정책이 필요성을 제거해 해당 항목이 obsolete가 됐다.

Secondary long-term fit rating도 추가한다.

- `High`: ownership을 단순화하고 예상되는 미래 모델과 일치한다.
- `Medium`: 현재는 수용 가능하지만 후속 단순화나 통합 가능성이 높다.
- `Low`: 구 모델을 우회하는 패치거나 혼란스러운 ownership 분리를 남긴다.

## 증거 템플릿

검토 항목마다 다음 형식을 사용한다.

```md
## <Item Name>

Master item:
Subgoal:
Policy source:

Expected behavior:

Runtime evidence:
- `<file>`: ...

Data evidence:
- `<file>`: ...

External contract evidence:
- `<file or DTO>`: ...

Test evidence:
- `<test name or command>`: ...

Legacy/fallback audit:
- ...

Long-term direction review:
- Fit: high / medium / low
- Improvement class: none / follow-up refactor / immediate correction recommended
- Reason: ...

Verdict:
- Complete / Partially complete / Document-only complete / Policy decision needed / Not applicable

Remaining risk:
- ...
```

## Cross-Component 검토

모든 항목을 개별 검토한 뒤 component 사이를 다시 확인한다.

- 완료된 두 항목이 서로 경쟁하는 source of truth를 정의하는가?
- 한 goal의 DTO 또는 schema 전제가 다른 goal과 충돌하는가?
- component를 함께 실행하지 않아 테스트가 통과하는 것은 아닌가?
- live RON/data가 여전히 과거 정책을 따르는가?
- Unity-facing 문서/계약이 여전히 과거 shape를 설명하는가?
- debug export/log가 의도대로 debug-only인지, 실수로 runtime source가 됐는가?

이 단계는 짧은 cross-component finding 목록을 만들어야 한다. Finding을 개별 항목의 note 안에
숨기지 않는다.

## 검증 기준

Focused validation을 먼저 실행하고 broad validation을 마지막에 실행한다.

Focused validation은 다음을 다뤄야 한다.

- 항목이 변경한 runtime 동작
- data shape가 바뀌었다면 live RON loading과 validation
- 외부 계약이 바뀌었다면 Unity/server DTO shape
- timing/order가 중요하다면 대표 gameplay flow

Broad validation은 영향 범위를 실행하는 가장 작은 명령 집합을 포함한다. 명령, 실패, 수정과 최종
결과를 기록한다.

## 검토 출력

검토 goal은 다음을 만들어야 한다.

- 항목별 audit table
- 불완전하거나 위험한 항목의 상세 evidence section
- cross-component finding section
- immediate correction 후보 목록
- follow-up refactor 후보 목록
- 검증 명령과 결과
- 명시적인 정책 질문(있는 경우)

검토 goal이 구현까지 명시적으로 포함하지 않았다면 검토 중 광범위한 문제를 조용히 수정하지 않는다.
전용 goal로 전환할 수 있는 정확한 correction plan을 우선 만든다.
