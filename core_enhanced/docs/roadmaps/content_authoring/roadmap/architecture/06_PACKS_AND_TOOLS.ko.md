# A06 — 외부 팩·데이터 세대·작성 도구 포트

**권장 설계**. 외부 `live_data`·다중 팩·새 런 적용은 승인된 방향이고, 아래 정확한 manifest 필드·폴더 구조·fingerprint 형식은 구현 AI가 코드와 대조할 기술 제안이다.

## 1. source와 실행 입력 분리

```text
game_resources/data/       공식 authoring 원본
      ↓ packaging (빌드에 문자열 삽입하는 것이 아님)
live_data/
  pack_selection.ron       활성 pack·명시적 충돌 선택
  official/manifest.ron
  official/definitions/...
  community_a/manifest.ron
  community_a/definitions/...
```

공식·community에 같은 manifest·compiler 규칙을 적용한다. 공식 namespace를 수정 불가능하게 만들지 않는다. 사용자가 편집하는 live_data는 runtime 입력이며 source tree와 서로 자동 덮어쓰기하지 않는다. 재배포가 기존 사용자의 커스텀 파일을 삭제하는 작업은 이 로드맵에서 승인하지 않는다.

파일을 효과 발생마다 다시 읽지 않는다. 로딩 서비스가 요청한 시점의 source 묶음을 확보하고, 검증 성공한 정의를 메모리에 게시한다. 파일 감시기는 선택적인 reload 요청 생산자일 뿐 실행 정의를 몰래 바꾸는 권한이 없다.

## 2. manifest와 합성

권장 manifest 정보는 pack ID/version, 요구 engine authoring version·capabilities, dependencies, definition file 목록, explicit add/replace/remove 선언이다. namespace가 다른 신규 정의는 충돌하지 않는다. 기존 정의 대체는 target definition을 명시한다.

팩 의존 순서가 내용 교체 우선순위라는 가정은 금지한다. replacement chain의 명시적 선후 또는 selection 파일의 winner가 필요하다. 모호한 다중 writer는 관련 팩과 target을 나열하는 오류다. full-definition Replace를 기본으로 하고 좁은 typed patch는 scope/type/postcondition 검증이 가능한 범위만 제공한다.

변경된 최종 catalog의 참조를 검사한다. 지운 definition을 참조하는 reward·boss·skill이 남으면 거부한다. 교체 전 loser의 더 이상 사용하지 않는 참조를 이유로 유효한 최종 세트를 막지는 않는다. dependencies/import 자체는 manifest 단계에서 따로 검사한다.

## 3. 읽기 일관성·비정상 데이터

manifest/file 목록을 고정해 원문 bytes와 source path·hash를 확보한다. 파일 누락·동시 변경·한계 초과에서는 후보를 게시하지 않는다. 느슨한 폴더 입력은 재확인·bounded retry 후 실패하는 경로로 두고, 원자적 배포가 필요하면 immutable pack directory + manifest 교체를 사용한다. OS 파일 읽기 여러 번만으로 완전한 파일 시스템 snapshot을 보장한다고 주장하지 않는다.

경로 정규화, 루트 밖 path traversal·절대 경로·symlink escape 정책, case 충돌, 중복 ID, 파일 수/크기·parser 깊이 한계를 검증한다. 자유로운 gameplay customization은 임의 OS 경로 읽기나 무제한 메모리 사용 권한이 아니다.

## 4. 게시와 pin

`load_candidate(source_set, engine_contract)`는 raw validation→팩 합성→최종 참조→compile→generated setup을 거쳐 `ValidatedGeneration`을 반환한다. publish가 application factory에서 가능한 유일한 경로가 되게 한다. 새 런이 이 객체의 강한 참조를 잡으면 기존 런의 참조는 변경하지 않는다.

새 데이터 B 로딩이 실패하면 새 런 요청은 오류이고, 기존 런 A는 계속 A를 사용한다. 오류를 숨기고 B 요청에 A를 적용하지 않는다. 시험 reload는 B 검증과 시험 session 생성이 모두 성공한 뒤 기존 시험 session을 교체한다. 기존 session을 먼저 파괴하지 않는다.

미참조 generation은 회수할 수 있다. 캐시·잔존 참조를 측정한다. 같은 schema의 이전 메모리 세대 유지와 구형 parser 지원은 다른 개념이다.

## 5. fingerprint·저장

권장 fingerprint는 정렬된 최종 definition/정책 의미 표현, resolved pack selection과 contract version에 근거한다. raw source hash도 별도 receipt에 남긴다. 주석/공백과 source bytes의 변경을 semantic change와 어떻게 구분할지는 CA02 상세 설계에서 명시한다. 내용이 다른데 ID만 같다고 저장을 허용해서는 안 된다.

저장은 최신 SaveSchemaVersion, execution/authoring contract, 실제 resolved content fingerprint를 기록한다. 불일치는 명시적 거부다. 구형 reader나 자동 migration을 만들지 않는다. 최신 팩으로 바꾼 뒤 옛 저장을 이어하려면 사용자가 그 저장에 맞는 동일 콘텐츠 구성을 복원하거나 새 런을 시작해야 한다. 도구가 파일을 임의 복구·삭제하지 않는다.

## 6. 작성 API

권장 기능 포트:

| 포트 | 입력 | 출력 |
| --- | --- | --- |
| Describe | engine capabilities + 선택 consumer | 타입·reference·property·result·lifetime·조건·현재 지원 여부 |
| Validate | source pack selection | structured diagnostics + validated candidate receipt |
| Explain | generation + loadout/boss graph context | 최종 효과 계획, 생성/변형 출처, selector와 상태 전이의 설명 |
| Simulate | generation + scenario + initial state + seed + inputs/clock | 결과·semantic trace·실행량·오류 receipt |

이 명칭은 현재 CLI의 존재를 뜻하지 않는다. CLI는 위 library API의 얇은 adapter다. 조합기 UI는 나중에 이 API를 사용할 수 있고 별도 validator를 복제하지 않는다.

Validation에는 구문·구조·최종 조합·실행 준비 단계가 있다. runtime HP 같은 입력이 필요한 조건은 “정적 유효/실행 시 재검사”로 표시하고, 모든 미래 결과를 검증기가 증명했다고 표현하지 않는다. 미구현 기능은 warning 뒤 무시하는 것이 아니라 publication error다.

## 7. 진단·trace

Diagnostic은 code/severity/phase, pack/file/source span, definition/effect/task, expected/actual type, 관련 source, 수정 가능한 선택지·지원 capability를 제공한다. Read-only Explain/Preview는 state·RNG·revision을 바꾸지 않는다.

Trace는 log 순서와 별개 semantic occurrence ID를 갖고 source map으로 작성 위치를 찾게 한다. actor/boss phase·task, action milestone, effect execution, resource before/after, composition decision과 차단된 chain 경로를 조합해 설명한다. Trace sink를 켜고 끈다고 RNG나 전투 결과가 달라지지 않아야 한다.
