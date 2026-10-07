# 2026-09-27 로드맵 cleanup 보존 기록

현재 개발은 [단일 현황](../../ROADMAP_STATUS.md)에서 재개한다. 이 폴더는 정리 근거·원문 복원용이며 별도 작업 진입점이 아니다.
사용자가 중복 진입점을 정리하고 불필요한 파일을 삭제 또는 아카이빙하도록 요청했다. 제품 구현·새 기능 계획 착수는 범위 밖이다.

## 처리 결과

- 개발 주작업은 R06, 다음 개발 작업은 잔여 D04 설계·구현 지시서 작성으로 바로잡았다.
- 현황표를 R00–R24로 통일하고 기존 기능/검증 의존성과 조건부 분기를 유지했다. CA/M는 원래 요구 대응표로 연결한다.
- 전체 순서와 후속 편입 위치는 단일 현황에서 안내한다. 후속 모드 계획의 MS 번호는 그 작업 안에서만 사용한다.
- 과거 현황의 최근 기록 12건을 [네 번째 이력 묶음](../../history/2026-09-04.md)에 옮겼다. 상대 링크는 같은 대상을 가리키도록 조정했다.
- 운영 규칙의 과거 진행 서술·절차 복제를 줄이고 현재 P2G/GEX 원문으로 연결했다. 원래 운영 문서는 아래 archive에 보존한다.
- 기존 docs 인덱스에 개발 현황 링크를 추가하고 단일 R 요청문도 같은 진행표를 가리키도록 맞췄다.

## 과거 배포 자료

다음 6개는 활성 경로에서 제거했고 [BEFORE.tar.gz](BEFORE.tar.gz)에 원래 저장소 상대 경로와 bytes 그대로 보존했다.
영구 삭제한 원본은 없다. 경로 표는 `core_enhanced/docs/roadmaps/content_authoring/`를 기준으로 한다.

| 원래 경로 | 정리 이유 |
| --- | --- |
| 00_PACKAGE_README.md | 9월9일 배포 안내와 NEXT-0001 설명. 현재 현황 입구로 오인할 수 있는 중복 안내 |
| CHANGES.ko.md | 과거 배포 수정 내역. 현재 변경 이력과 분리 |
| DOCUMENT_CHECKS.json | 과거 문서 묶음 검사. 현재 프로젝트 검증처럼 보이지 않게 분리 |
| PACKAGE_MANIFEST.json | 당시 배포본 파일 hash. 현재 변경된 현황·운영 파일의 manifest가 아님 |
| changes/final_revision.diff | 당시 배포 patch. 현재 작업에 적용할 patch가 아님 |
| guides/AGENTS_P2G_ROUTING.example.md | 적용되지 않은 예전 라우팅 예시. 실제 지침으로 사용하지 않음 |

과거 baseline JSON 등에 기록된 이 경로들은 당시 위치다. 기존 baseline·manifest를 새 hash로 덮어쓰지 않았다.
당시 파일을 비교할 때는 이 archive에서 해당 member를 읽는다. 현재 tree에 파일이 그대로 있다고 주장하지 않는다.
관련 history의 Markdown 링크는 이 보존 위치로 연결했다.

아카이브에서 읽는 예시(저장소 루트):

```sh
tar -xOf core_enhanced/docs/roadmaps/content_authoring/archive/cleanup_20260927/BEFORE.tar.gz core_enhanced/docs/roadmaps/content_authoring/CHANGES.ko.md
```

BEFORE에는 위 6개뿐 아니라 편집 전 현황·운영·요청문·관련 계획/검증과 기존 dirty 상태의 docs 인덱스 등 총 18개 파일을 담았다.
복원할 때는 별도 임시 디렉터리에 추출해 비교한다. 현재 작업 트리에 전체 덮어쓰기 하지 않는다.
상대 링크의 원래 문맥도 archive member의 저장소 상대 경로다.

## 원위치 보존한 자료

| 위치 | 유지 이유 |
| --- | --- |
| ../../roadmap/ | 기존 CA 요구·아키텍처·계약·수용 기준·의존성. 동결 계획과 후속 작업이 실제 참조하는 근거 |
| ../../reviews/ | R 의존성 연구와 조건부 재검증, 원래 연구의 manifest. 현재 순서의 근거이며 새 현황 원본은 아님 |
| ../../records/ 및 기존 history/ | 인수·완료·교정 기록과 원문 증거. 현재 인수 상태를 소급 수정하지 않음 |
| guides의 나머지 두 문서 | 이전 인수/문서가 원래 경로로 참조하는 사용 안내·P2G 사본. 일상 진입에서는 저장소 P2G 원문 사용 |
| 기존 Goal의 계획·target·review·manifest | 별도 감사/동결 자료. 이번 cleanup에서 수정하지 않음 |

현재 사용하지 않는 배포 보조물과 참조가 필요한 역사 자료를 구분했다. 파일이 오래됐다는 이유만으로 요구사항·증거를 버리지 않는다.
파일을 다시 만들거나 원본 경로에 symlink/중복 진입용 redirect 파일을 남기지 않았다.

## 검증

[ENTRY.json](ENTRY.json)은 변경 전 18개 문서와 원문 archive, 보호 대상 362개 파일의 hash를 기록한다.
[VALIDATION.json](VALIDATION.json)은 archive 원본 일치, 제거 대상, 문서 patch 양방향 적용,
현재 문서 링크/앵커, R 순서·선행표, 기록 이동과 보호 대상 무변경을 확인한다.
[DOCUMENT_CHANGES.patch](DOCUMENT_CHANGES.patch)는 이번 편집/아카이빙의 비교 자료이며 작업 트리에 무조건 적용하는 명령이 아니다.
[validate.py](validate.py)는 읽기 검사 후 이 폴더의 검사 결과와 patch만 갱신한다.

새 검증 결과가 없는 최초 생성 때는 cleanup 검사 → 기존 안전 계획 검사 → 모드 편입안 검사 → cleanup 검사 순서로 실행한다.
최종 검사까지 관련 문서 hash를 맞춘다. 이전 검사 파일도 BEFORE에 보존되어 있다.
제품 빌드·게임 시험·부하 측정·동결 source currentness·독립 감사는 이번에 수행하지 않는다.
