# 노드 편집기: 기존 구현 분석과 적용 결정

2026-10-20. 노드 편집기 기반 구현의 0단계 기록이다. 아래는 고정된 버전의 소스를
읽어 확인한 결과이며 Blender/Godot을 빌드하거나 성능을 비교한 결과는 아니다.
게임 스킬의 실행 의미, 셰이더 평가기, 전체 에디터 프레임워크는 분석 범위에서 제외한다.

## 기준 버전과 추적 범위

- Blender `v4.5.0`, commit `8cb6b388974a817afedf1317ce26f0c75aa5f181`.
- Godot `4.5-stable`, commit `876b290332ec6f2e6d173d08162a02aa7e6ca46d`.

추적한 경로는 문서와 화면의 분리, 이동의 시작/미리보기/종료, 연결 교체,
삭제·복제의 Undo 단위, 파생 정보 갱신, 저장할 모델의 경계다.
아래 링크는 모두 해당 commit에 고정했다. 외부 구현 코드는 제품에 복사하지 않는다.

## 1. 문서와 화면

Blender는 노드에 영속 ID, 종류, 소켓, 부모, 좌표를 두고 런타임 자료를 별도로 둔다.
소켓에도 표시 이름과 별개의 식별자가 있다. 연결은 노드와 소켓을 참조한다.
근거: [DNA_node_types.h](https://github.com/blender/blender/blob/8cb6b388974a817afedf1317ce26f0c75aa5f181/source/blender/makesdna/DNA_node_types.h#L117),
[노드 정의](https://github.com/blender/blender/blob/8cb6b388974a817afedf1317ce26f0c75aa5f181/source/blender/makesdna/DNA_node_types.h#L410).

Godot의 GraphEdit은 포트 연결/해제 요청을 신호로 보낸다. VisualShaderEditor가 이를 받아
VisualShader 모델과 graph plugin의 화면 갱신을 조정한다. GraphEdit 하나만으로 저장 모델이나
편집 의미가 완성되지 않는다.
근거: [GraphEdit 연결 요청](https://github.com/godotengine/godot/blob/876b290332ec6f2e6d173d08162a02aa7e6ca46d/scene/gui/graph_edit.cpp#L1178),
[VisualShaderEditor 연결 처리](https://github.com/godotengine/godot/blob/876b290332ec6f2e6d173d08162a02aa7e6ca46d/editor/shader/visual_shader_editor_plugin.cpp#L4212).

**우리 결정:** TypeScript Document가 위치·속성·연결·프레임의 원본이다. React Flow 노드는
그 문서의 표시 자료다. 선택·줌·호버와 미확정 드래그는 UI에 둔다. 포트는 카탈로그의
안정적인 문자열 ID로 참조하며, 이름이나 화면 순서로 연결하지 않는다. 카탈로그 변경이
저장 파일의 포트 의미를 바꿀 때에는 명시적인 형식 변경/변환이 필요하다.

## 2. 이동: 시작 → 표시 → 확정/취소

Blender의 transform adapter는 시작 좌표를 준비하고 `flushTransNodes`에서 문서의
`node.location`을 갱신한다. 종료 처리에서 취소 여부를 구분하여 부모 관계 복원과
후속 연결/프레임 처리를 수행한다. 따라서 Blender가 드래그 내내 문서를 불변으로
유지한다고 해석해서는 안 된다.
근거: [좌표 준비](https://github.com/blender/blender/blob/8cb6b388974a817afedf1317ce26f0c75aa5f181/source/blender/editors/transform/transform_convert_node.cc#L51),
[좌표 반영과 종료](https://github.com/blender/blender/blob/8cb6b388974a817afedf1317ce26f0c75aa5f181/source/blender/editors/transform/transform_convert_node.cc#L335).

Godot은 GraphEdit에서 화면 좌표를 갱신하고 놓을 때 `set_drag(false)`를 호출한다.
GraphElement가 시작/끝 위치를 `dragged`로 전달하고, VisualShaderEditor는 이를 버퍼에
모아 여러 노드의 이동을 하나의 Undo 작업에 넣는다.
근거: [GraphEdit 이동](https://github.com/godotengine/godot/blob/876b290332ec6f2e6d173d08162a02aa7e6ca46d/scene/gui/graph_edit.cpp#L2002),
[드래그 종료](https://github.com/godotengine/godot/blob/876b290332ec6f2e6d173d08162a02aa7e6ca46d/scene/gui/graph_edit.cpp#L2111),
[GraphElement 신호](https://github.com/godotengine/godot/blob/876b290332ec6f2e6d173d08162a02aa7e6ca46d/scene/gui/graph_element.cpp#L129),
[이동 묶음](https://github.com/godotengine/godot/blob/876b290332ec6f2e6d173d08162a02aa7e6ca46d/editor/shader/visual_shader_editor_plugin.cpp#L4167).

**우리 결정:** UI에서 좌표 미리보기를 갱신하고 놓을 때 `Move { positions }`를 한 번
보낸다. 시작 시 문서 세대·리비전을 잡으며, 실패·Escape·포인터 취소는 원본 좌표로
돌린다. 프레임 이동도 소속 노드 전체의 위치 변경 한 건이다. 위치는 저장과 Undo에
포함한다. 기존 스킬 화면의 세션 전용 위치 구현을 새 편집기의 모델로 사용하지 않는다.

## 3. 연결과 편집 원자성

Blender의 `add_dragged_links_to_tree`는 점유된 소켓의 기존 연결 처리, 노드별 삽입 검사,
새 연결 추가, 변경 표시와 불변식 갱신을 조정한다.
근거: [연결 적용](https://github.com/blender/blender/blob/8cb6b388974a817afedf1317ce26f0c75aa5f181/source/blender/editors/space_node/node_relationships.cc#L1182).

Godot의 `_connection_request`는 호환성을 먼저 검사하고 기존 입력 연결의 제거와
새 연결 추가를 같은 Undo 작업에 등록한다. 별도의 `_disconnection_request`는 독립된
Undo 작업을 만든다. 따라서 이 구현 전체를 그대로 적용하면 우리 재연결 동작의
취소/Undo 계약과 일치한다고 가정할 수 없다.
근거: [연결/해제 처리](https://github.com/godotengine/godot/blob/876b290332ec6f2e6d173d08162a02aa7e6ca46d/editor/shader/visual_shader_editor_plugin.cpp#L4212).

Godot UndoRedo는 등록된 연산을 순서대로 실행한다. 호출 오류는 기록하지만 이미 실행한
연산 전체를 자동으로 롤백하는 준비/검증 트랜잭션은 아니다.
근거: [commit_action와 연산 실행](https://github.com/godotengine/godot/blob/876b290332ec6f2e6d173d08162a02aa7e6ca46d/core/object/undo_redo.cpp#L311).

**우리 결정:** UI는 재연결 시작 때 기존 연결을 삭제하지 않는다. 놓을 때 원래 연결 ID와
새 양 끝을 한 명령으로 보낸다. TypeScript 코어는 문서 사본에 교체를 준비하고 전체 참조를 검증한 뒤
문서·이력·리비전을 확정한다. 실패하면 기존 연결, Redo, dirty를 보존한다.
다중 삭제는 관련 연결과 빈 프레임 정리를 포함한 한 건이다. 복제/붙여넣기는 새 ID를
발급하고 선택 내부 연결·함께 복사한 프레임만 새 ID로 연결한다.
Godot의 [복제/붙여넣기 ID 매핑](https://github.com/godotengine/godot/blob/876b290332ec6f2e6d173d08162a02aa7e6ca46d/editor/shader/visual_shader_editor_plugin.cpp#L5481)은
참고하되 프레임 재연결 정책은 우리 계약으로 명시한다.

## 4. 검증과 파생 정보 갱신

Blender는 변경 flag와 런타임 캐시 무효화를 통해 토폴로지·소켓 사용·링크 검증 등의 갱신을
조정한다. 셰이더/지오메트리의 평가 의미와 편집기의 배치를 같은 종류의 갱신으로
취급하지 않도록 분리할 근거가 된다.
근거: [변경 표시](https://github.com/blender/blender/blob/8cb6b388974a817afedf1317ce26f0c75aa5f181/source/blender/blenkernel/intern/node_tree_update.cc#L50),
[트리 갱신](https://github.com/blender/blender/blob/8cb6b388974a817afedf1317ce26f0c75aa5f181/source/blender/blenkernel/intern/node_tree_update.cc#L495).

Godot VisualShader는 연결 전 포트 범위·형식·중복·순환 등을 검사한다. 이 순환 제한은
셰이더 모델의 결정이며 모든 스킬 그래프에 그대로 적용할 근거는 아니다.
근거: [can_connect_nodes](https://github.com/godotengine/godot/blob/876b290332ec6f2e6d173d08162a02aa7e6ca46d/scene/resources/visual_shader.cpp#L1169).

**우리 결정:** 없는 ID/포트, 형식 불일치, 잘못된 위치는 확정을 거부한다. 미연결 입력과
순환/순환 의존은 저장 가능한 작성 중 진단이다. 실행 의미는 아직 정의하지 않는다.
초기에는 확정 시 전체 구조 검증을 유지하되 진단은 문서 변경 때만 산출한다.
포인터 이동에 편집 확정·전체 직렬화를 넣지 않는다. 위치만 바꿀 때 진단 재계산을
생략하는 최적화는 대표 문서 측정 결과에 따라 도입한다.

## 5. 저장과 불러오기

Blender는 노드·소켓·링크를 기록하고 읽을 때 런타임 구조와 ID 조회 캐시를 재구축한다.
오래된 파일의 ID 보정 등은 Blender의 호환성 정책이며 우리 새 형식의 암묵적 복구 정책으로
가져오지 않는다.
근거: [쓰기](https://github.com/blender/blender/blob/8cb6b388974a817afedf1317ce26f0c75aa5f181/source/blender/blenkernel/intern/node.cc#L1464),
[읽기](https://github.com/blender/blender/blob/8cb6b388974a817afedf1317ce26f0c75aa5f181/source/blender/blenkernel/intern/node.cc#L2146).

Godot VisualShader는 Resource 속성 목록에 노드·위치·연결을 노출한다. GraphEdit 화면
객체를 저장 문서로 삼지 않는다.
근거: [저장 속성](https://github.com/godotengine/godot/blob/876b290332ec6f2e6d173d08162a02aa7e6ca46d/scene/resources/visual_shader.cpp#L2040).

**우리 결정:** 버전 있는 JSON에 문서만 기록한다. 파싱과 참조 검증을 끝낸 뒤 세션을
교체하고 이전 비동기 요청을 무효화한다. 저장 성공 확인에는 실제 내보낸 문서의 토큰을
사용한다. 저장 중 추가 편집은 dirty로 남긴다. 파일 대화상자·디스크 쓰기는 전송/파일 경계의
책임이며, 네이티브 파일 저장의 원자성은 이 소스 분석으로 검증된 것이 아니다.

## 초기 초안 재검토와 현재 적용

| 항목                        | 결정                             | 확인할 결과                                                    |
| --------------------------- | -------------------------------- | -------------------------------------------------------------- |
| 위치를 Document에 저장      | 유지                             | 이동→Undo→Redo→저장/열기의 좌표 일치                           |
| 문서 세션과 UI 투영         | 유지                             | UI가 실제 로컬 코어를 호출하고 여러 문서 세션이 섞이지 않음    |
| 사본 검증 후 패치 이력 확정 | 유지                             | 다중 변경 중 뒤 항목 실패 시 문서·이력·리비전 불변             |
| 재연결                      | 한 요청으로 적용 완료            | 취소하면 원래 연결 보존, 교체는 한 번의 Undo                   |
| 복제·프레임                 | 계약 명시                        | 내부 참조 재매핑, 외부 연결 제외, 한 번의 Undo                 |
| 저장 확인                   | 보강 완료                        | 저장 결과 순서 역전/중복 확인이 saved 기준을 되돌리지 않음     |
| 진단 계산                   | 문서 변경 시 계산·캐시 반영 완료 | 문서 변경 없는 조회의 불필요한 재계산 제거 또는 측정 근거 기록 |
| 포트/종류 정의              | TypeScript 코어 카탈로그로 이관  | UI가 별도 편집 규칙·기본값을 소유하지 않음                     |
| 전체 문서 복제·왕복 응답    | 초기 구현으로 검증 후 측정       | 노드 수별 명령 시간/응답 크기와 드래그 프레임을 별도 보고      |

0단계의 지정 경로 분석을 완료했고, 이후 위 계약을 코어와 UI에 반영했다.
현재는 사용자 승인에 따라 편집 코어를 TypeScript로 이관하고 노드 전용 Rust 브리지를 제거했다.
결과와 Windows 네이티브 미검증 범위는
[검증 기록](node-editor-validation.md)을 따른다.
함수형 그룹, 중첩 서브그래프, 동적 포트, 셰이더 실행기, Blender 전체 Undo 내부,
양 프로젝트의 네이티브 렌더링 성능은 이번 분석에서 검증하지 않았다.
