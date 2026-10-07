# Simulator 콘텐츠 제작 기반 — 의존성 검토·개정 배포본

**2.2-review.1 · 2026-09-09 · 한국어 연구 보고서 / 저장소 미적용**

**권고: 기존 M0 성과와 GOAL-CAMEC-01을 유지하고, 후속 구현 순서를 최소 계약과 유효한 중간 상태 중심으로 재분할한다.** 최소 Goal 영향 판정은 **A**다. 이는 정식 current/stale 판정, GEX 자격·실행 승인 또는 게임 테스트 통과를 뜻하지 않는다.

## 읽는 순서

| 문서 | 확인할 내용 |
| --- | --- |
| [핵심 보고서](report/REVIEW.ko.md) | 권장 순서, 핵심 동시 전환, 21개 finding, 두 최종 질문의 답 |
| [최소 Goal 영향 보고](report/GOAL_CAMEC_IMPACT.ko.md) | A 권고, exact artifact, 기존 성과 재사용, 수정이 필요할 때의 owner |
| [M/CA 전체 대응표](report/M_CA_MAPPING.ko.md) | M0–M8·CA00–CA23의 유지·분할·재배치·미검토 구별 |
| [개정 로드맵](roadmap_v2.2-review/ROADMAP.ko.md) | R00 인수와 R01–R24의 하나의 기본 순서 |
| [의존성 판정](roadmap_v2.2-review/DEPENDENCIES.ko.md) | 기존 49개 관계의 개별 disposition, 새 76개 관계, 16개 동시 전환 |
| [개정 파일 색인](roadmap_v2.2-review/00_README.md) | GOALS.json, 25개 카드, 8개 아키텍처 문서, 계약·정책·검증·인수 안내 |
| [근거 색인](evidence/SOURCES.ko.md) | 현재 본문을 읽은 97개 파일의 경로·범위·내용 식별 한계 |
| [조사 한계](report/LIMITATIONS.ko.md) | 전체 COV 재검증·실행 시험·정밀 기하·현재성의 미완료 범위 |
| [문서 검사 결과](evidence/DOCUMENT_VALIDATION.ko.md) | 실제 로컬 정합성 검사와 게임·정식 검증 미실행의 구별 |
| [적용 안내](APPLY.ko.md) | 별도 경로에 새 파일로만 배치하는 방법 |
| [현황 추가 기록안](ROADMAP_STATUS_ADDENDUM.ko.md) | 기존 기록 0010·NEXT-0009를 보존하는 미적용 제안 |
| [변경 요약](CHANGELOG.ko.md) | 기존 v2.1과 비교한 변경 이유·보존 범위 |

## 확인한 것과 하지 않은 것

97개는 **기재된 본문 구간을 읽은 파일 수**다. 모든 파일의 완독, 전체 dirty snapshot, 전체 354 COV/127 BOSS-COV의 새 행별 표현력 검증을 뜻하지 않는다. 읽은 원본에서 관찰한 사실과 개정 설계 판단은 보고서에서 구별했다.

이번에는 **게임 구현·RON 이식·게임 빌드/시험·독립 감사·저장소 쓰기·NEXT-0009 실행을 하지 않았다.** 실제 수행한 자동 검사는 이 로컬 배포본의 문서·JSON·참조·후보 그래프 검사다. 검사기의 PASS를 P2G/GEX 또는 게임 완료로 사용하지 않는다.

## 파일 형식과 재현

`roadmap_v2.2-review/GOALS.json`은 연구용 버전 스키마 `content-authoring-roadmap-review/1`을 쓰는 **별도 개정 로드맵**이다. 정식 `GS-CAMEC-P2G50-001`이나 기존 배열형 로드맵 파일에 덮어쓰는 입력이 아니다. 원래 파일을 요구하는 도구와의 호환성을 인증하지 않았다.

`tools/validate_package.py`는 Python 표준 라이브러리만 사용하며, 이 배포 폴더 안의 파일을 읽고 `evidence/DOCUMENT_VALIDATION.*`만 쓴다. 저장소·네트워크·게임 명령을 호출하지 않는다.

```bash
python tools/validate_package.py
```

배포본의 `SHA256SUMS.json`은 **배포 파일**의 해시 목록이다. 연결된 simulator의 dirty source 해시가 아니다. 파일을 편집하거나 검사 결과를 다시 생성하면 그 배포 해시도 달라질 수 있다.

`input_package/`는 제공된 인수인계 ZIP의 문서 사본이다. 프로젝트 소스가 들어 있지 않으며, 그 안의 과거 sandbox 경로·검사 결과는 이번 결과로 승격하지 않는다. `tools/`의 다른 파일은 이 조사 문서를 작성한 로컬 생성 스크립트이며 필수 실행 절차가 아니다.
