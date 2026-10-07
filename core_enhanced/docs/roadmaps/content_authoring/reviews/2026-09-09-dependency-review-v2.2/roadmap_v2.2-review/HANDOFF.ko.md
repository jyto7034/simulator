# 개정안 인수 안내

**2.2-review.1 · 2026-09-09 · 조사·개정 권고 / 저장소 미적용 / 게임 구현·Goal 실행 미승인**

이 패키지는 simulator dirty 작업 트리의 **조사·개정 권고**다. source repository가 들어 있지 않으며 P2G/GEX 정식 원본을 복제해 새로운 authority를 만들지 않는다. 실제 저장소 접근과 현재 dirty 내용을 별도로 유지해야 한다.

먼저 [조사 보고서](../report/REVIEW.ko.md)와 [최소 Goal 영향](../report/GOAL_CAMEC_IMPACT.ko.md)를 읽고, [권장 순서](ROADMAP.ko.md)·[관계/동시 전환](DEPENDENCIES.ko.md)·[전체 대응](../report/M_CA_MAPPING.ko.md)·[한계](../report/LIMITATIONS.ko.md)를 함께 인수한다.

현재 관찰된 formal basis는 WR-CAMEC-P2G20-001 / FPF-CAMEC-P2G30-001 / PP-CAMEC-P2G40-002 / GS-CAMEC-P2G50-001 / GOAL-CAMEC-01 / HOF-CAMEC-P2G60-001이다. ROADMAP_STATUS는 v1.9 / 기록 0010 / NEXT-0009였고 구현 권한은 없었다. 새 R 번호는 기존 Goal ID를 개명한 것이 아니다.

후속 요청이 **검토**이면 조사/문서 범위만 이어간다. 별도 **구현 요청**이 있을 때만 그 exact target과 기존 GEX 인수 규칙에 따라 현재성을 확인한다. 이 파일 또는 NEXT-0009가 권한을 만들지는 않는다. 다른 후속 R은 아직 정식 Plan/Goal이 아니다. 필요할 때 담당 P2G owner가 이 근거를 사용해 형식화한다.

sourceledger의 전체 파일 해시는 대부분 비어 있고 일부 앵커/Git 짧은 ID만 있다. 따라서 새 작업의 source 변화는 영향 범위를 다시 읽어 확인한다. L03/L04/L05의 미검토/정책대기를 완료로 바꾸지 않는다. 이전 sandbox 경로가 다른 세션에 그대로 있다고 가정하지 않고 제공된 ZIP/실제 파일을 인수한다.

적용은 [APPLY](../APPLY.ko.md)와 [추가 기록안](../ROADMAP_STATUS_ADDENDUM.ko.md)을 따른다. 이번 배포본은 저장소에 적용하지 않았다.
