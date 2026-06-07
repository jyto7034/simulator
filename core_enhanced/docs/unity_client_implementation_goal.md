# Unity Client Implementation Goal

이 문서는 Unity 클라이언트 구현 AI에게 넘길 작업 지침서다. 목적은 완성 UI가 아니라, 현재 `game_server`/`game_core` 계약을 실제로 조작할 수 있는 Unity 클라이언트 골격을 만드는 것이다.

완료된 뒤 장기 보관할 정책 문서는 아니다. 구현이 끝나면 필요한 내용만 `unity_core_contract.md` 또는 Unity 프로젝트 문서로 옮기고 이 goal 문서는 정리한다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

적용할 핵심 원칙:

- 목표는 측정 가능한 완료 조건과 검증 방식으로 정의한다.
- 큰 작업은 빠른 피드백 루프를 만들 수 있는 작은 단계로 나눈다.
- 오래 걸리는 작업에서는 계획, 실험, 생각을 Markdown 파일에 남겨 컨텍스트 압축이나 세션 교체 이후에도 이어받을 수 있게 한다.

Unity 구현 AI는 작업 시작 시 다음 파일을 만들고 계속 갱신한다.

```text
F:\unity projects\ark\docs\goals\unity_client_implementation\PLAN.md
F:\unity projects\ark\docs\goals\unity_client_implementation\EXPERIMENTS.md
F:\unity projects\ark\docs\goals\unity_client_implementation\EXPERIMENT_NOTES.md
```

각 파일의 역할:

- `PLAN.md`: 현재 목표, 구현 순서, 남은 체크리스트, 완료 조건을 기록한다. 계획이 바뀌면 왜 바뀌었는지 함께 적는다.
- `EXPERIMENTS.md`: 시도한 구현 접근, 결과, 실패 원인, 다음 결정을 기록한다. 특히 UI 구조, WebSocket 연결, snapshot 파싱, battle delta 적용 방식의 시행착오를 남긴다.
- `EXPERIMENT_NOTES.md`: 작업 중 떠오른 판단, 의심, 질문, 부채 후보를 시간순으로 남긴다. 사용자와 의논해야 할 내용은 이 파일에도 모은다.

이 세 파일은 최종 제품 문서가 아니라 goal 실행 중의 작업 기억장치다. goal 완료 후에는 유지해야 할 정책만 공식 문서로 옮기고, 임시 기록은 정리할 수 있다.

## Primary References

다음 4개를 기본 레퍼런스로 사용한다.

1. `F:\unity projects\Game\docs\ui_scene_function_notes.md`
2. `F:\unity projects\Game\docs\ui_fixed_layout_contract.md`
3. `F:\unity projects\Game\tools\ima2-mcp\generated\ui-prototype\final\wireframes_ima2_fixed_contract_topbar_v2`
4. `F:\work\simulator\core_enhanced\docs\unity_core_contract.md`

주의: 실제 구현 대상 Unity 프로젝트는 `F:\unity projects\ark\`다. 위 `F:\unity projects\Game\...` 경로는 폐기된 Unity 프로젝트 안에 남아 있는 공식 UI 레퍼런스 위치일 뿐이며, `Game` 프로젝트의 scene/prefab/script를 수정하거나 그대로 복사하지 않는다.

최신 core snapshot 계약 기준:

- 전투 필드 조립은 `combat_preview.tiles[*].position`과 `combat_preview.tiles[*].kind`를 기준으로 한다.
- `tiles.kind`는 `Ground`, `Platform`, `Obstacle` 중 하나다.
- `Void`는 별도 tile kind가 아니라 `tiles`에 없는 좌표다.
- 기존 `valid_tiles`, `deployment_zones`, `obstacles`는 보조/검증 계약으로 유지되지만, Unity 렌더링의 source of truth는 `tiles`다.
- 직원 배치 허용 타입은 `roster.employees[*].combat_profile.effective_deployment_affinity`를 우선 사용하고, 없을 때만 `deployment_affinity`를 표시용 fallback으로 본다.
- `deployment_affinity` 값은 `ground_only`, `platform_only`, `any` 같은 snake_case다.

필요하면 다음 core 문서를 보조로 읽는다.

- `F:\work\simulator\core_enhanced\docs\game_rulebook.md`: 게임 흐름, 전투 모드, 노드 정책.
- `F:\work\simulator\core_enhanced\docs\gameplay_flow_example.md`: 실제 플레이 흐름 예시.
- `F:\work\simulator\core_enhanced\docs\skill_target_contract.md`: 스킬 타겟/범위 계약. 전투 HUD나 스킬 범위 표시가 필요할 때만 읽는다.
- 폐기된 `F:\unity projects\Game` 프로젝트의 Builder 관련 과거 기록은 필요할 때만 참고한다.
  - `F:\unity projects\Game\docs\goal_work\03_battlefield_tile_assembler\PLAN.md`
  - `F:\unity projects\Game\docs\goal_work\03_battlefield_tile_assembler\RESULT.md`
  - `F:\unity projects\Game\docs\goal_work\04_combat_deployment_tile_field_assembler\RESULT.md`
  - `F:\unity projects\Game\docs\tile_art_asset_handoff.md`
  - 이 문서들은 방향 참고용이다. `ark` 프로젝트에 과거 코드를 그대로 복사하지 않는다.

## Existing Unity Project Reference Policy

이 goal의 실제 구현 대상은 `F:\unity projects\ark` 프로젝트다.

`F:\unity projects\Game` 프로젝트는 폐기된 Unity 구현 프로젝트지만, 다음은 여전히 공식 UI 레퍼런스로 사용한다.

- `F:\unity projects\Game\docs\ui_scene_function_notes.md`
- `F:\unity projects\Game\docs\ui_fixed_layout_contract.md`
- `F:\unity projects\Game\tools\ima2-mcp\generated\ui-prototype\final\wireframes_ima2_fixed_contract_topbar_v2`
- `F:\unity projects\Game\docs\goal_work\03_battlefield_tile_assembler\PLAN.md`
- `F:\unity projects\Game\docs\goal_work\03_battlefield_tile_assembler\RESULT.md`
- `F:\unity projects\Game\docs\goal_work\04_combat_deployment_tile_field_assembler\RESULT.md`
- `F:\unity projects\Game\docs\tile_art_asset_handoff.md`

`Game`의 `Assets`, scene, prefab, script는 구현 대상으로 수정하지 않는다. 필요한 경우 과거 코드의 설계 의도만 확인하고, `ark` 프로젝트의 새 구조에 맞게 재작성한다.

`ark` 프로젝트에서 기존 코드가 있다면 참고해도 되는 부분:

- `Assets/Scripts/MetaGame/Network/GameWsClient.cs`: `/game` WebSocket 연결, 송수신 루프, main thread dispatch 구조.
- `Assets/Scripts/MetaGame/Network/GameWireMessages.cs`: auth, command, state_snapshot, command_result, error, notification message envelope 처리 방식.
- `Assets/Scripts/MetaGame/Core/StateStore.cs`: 최신 snapshot, allowed_actions, command result, error 보관 아이디어.
- `Assets/Scripts/MetaGame/Core/MetaGameCommandBus.cs`: request_id 생성과 command 전송 경계.
- `Assets/Scripts/MetaGame/ViewModel/SnapshotParser.cs`: map, roster, inventory, combat_preview field 파싱 아이디어.
- 전투 필드 좌표, 배치 타일, 노드맵 layout 관련 소형 유틸은 필요할 때만 검토한다.

가져오지 말아야 할 부분:

- `Assets/Scripts/MetaGame/NewFlow/Presenters/GameFlowUiRoot.cs`
- 기존 `NewFlow` 화면 구현 전체
- 기존 `TopStatusBarView`, `BottomCommandBarView`
- 기존 scene, prefab, review capture, screenshot flow
- replay/timeline 중심의 combat replay flow
- `NodeBriefing`, `CombatDeployment`, `Loadout` 등 현재 UI 문서와 맞지 않는 이전 화면 구조

기존 코드를 그대로 복사하거나 compatibility layer로 감싸지 않는다. 필요한 아이디어만 읽고, 새 프로젝트의 `unity_core_contract.md`, `ui_fixed_layout_contract.md`, `ui_scene_function_notes.md`에 맞춰 작고 명확한 새 구조로 재작성한다.

## Source Of Truth Priority

충돌이 있으면 다음 순서로 판단한다.

1. `unity_core_contract.md`: 서버 통신, snapshot, command, result, live battle delta 계약.
2. `ui_fixed_layout_contract.md`: 상단바, 하단 floating 버튼, safe area 등 공통 레이아웃 규칙.
3. `ui_scene_function_notes.md`: 장면별 기능 범위와 보류 항목.
4. `wireframes_ima2_fixed_contract_topbar_v2`: 시각 배치 참고 이미지.
5. `game_rulebook.md`: 게임 정책과 흐름 설명.

와이어프레임 이미지는 최종 아트가 아니라 레이아웃 참고다. 이미지와 계약 문서가 충돌하면 계약 문서를 우선한다.

## Objective

Unity에서 `/game` WebSocket에 연결해 다음 흐름을 실제로 조작할 수 있게 만든다.

1. 서버 연결 및 auth.
2. 초기 `state_snapshot` 수신.
3. `StartNewGame`.
4. 후보 직원 3명 선택.
5. 노드맵 표시.
6. 노드 선택 및 진입 확인.
7. DefenseRoute 전투 화면 진입.
8. 전투 중 배치, 후퇴, 수동 스킬, 일시정지, 재생, 배속, 전투 상태 요청.
9. `battle_delta` notification과 뒤따르는 `state_snapshot` 반영.
10. 전투 결과 확인 후 Safezone의 `Node Exploration` 복귀.
11. 지원, 정비, 본사 연락, 상점, 보상 노드의 기본 화면 전환.
12. safe node command는 `unity_core_contract.md`와 `allowed_actions`에 명확히 정의된 것만 연결한다.

## Node Map Entry Contract

노드맵 UX와 Unity-facing command 계약은 다음 방향으로 확정한다.

- 노드 클릭은 서버 preview request다.
- 노드 클릭 시 `/game` WebSocket command `{ type: "select_map_node", node_id: "..." }`를 보낸다.
- core는 노드를 검증하고 `NodePreview` command result와 preview/confirm 계열 snapshot을 반환한다.
- Unity는 `NodePreview` 또는 snapshot의 preview data를 읽어 노드맵 preview panel에 전장 형태, 출현 정보, 보상, 위험도 등을 표시한다.
- 하단 우측 `진입` 버튼은 서버 preview data를 받은 뒤 활성화한다.
- 하단 우측 `진입` 버튼을 누르면 `{ type: "confirm_enter_node" }`를 보내며, 이때 실제 노드에 입장한다.
- `SelectMapNode`는 preview 요청/선택 command이고, `ConfirmEnterNode`는 실제 진입 command다.
- `NodeConfirm`/`node_confirm`은 "별도 확인 화면"이나 "잠금 상태"가 아니라 노드맵의 preview-ready state다.
- `node_confirm`은 노드맵에서 서버 preview가 준비되고 실제 진입 확인을 기다리는 preview-ready state다.
- `node_confirm` 상태에서도 다른 available node를 클릭할 수 있다. 이때 Unity는 다시 `{ type: "select_map_node", node_id: "..." }`를 보내 새 preview를 요청한다.
- core는 새 `SelectMapNode`를 기존 preview/session 교체로 처리하고, `NodePreview`와 새 `node_confirm` snapshot을 반환한다.
- `ConfirmEnterNode`는 항상 마지막으로 preview된 노드에 진입한다.
- `CancelSelectedNode`는 preview/session을 해제하고 선택 없는 노드맵으로 돌아가는 command다.
- Unity UX에서는 `node_confirm`도 Node Map 레이아웃을 유지한다. 좌측/중앙 맵, 우측 preview panel, 하단 우측 `진입` CTA로 표현한다.
- 따라서 `NodeConfirm` route를 roster/loadout/slot 편집 화면으로 렌더링하지 않는다.

## Safe Node and Safezone Terminology

`Safe Node`와 `Safezone`은 다른 개념이다.

- `Safe Node`는 맵 위 비전투 노드다. `Medical`, `Rest`, `Maintenance`, `HeadquartersContact`, `Shop`, `Reward`가 여기에 속한다.
- `Support Node`는 Safe Node 중 core `SupportState`를 쓰는 하위 계열이며 `Medical`, `Rest`, `Maintenance`만 포함한다.
- `Safezone`은 다음 노드 진입 전 준비 UX다. 별도 map node가 아니고 노드를 소비하지 않는다.
- Safezone은 플레이어가 선택적으로 들어가는 장소가 아니라 게임 흐름상 노드와 노드 사이에서 항상 거치는 장면이다.
- `StartNewGame` 이후에는 직원 선발이 먼저 진행되고, 직원 선발 완료 후 Safezone의 `Node Exploration`으로 들어간다.
- 노드를 소비한 뒤에도 기본 복귀 지점은 Safezone의 `Node Exploration`이다.
- Safezone은 `Node Exploration`, `Item Use`, `Loadout` 세 장면으로 구성한다.
- `Node Exploration`은 다음 노드 preview, 맵 탐색, 진입 후보 확인을 담당한다.
- `Item Use`는 인벤토리와 사용 아이템을 보여주는 장면이다. 하단바 좌측 버튼으로 `Node Exploration`과 전환한다.
- `Item Use`의 직원 카드 아래에는 상태창이 있어야 한다. 상태창은 최소 트라우마 표시와 active consumable modifier 1칸을 포함하고, 추후 상태/버프 정보가 늘어날 수 있도록 확장 가능한 구조로 만든다.
- `Item Use` 우측 가방은 4x10 기본 슬롯 grid로 표현한다. 각 슬롯은 정사각형이며, 아이템이 없어도 빈 슬롯을 항상 표시한다.
- `Item Use` 가방은 추후 가방 크기 변경을 고려해 스크롤 가능한 구조로 만든다.
- `Item Use` 가방 패널 내부, 슬롯 위에는 `소모성`, `무기`, `방어구`, `악세서리`, `스킬 파편` 5종 카테고리 버튼을 둔다.
- `Loadout`은 `Item Use` 장면에서 유닛을 long press했을 때 열리는 장면이다. 장비 장착/해제와 스킬 파편 장착/해제를 담당한다.
- `Loadout` 우측 가방도 장기적으로 `Item Use`와 같은 카테고리/슬롯 정책을 공유한다.
- `Loadout`의 장착/해제 기본 UX는 drag-and-drop이다. 우측 가방 아이템을 중앙 호환 슬롯으로 드롭하면 장착하고, 중앙 장착 슬롯의 아이템/파편을 우측 가방으로 드롭하면 해제한다. 클릭은 장착/해제가 아니라 선택/미리보기 용도로 둔다.
- 호환되지 않는 슬롯에 드롭하면 command를 보내지 않고 시각 피드백만 표시한다.
- Safezone에서는 현재 계약상 `use_consumable_item`, 장비 장착/해제, 스킬 파편 장착/해제, 인벤토리 확인, 다음 노드 preview 확인을 live 연결한다.
- `휴식 정비` 와이어프레임은 Safezone의 `Loadout` 장면 표현으로 취급한다. 여기서 `Maintenance` 전용 강화/개화/분쇄/복원/분해/강화 command를 노출하지 않는다.
- `휴식` 와이어프레임은 Rest Node 연출이나 Safezone 분위기 연출에 쓸 수 있지만, Rest Node의 실제 효과는 core에서 `complete_node` 시점에만 적용된다.
- `Item Use` 장면에서 사용 아이템/소모품을 사망자가 아닌 직원에게 드래그하면 `use_consumable_item`을 보낸다. 사망자인 직원은 UI에서 사용 불가로 처리한다.
- 하단 좌측 `저장` 버튼은 1차 구현에서 실제 저장 기능 없이 mock 버튼으로 둔다.

## Non-Goals

이번 goal에서 하지 않는다.

- core 게임 규칙을 Unity에서 재구현하지 않는다.
- 보상/피해/스킬/웨이브 수치를 Unity에서 계산하지 않는다.
- 최종 아이콘, 초상화, 배경 아트, 세부 애니메이션을 완성하지 않는다.
- 와이어프레임 픽셀 위치를 절대값으로 하드코딩하지 않는다.
- `timeline_delta`를 사전 생성 replay로 해석하지 않는다.
- 서버 계약에 없는 legacy 요청을 되살리지 않는다.
- 임의의 mock-only 흐름을 실제 서버 흐름처럼 고정하지 않는다.

## Engineering Principles

이 작업은 규모가 크기 때문에, 초기에 부채가 쌓이면 이후 UI/서버 연동 전체를 다시 뜯어고치게 된다. 구현자는 다음 원칙을 따른다.

- 코드 수정은 장기적인 방향으로 한다. 임시방편, 최소한의 수정, 화면 하나만 겨우 맞추는 패치는 피한다.
- 처음에는 어떤 구조가 장기적인 방향인지 확실하지 않을 수 있다. 이 경우 작은 단위로 trial and error를 수행하고, 실패한 접근과 이유를 `EXPERIMENTS.md`에 남긴다.
- 레거시는 과감하게 제거한다. 다만 기존 Unity 작업물을 무차별 삭제하지 않는다. 서버 계약과 충돌하거나 실제로 사용되지 않음이 확인된 legacy mock flow, prefab, command adapter, 과거 replay/timeline 해석 코드만 제거하고, 제거 근거를 작업 메모에 남긴다.
- 문서를 무조건 신뢰하지 않는다. 실제 Unity 코드, prefab 구조, server response, `unity_core_contract.md`의 최신 계약을 읽으면서 더 나은 개선안이 보이면 그 근거를 기록하고 적용한다.
- 단, Unity-facing DTO 변경, 서버 계약 변경, 게임 정책 변경처럼 사용자 결정이 필요한 내용은 임의로 확정하지 않고 goal을 종료한 뒤 질문 목록을 보고한다.
- 새 추상화는 실제 반복이 확인된 뒤 만든다. 화면이 1~2개뿐인데 공통 프레임워크, 복잡한 generic, 과한 manager 계층을 먼저 만들지 않는다.
- UI 리소스와 위치는 앞으로 바뀔 수 있다. 그래서 hard-coded absolute layout보다 prefab variant, layout group, anchor, data binding 경계를 우선한다.
- 테스트와 검증은 내부 클래스 모양보다 실제 플레이 흐름과 서버 계약을 고정해야 한다.
- 작업 중 중요한 결정을 내리거나 구조를 바꾸면 `PLAN.md`와 `EXPERIMENT_NOTES.md`를 함께 갱신한다.

## UI Architecture Boundary

현재 UI 레이아웃과 화면 흐름은 어느 정도 정해졌지만, 최종 텍스쳐는 아직 확정되지 않았다. 전투 필드 표현은 `3D tile-block battlefield + 2D sprite/spine units`로 고정한다. 따라서 Unity 구현은 데이터 계층과 표현 계층을 분리하되, 순수 2D tilemap 전장이나 3D 캐릭터 모델 전투를 만들지 않는다.

권장 흐름:

```text
game_server / game_core contract
-> Unity DTO / Snapshot Store
-> Screen Router
-> Screen Presenter / ViewModel
-> View Adapter
-> 3D Tile Battlefield View + 2D Sprite/Spine Unit View
```

각 계층의 책임:

- `GameClient`: `/game` WebSocket 송수신과 message envelope 처리.
- `GameSnapshotStore`: 최신 `state_snapshot`, pending command, battle delta 누적 상태 보관.
- `GameScreenRouter`: `game_state_context.type`을 보고 어떤 화면을 열지 결정.
- `ScreenPresenter`: snapshot과 command result를 화면별 ViewModel로 변환.
- `ViewModel`: 화면에 필요한 표시 데이터만 가진다. prefab, texture, material, world position, animation reference를 넣지 않는다.
- `View Adapter`: ViewModel을 실제 View가 이해하는 호출로 연결한다.
- `View`: 실제 Unity 오브젝트 계층이다. 일반 UI는 UGUI/UIToolkit을 쓸 수 있지만, 전투 필드는 3D tile-block view와 2D sprite/spine unit view로 구현한다.

피해야 할 구조:

- `MonoBehaviour`가 서버 DTO를 직접 파싱한다.
- 버튼 클릭에서 WebSocket JSON을 직접 만든다.
- 서버 snapshot 필드와 prefab hierarchy가 직접 연결된다.
- 텍스쳐, material, mesh prefab, sprite/spine asset reference가 ViewModel에 들어간다.
- mock UI 데이터와 live server 데이터가 같은 클래스에 섞인다.

권장 최소 인터페이스 예시:

```csharp
public sealed class EmployeeSelectionViewModel
{
    public IReadOnlyList<EmployeeCandidateVm> Candidates { get; init; }
    public IReadOnlyList<EmployeeCandidateVm> Selected { get; init; }
    public bool CanConfirm { get; init; }
}

public interface IEmployeeSelectionView
{
    void Render(EmployeeSelectionViewModel vm);
}
```

이 구조의 목적은 거대한 MVVM 프레임워크를 만드는 것이 아니다. 최종 텍스쳐와 리소스가 바뀌어도 core 계약, snapshot store, 화면 presenter, command binding을 다시 만들지 않게 하는 것이다.

## Dynamic Battlefield Tile Assembly

전투 필드는 고정 배경 이미지가 아니다. core/RON의 ASCII battlefield template과 `/game` snapshot의 `combat_preview` 데이터를 기반으로 Unity가 3D 큐브/타일 블록으로 동적으로 조립해야 한다. 유닛은 3D 모델이 아니라 2D sprite/spine actor로 고정한다.

핵심 의도:

- core/RON에서 전투 맵은 ASCII 형태로 정의된다.
- ASCII 맵은 복도, ㄱ자, split room, 포위형, 비정형 방처럼 매번 형태가 다를 수 있다.
- Unity는 맵마다 완성 배경 이미지를 따로 요구하지 않는다.
- Unity는 `combat_preview.tiles`, `deployment_zones`, `spawn_zones`, `routes`, `spawn_waves` 같은 서버 데이터를 source of truth로 삼아 전투 필드를 조립한다.
- 최소 단위는 `1x1 tile`이다.
- `width`와 `height`는 전체 사각형을 채우라는 의미가 아니라 bounds/framing hint다.
- 실제 타일은 `combat_preview.tiles`에 있는 좌표만 생성한다.
- `combat_preview.tiles`에 없는 좌표는 void다. void에는 floor, hit target, drop target, movement target을 만들지 않는다.
- `valid_tiles`는 기존 계약/검증용 보조 필드다. Unity 전장 렌더링은 `valid_tiles`에서 tile kind를 추론하지 않는다.

기본 타일 종류:

- `Ground`: 일반 지형. 근거리 아군 유닛과 적 유닛이 이동할 수 있는 타일이다.
- `Platform`: 원거리/힐러 등 플랫폼 배치 타입을 가진 아군이 배치될 수 있는 타일이다. 적은 이동할 수 없고, 적에게는 장애물처럼 취급한다.
- `Obstacle`: 아군/적 모두 배치하거나 지나갈 수 없는 타일이다.
- `Void`: 맵 외부/빈 공간이다. 타일 자체가 생성되지 않아야 한다.

타일/배치 계약:

- Unity는 `Platform`을 `deployment_zones`에서 임의 추론하지 않는다.
- `Ground`, `Platform`, `Obstacle` 같은 tile kind는 `combat_preview.tiles[*].kind`에서 읽는다.
- 유닛은 배치 허용 타입을 가진다. 예: `ground_only`, `platform_only`, `any`.
- `Ground`에는 ground 배치 flag가 있는 유닛만 배치할 수 있다.
- `Platform`에는 platform 배치 flag가 있는 유닛만 배치할 수 있다.
- 직원별 현재 배치 허용 타입은 `roster.employees[*].combat_profile.effective_deployment_affinity`를 우선 사용한다.
- Unity는 배치 가능 타일을 미리 표시할 수 있지만, 최종 배치 가능 여부는 항상 core validation 결과를 따른다.

표현 정책:

- 전투 맵은 3D 큐브/타일 블록 기반이다.
- `Ground`, `Platform`, `Obstacle`은 서로 다른 prefab/material/height를 가진다.
- 큐브 높이는 tile kind별로 다르게 둔다. platform과 obstacle은 깊이감을 위해 ground보다 높게 표현한다.
- `Void`는 아무 오브젝트도 만들지 않는다.
- 카메라는 고정이다. 1차 구현에서 pan/zoom/rotate 조작을 만들지 않는다.
- 유닛은 2D sprite/spine actor로 고정한다. 3D 캐릭터 모델을 만들지 않는다.
- 유닛은 tile cube의 top center 위에 배치한다.
- 배치 방향은 명일방주처럼 sprite/spine 방향 전환 또는 좌우 반전으로 표현한다.
- 방향 표시는 sprite 연출을 우선하고, 필요하면 발밑 marker/arrow를 보조로 사용한다.

권장 Unity 구조:

```text
CombatPreview / BattleState field data
-> BattlefieldFieldModel
-> BattlefieldTileMapBuilder or BattlefieldFieldAssembler
-> Tile Presentation Adapter
-> 3D Tile Cube/Block Prefabs + 2D Sprite/Spine Unit Actors
```

Builder/Assembler 책임:

- `combat_preview.tiles`만 순회해 `Ground`, `Platform`, `Obstacle` tile을 생성한다.
- void와 맞닿은 missing neighbor 방향에 edge/rim/boundary 표현을 만든다.
- `tiles.kind == Obstacle` 좌표에 obstacle prop 또는 blocker presentation을 만든다. `obstacles` 배열은 consistency/debug 표시용으로만 사용한다.
- `deployment_zones`는 deploy overlay로 표시한다.
- `spawn_zones`는 hostile/spawn overlay로 표시한다.
- `routes`는 적 이동 경로 preview 또는 tactical line으로 표시할 수 있게 데이터를 보존한다.
- hover/selection/invalid drop feedback overlay를 타일 표시와 분리한다.
- tile coordinate와 Unity world position 변환을 한 곳에서 담당한다.

피해야 할 것:

- `width`/`height` 내부를 full rectangle grid로 채우지 않는다.
- `7x5`, `7x8`, 고정 deployment row 같은 하드코딩을 만들지 않는다.
- 전투 필드를 UI button grid, card grid, 임시 텍스트 placeholder로 만들지 않는다.
- 배치 가능 여부를 배경 이미지나 색상에서 추론하지 않는다.
- 플랫폼/장애물/void 판정을 Unity가 임의로 계산하지 않는다. 서버 snapshot/core 계약이 기준이다.
- 과거 `Game` 프로젝트의 `NewFlowBattlefieldTileAssembler`, `BoardRoot`, `NewFlowDeploymentWorldOverlay`를 그대로 복사하지 않는다. 설계 의도와 검증 포인트만 참고해 `ark` 구조에 맞게 재작성한다.

권장 첫 구현:

- 최종 타일 아트가 없어도 단색 3D cube/block prefab과 material로 `Ground`, `Platform`, `Obstacle`, `DeployOverlay`, `SpawnOverlay`, `RouteArrow`를 구분한다.
- 이후 `Assets/Art/CombatTiles` 같은 조립형 리소스 팩으로 교체 가능한 구조를 유지한다.
- corridor, L-shape, split room, sparse/irregular room fixture를 mock snapshot으로 만들어 full rectangle 오염이 없는지 검증한다.
- 전투 시작 후 deploy/spawn overlay는 상황에 따라 숨길 수 있어야 한다.

Route preview 정책:

- 적 경로는 전투 시작 시 처음에 한 번 보여주고 사라진다.
- 단순 static line을 계속 켜두지 않는다.
- 시작 지점에서 화살표가 나타나 route를 따라 도착 지점까지 이동한 뒤 사라지는 연출을 기본으로 한다.
- route preview는 전투 정보를 과하게 계속 노출하는 HUD가 아니라, 전투 시작 전/초기 전장 이해를 돕는 짧은 안내 연출이다.

## Layout Rules

공통 UI는 `ui_fixed_layout_contract.md`를 따른다.

- 상단바는 책갈피형 고정 top bar다.
- 좌측 노드블록은 모든 top-bar 화면에서 고정 폭을 유지한다.
- 중앙 resource block은 있을 때만 내용을 채우고, 없으면 frame 구조를 유지한 채 비워둔다.
- 우측 info block은 있을 때 항상 오른쪽 끝에 붙인다.
- 하단은 solid bar가 아니라 floating button이다.
- 본문은 top bar와 bottom floating controls safe area를 침범하지 않는다.
- 장면별 상단바를 이미지마다 다르게 재해석하지 않는다.

Battle Result는 현재 계약상 top bar 없는 결과 화면으로 취급한다.

## Core Integration Rules

Unity는 Rust 내부 구조를 추측하지 않는다.

- 화면 전환은 `state_snapshot.state.game_state_context.type`을 기준으로 한다.
- 버튼 활성화는 `allowed_actions`를 기준으로 한다.
- command 성공 후 즉시 payload를 반영할 수 있지만, 최종 UI 상태는 뒤따르는 `state_snapshot`을 신뢰한다.
- 실패는 `error` message로 처리한다.
- `request_id`는 Unity가 생성하고 `command_result.request_id`와 매칭한다.
- `selected_event`는 이름이 낡았지만 현재 활성 노드 콘텐츠로 취급한다.
- DefenseRoute 배치 비용 표시는 `game_state_context.deployment.unit_deploy_costs[*].effective_deploy_cost`를 우선 사용한다. `base_deploy_cost`와 `redeploying_units[*].deploy_cost`는 기준값이며, consumable modifier가 적용된 직원별 실제 비용을 Unity에서 재계산하지 않는다.
- 비전투 safe node command는 `unity_core_contract.md`에 command와 payload가 명시되어 있고, 현재 snapshot의 `allowed_actions`에 노출된 경우에만 live 연결한다.
- safe node action이 아직 계약에 없으면 layout, selection state, disabled/placeholder button, mock callback까지만 구현하고, 누락된 계약을 질문 목록으로 보고한다.
- Safezone은 별도 서버 상태를 추론해 만들지 않는다. 하위 장면 전환은 Unity UX 상태로 처리하고, 현재 snapshot에서 허용된 `use_consumable_item`, 장착/해제 및 preview command만 live 연결한다. 나머지는 화면 표현 또는 disabled placeholder로 둔다.
- node 완료 이후 기본 복귀 지점은 Safezone의 `Node Exploration`이다. 단, 최신 `state_snapshot.game_state_context.type`이 다른 상태를 가리키면 snapshot을 우선한다.

## Scene Flow Clarifications

Node Confirm은 1차 구현에서 별도 scene으로 만들지 않는다. Node Map의 우측 상세/확정 패널로 우선 처리한다. 추후 연출상 독립적인 Node Briefing 화면이 필요하면 그때 분리한다.

Node Confirm은 Node Map을 잠그는 상태가 아니다. 사용자는 preview-ready 상태에서도 다른 available node를 클릭해 비교할 수 있다. 다른 노드를 클릭하면 Unity는 다시 `SelectMapNode`를 보내고, 서버가 내려준 새 `NodePreview`/snapshot으로 같은 Node Map의 preview 패널을 교체한다. 우하단 진입 버튼은 `ConfirmEnterNode`가 `allowed_actions`에 있을 때만 활성화하고, 클릭 시 마지막으로 preview된 노드에 진입한다.

HQ briefing dialogue는 생략하지 않는다. 최종 대사와 연출이 확정되지 않았더라도 `01_hq_briefing_dialogue_v1.png` 계열 흐름을 기준으로 최소 더미 대사 2-3줄, 다음 버튼, 종료 후 직원 선발 이동을 구현한다.

## DefenseRoute Live Battle Rules

현재 공식 전투는 DefenseRoute 실시간 전투다.

- 전투는 사전 생성된 전체 replay를 재생하는 방식이 아니다.
- `battle_delta` notification의 `timeline_delta`는 battle event log delta다.
- Unity는 `timeline_delta[*].seq` 기준으로 중복 적용을 막는다.
- 서버는 `battle_delta` 뒤에 최신 `state_snapshot`도 push한다.
- 배치 가능 타일, route, spawn wave, enemy briefing은 `combat_preview`와 live state를 기반으로 표시한다.
- 전투 필드는 `combat_preview`/battle state 기반의 dynamic tile assembly로 구성한다. 고정 배경 이미지를 전투 맵의 source of truth로 쓰지 않는다.
- 적 route preview는 전투 시작 시 한 번 화살표가 경로를 따라 이동한 뒤 사라지는 방식으로 표시한다.
- 플레이어 유닛 배치는 전투 중 command로 수행한다.
- 배치 방향은 필수이며, 배치 후 방향 변경은 하지 않는다. 방향을 바꾸려면 후퇴 후 재배치한다.
- 수동 스킬은 Unity 입력으로 `activate_skill` command를 보낼 때만 발동한다.
- Pause/Resume/SetBattleSpeed는 서버 전투 시뮬레이션 상태를 바꾸는 명령이다. Unity 화면만 멈추는 기능으로 처리하지 않는다.
- 전투 노드는 이상현상이며 최대 3번까지 진입 시도할 수 있다. Unity는 NodeConfirm/전투/후퇴 확인 흐름에서 서버 snapshot이 제공하는 남은 시도 횟수와 퇴각 가능 여부를 표시해야 한다.
- NodePreview/CombatPreview의 `threat_warnings`는 진입 전 경고문으로 표시한다. 정확한 적 방어력/마법 저항 숫자나 도감 UI는 만들지 않는다.
- `threat_warnings[*].status == "Disproved"`인 경고는 같은 이상현상 재진입 preview에서 취소선 처리한다. Unity는 루머성 오경고를 직접 랜덤 생성하지 않고 core가 내려준 항목만 표시한다.
- 전투 중 피해 숫자는 `HpChanged.damage_type`과 `HpChanged.feedback_tags`를 사용해 표시한다. UI 표현에서 AD는 `Physical`, AP는 `Magic`에 대응한다. `Physical`은 붉은 계열, `Magic`은 푸른 계열로 숫자 색을 나누고, `True`는 고정 피해 색상 또는 neutral 색상으로 둔다.
- 피해 숫자 상단에는 `critical`, `mitigated`, `fixed_damage`, `immune`만 표시한다. 예시는 `치명타!`, `경감됨!`, `고정 피해!`, `면역!`이다. 상단 우선순위는 `immune > fixed_damage > mitigated > critical`이다.
- 피해 숫자 좌측에는 그 외 보조 태그를 붙인다. 초기에는 비어 있어도 되며, 추후 `piercing`, `shield`, `blocked`, `resisted_status` 같은 태그가 생기면 작은 라벨/아이콘으로 표시한다.
- Unity는 `feedback_tags`를 재계산하지 않는다. 라벨 표시 여부와 우선순위만 클라이언트 표현으로 처리한다.

Unity-facing 필드인 `timeline_delta`, `compressed_timeline`, `has_timeline`은 이름이 timeline이지만 현재 의미는 event log다. 임의로 필드명을 바꾸지 않는다.

## Recommended Implementation Order

1. goal 작업 기억장치인 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 만든다.
2. 현재 Unity 프로젝트 구조, scene, prefab, existing UI flow, asmdef, package dependency를 조사하고 결과를 `PLAN.md`에 요약한다.
3. 문서와 이미지 import 경로를 확인한다.
4. WebSocket client와 message envelope를 구현한다.
5. `state_snapshot`, `command_result`, `error`, `notification/battle_delta` 수신 파이프라인을 만든다.
6. snapshot store를 만들고 UI는 이 store만 바라보게 한다.
7. `allowed_actions` 기반 command gate를 만든다.
8. 공통 top bar와 bottom floating controls prefab을 만든다.
9. HQ briefing dialogue를 최소 더미 대사 흐름으로 구현한다.
10. 직원 선발 화면을 구현하고 `StartNewGame` 이후 선택 흐름을 연결한다.
11. 노드맵 화면을 구현하고 노드 클릭 시 `SelectMapNode`로 서버 preview를 요청한다.
12. 서버 preview data를 노드맵 preview panel에 표시하고, 하단 우측 `진입` CTA로 `ConfirmEnterNode` 실제 진입을 요청한다.
13. `CombatPreview` 기반 `BattlefieldTileMapBuilder` 또는 동등한 Assembler를 구현한다.
14. DefenseRoute 전투 화면을 구현하고 중앙 필드는 dynamic tile assembly로 표시한다.
15. 배치/철수/스킬/후퇴/정지/재생/배속/상태요청 command를 연결한다.
16. `battle_delta` event log를 누적 적용해 전투 UI를 갱신한다.
17. 전투 결과와 `CompleteCombatResult`를 연결한다.
18. Safezone/pre-node preparation을 `Node Exploration`, `Item Use`, `Loadout` 세 장면으로 구현한다. `Node Exploration`과 `Item Use`는 하단바 좌측 버튼으로 전환하고, `Loadout`은 `Item Use`에서 유닛 long press로 연다. 현재 계약의 `use_consumable_item`, 장비/파편 장착/해제, preview 확인을 live 연결한다.
19. Medical, Rest, Maintenance, HeadquartersContact, Shop, Reward의 기본 화면과 나가기/확정 흐름을 구현한다.
20. safe node command는 계약이 명확한 것만 live 연결하고, 나머지는 disabled/placeholder로 둔다.
21. mock data와 live data path를 분리해, 서버 연결 없이도 레이아웃 확인이 가능하게 한다.

## Scene Coverage Checklist

최소 구현해야 하는 화면:

- HQ briefing dialogue. 최소 더미 대사 2-3줄, 다음 버튼, 직원 선발 이동을 포함한다.
- Employee Selection.
- Node Map. 좌측/중앙은 맵, 우측은 선택된 노드 preview, 하단 우측은 `진입` CTA다.
- Safezone / Pre-node Preparation. `Node Exploration`, `Item Use`, `Loadout` 장면을 가진다. `Node Exploration`은 다음 노드 preview, `Item Use`는 인벤토리/사용 아이템 표시와 사망자가 아닌 직원 대상 아이템 사용, `Loadout`은 장비/파편 장착 상태를 보여준다. `Item Use` 직원 카드 하단에는 트라우마와 active consumable modifier 1칸을 포함한 확장 가능한 상태창이 있어야 한다. `Item Use` 우측 가방은 5종 카테고리 버튼과 4x10 정사각형 슬롯 grid, 빈 슬롯 표시, 스크롤 가능한 구조를 가진다. `Loadout`의 장착/해제 UX는 우측 가방과 중앙 장착 슬롯 사이의 drag-and-drop을 기본으로 한다. Maintenance 전용 작업은 노출하지 않는다.
- DefenseRoute Battle.
- Dynamic Battlefield Tile Assembly. `Ground`, `Platform`, `Obstacle`, `Void`, deployment/spawn overlay가 구분되어야 한다.
- Battle Result.
- Medical.
- Rest.
- Maintenance.
- Headquarters Contact.
- Shop.
- Reward.
- Error/Disconnected overlay.

각 화면은 최소한 다음을 만족한다.

- 현재 `game_state_context.type`을 표시하거나 내부적으로 로깅한다.
- 현재 `allowed_actions`에 따라 주요 버튼을 활성/비활성화한다.
- command 전송 실패 시 error overlay 또는 toast를 표시한다.
- 임시 UI 데이터와 서버 snapshot 데이터가 코드상 분리되어 있다.

## Validation

작업 중 빠른 검증 루프를 유지한다.

- Unity Play Mode에서 서버 연결 없이 mock snapshot으로 각 화면을 렌더링한다.
- `ui_scene_function_notes.md`의 주요 장면이 mock snapshot path에서 모두 렌더링되는지 확인한다.
- 로컬 `game_server`에 `/game` WebSocket으로 접속해 auth와 초기 snapshot 수신을 확인한다.
- `StartNewGame -> 직원 선택 -> Safezone(Node Exploration) -> 노드 클릭 preview -> 진입 -> DefenseRoute 전투 -> 결과 -> Safezone(Node Exploration) 복귀`를 한 번 이상 실제 서버로 통과시킨다.
- `battle_delta` 수신 후 같은 `seq`를 중복 적용하지 않는지 로그로 확인한다.
- Pause 상태에서 전투 시간이 증가하지 않는지 확인한다.
- SetBattleSpeed `0.5x`, `1x`, `2x`, `3x` 표시와 command payload가 맞는지 확인한다.
- corridor, L-shape, split room, sparse/irregular room mock field가 full rectangle으로 오염되지 않고 조립되는지 확인한다.

## Stop Conditions

다음 경우 임의 구현하지 말고 작업을 멈추고 질문 목록을 보고한다.

- `unity_core_contract.md`에 없는 command나 snapshot 필드가 필요하다.
- Unity-facing DTO 필드명을 바꿔야 한다.
- `game_rulebook.md`와 core snapshot/command 계약이 충돌한다.
- 특정 UI 행동이 게임 정책을 새로 정해야만 구현 가능하다.
- 서버가 주지 않는 데이터를 Unity에서 추론해야 한다.
- `combat_preview.tiles[*].kind` 또는 `roster.employees[*].combat_profile.effective_deployment_affinity`가 실제 서버 snapshot에서 누락되어 문서와 런타임 계약이 다르다.
- wireframe과 `ui_fixed_layout_contract.md`가 충돌한다.
- 세션이 길어져 컨텍스트가 불안정해졌는데 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`가 최신 상태가 아니다.

## Completion Criteria

goal 완료 조건:

1. Unity가 `/game` WebSocket에 연결하고 auth 후 snapshot을 수신한다.
2. `allowed_actions` 기반으로 주요 버튼이 열리고 닫힌다.
3. 최소 플레이 흐름이 서버와 실제로 연결된다.
4. DefenseRoute battle delta를 event log로 소비한다.
5. `CombatPreview` 기반 dynamic battlefield tile assembly가 구현되어 있고, `combat_preview.tiles`만 타일을 생성한다.
6. `Ground`, `Platform`, `Obstacle`, `Void`, deploy overlay, spawn overlay가 데이터상/표현상 구분된다.
7. 전투 필드는 3D cube/block으로 렌더링되고, 유닛은 2D sprite/spine actor로 렌더링된다.
8. tile kind별 높이가 다르게 표현된다.
9. 적 route preview는 전투 시작 시 화살표가 경로를 따라 이동한 뒤 사라진다.
10. corridor, L-shape, split room, sparse/irregular room mock field가 full rectangle으로 오염되지 않는다.
11. top bar와 bottom floating controls가 `ui_fixed_layout_contract.md`의 공통 규칙을 따른다.
12. mock UI path와 live server path가 분리되어 있다.
13. `ui_scene_function_notes.md`의 주요 장면이 mock snapshot path에서 모두 렌더링된다.
14. safe node command는 계약이 명확한 것만 live 연결되고, 미정 계약은 disabled/placeholder와 질문 목록으로 남는다.
15. `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`에 최종 계획, 주요 시도 결과, 남은 질문/부채가 기록되어 있다.
16. Unity-facing 계약 변경이 필요하면 임의 확정하지 않고 사용자에게 질문한다.
17. 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Goal Command

```text
/goal F:\work\simulator\core_enhanced\docs\unity_client_implementation_goal.md를 기준으로, Unity 클라이언트가 game_server의 /game WebSocket과 연결되어 현재 core 계약을 실제로 조작할 수 있는 1차 클라이언트 골격을 구현하라.

반드시 먼저 읽을 문서:
- F:\unity projects\Game\docs\ui_scene_function_notes.md
- F:\unity projects\Game\docs\ui_fixed_layout_contract.md
- F:\unity projects\Game\tools\ima2-mcp\generated\ui-prototype\final\wireframes_ima2_fixed_contract_topbar_v2
- F:\work\simulator\core_enhanced\docs\unity_core_contract.md

핵심 구현 범위:
- goal 작업 시작 시 F:\unity projects\ark\docs\goals\unity_client_implementation\PLAN.md, EXPERIMENTS.md, EXPERIMENT_NOTES.md를 만들고 계속 갱신
- F:\unity projects\ark의 scene/prefab/script/asmdef 구조 확인
- F:\unity projects\Game은 폐기된 구현 프로젝트지만 docs와 wireframe 이미지는 공식 UI 레퍼런스로 읽기
- F:\unity projects\Game의 Assets/scene/prefab/script는 수정하지 말고 그대로 복사하지 않기
- ark 프로젝트에 기존 코드가 있다면 GameWsClient, GameWireMessages, StateStore, MetaGameCommandBus, SnapshotParser, 전투 필드/노드맵 소형 유틸의 아이디어만 참고
- 기존 NewFlow UI, GameFlowUiRoot, TopStatusBarView, BottomCommandBarView, replay/timeline 중심 flow는 가져오지 않기
- WebSocket auth, snapshot 수신, command_result/error/notification 파이프라인
- SnapshotStore, ScreenRouter, Presenter/ViewModel, View Adapter, 3D tile battlefield view, 2D sprite/spine unit view 경계 분리
- CombatPreview 기반 dynamic battlefield tile assembly 구현. combat_preview.tiles만 3D tile cube/block을 생성하고 Ground/Platform/Obstacle/Void/deploy overlay/spawn overlay를 구분
- tile kind는 combat_preview.tiles[*].kind, 유닛 배치 허용 타입은 roster.employees[*].combat_profile.effective_deployment_affinity를 우선 사용. 실제 서버 snapshot에서 이 필드가 문서와 다르게 누락되면 Unity에서 임의 추론하지 말고 goal 종료 후 질문 목록으로 보고
- 공통 top bar와 bottom floating controls
- HQ briefing dialogue, 직원 선발, Safezone(Node Exploration/Item Use/Loadout), Node Confirm panel, DefenseRoute 전투, 전투 결과, Safezone 복귀
- 배치/철수/수동 스킬/후퇴/정지/재생/0.5x/1x/2x/3x 배속, battle_delta event log 적용
- Medical/Rest/Maintenance/HeadquartersContact/Shop/Reward는 기본 화면을 만들되, command와 payload가 unity_core_contract.md 및 allowed_actions에 명확한 것만 live 연결

원칙:
- 화면 전환은 game_state_context.type, 버튼 활성화는 allowed_actions, 서버 연동은 unity_core_contract.md를 source of truth로 삼아라.
- 와이어프레임은 레이아웃 참고이며 최종 아트나 절대 좌표로 하드코딩하지 마라.
- core 게임 규칙을 Unity에서 재구현하지 말고, timeline_delta를 replay로 해석하지 말고, 서버 계약에 없는 legacy 요청을 되살리지 마라.
- 전투 맵은 3D cube/block, 유닛은 2D sprite/spine actor로 구현하라. 순수 2D tilemap 전장이나 3D 캐릭터 유닛을 만들지 마라.
- 적 route preview는 전투 시작 시 화살표가 경로를 따라 이동한 뒤 사라지는 방식으로 구현하라.
- 코드 수정은 장기적인 방향으로 하고, trial and error를 EXPERIMENTS.md에 기록하며, 서버 계약과 충돌하거나 실제로 사용되지 않음이 확인된 레거시는 근거를 남기고 제거하라.
- 계획이 바뀌면 PLAN.md를 갱신하고, 작업 중 떠오른 판단/질문/부채는 EXPERIMENT_NOTES.md에 남겨라.
- mock UI path와 live server path를 분리하라.
- 기존 코드를 그대로 복사하거나 compatibility layer로 감싸지 말고, 필요한 아이디어만 새 구조에 맞게 재작성하라.
- 문서를 무조건 신뢰하지 말고 실제 Unity 코드, prefab 구조, server response, core 계약을 읽으면서 더 나은 개선안이 있으면 근거를 기록하고 적용하라.

완료 조건:
- 주요 장면이 mock snapshot path에서 렌더링된다.
- /game WebSocket으로 StartNewGame -> 직원 선택 -> Safezone(Node Exploration) -> 노드 클릭 preview -> 진입 -> DefenseRoute 전투 -> 결과 -> Safezone(Node Exploration) 복귀를 최소 1회 통과한다.
- corridor, L-shape, split room, sparse/irregular room mock field가 combat_preview.tiles 기반 dynamic tile assembly로 렌더링되고 full rectangle으로 오염되지 않는다.
- 전투 필드는 3D cube/block이고 유닛은 2D sprite/spine actor로 표시된다.
- 적 route preview는 전투 시작 시 화살표가 경로를 따라 이동한 뒤 사라진다.
- battle_delta는 event log로 seq 중복 없이 소비한다.
- safe node 미정 command는 disabled/placeholder와 질문 목록으로 남긴다.
- PLAN.md, EXPERIMENTS.md, EXPERIMENT_NOTES.md에 최종 계획, 주요 시도 결과, 남은 질문/부채가 기록되어 있다.
- Unity-facing DTO 필드명 변경, 서버 계약에 없는 데이터 필요, 게임 정책 결정이 필요한 모호함, 또는 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료하고 질문 목록을 보고하라.
```
