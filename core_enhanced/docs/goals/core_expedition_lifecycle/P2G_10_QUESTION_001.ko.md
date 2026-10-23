# U12 기억 보존 범위 확인 — OPQ-CEXL-001

작성: 2026-10-12T08:57:31.550560+09:00

U11은 Complete/High/open0으로 종료됐고 U12 P2G-00은 완료됐다. U12는 P2G-10 정책 확인 중이며 현재 checkpoint는 WR-CEXL-002다. [불변 기록](P2G_10_OPEN_001.json)의 exact ref가 authoritative하다. P2G-10 complete=false이고 P2G-20 진입 근거가 아니다.

## 확인할 의미

U12에서 실제 기억을 만드는 게임 사건은 추가하지 않고, 신뢰와 독립된 기억 보존 기반을 구현·검증하는 범위로 진행할지 확인한다. 권고는 **보존 기반만 구현·검증**이다. 합침·기간·최근8·저장/복원·후퇴/checkpoint/사망 기록·새 인원 미계승은 비어 있지 않은 검증용 기록으로 확인하되, 정상 플레이에서 새 기억이 생긴다고 주장하지 않는다. 다른 선택은 실제 생성 사건을 먼저 정책으로 확정한 뒤 구현하는 것이다. 사건 종류/생성 시점은 임의로 채우지 않는다.

현재 `PsychologyEventKind`는 비어 있어 유효한 비어 있지 않은 memory/recent event를 표현할 수 없다. 빈 목록 왕복만으로 합침·기간·최근8 보존을 입증할 수 없다. 과거 July18 완료 문서는 `RestedAtSupportRest`를 서술하지만 현재 results-and-save는 SavePoint/Gate가 기억/최근 사건을 생산하지 않는다고 정한다. 과거 생산자를 자동 복구하거나 기억 전체를 삭제하지 않는다.

로드맵 U12는 “기억/최근 반응을 보존하기 어려운 실제 구조 충돌이 발견되면 해당 변경만 사용자에게 질문한다”고 명시한다. invoked P2G §8.6에 따라 질문 후 주 작업은 답변을 기다린다. 신뢰 제거·튜토리얼 포팅 등 이미 확정된 정책의 재승인 요청이 아니다.

## 보존과 재개

[질문 원문/선택 영향/근거/미결 요소와 immutable successor](P2G_10_OPEN_001.json), [읽은 원문 보존](P2G_10_QUESTION_SOURCE_001.tar.gz). 기존 intake와 WR-CEXL-001은 수정하지 않았다. 제품·시험 파일 변경이나 실행은 없다. 기존 독립 읽기 전용 조사는 배정 범위에서 종료됐으며 새 작업은 배정하지 않는다.

실제 답변 후 raw input successor intake, Decision/decided Element와 successor Working Revision을 P2G-10에서 고정하고, 나머지 U12 요소 검토→P2G-20 이후를 기존 연속 권한으로 재개한다. 답변 전 후속 조사/구현/검사를 진행하지 않는다.
