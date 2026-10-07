# R06 WP02 F02/F03 교정 실행 범위

기준은 F01-R01 correction target `ba25a8fbedfeb4b43c13d8f3bce88340e3ffafc4eb6dba2424169028bad7be98`이다. 진입 시 read-only currentness verifier가 328개 source의 일치를 확인했다. 수정 전 dirty/untracked를 포함한 현재 333개 입력 파일의 실제 bytes와 hash는 `baseline/entry_001`에 보존했다.

이번 교정은 F02(action의 actor/provider 결속)와 F03(Live resource 읽기의 active unit 가용성)만 다룬다. F02는 `runtime_facts` 소유자의 immutable action snapshot을 검증 기준으로 삼고, F03은 ValueExpr frame 읽기 경계에서 unit 상태를 확인한다. 기존 계산·예산·누적 청구·취소 계약은 그대로 둔다.

의미 있는 책임이 닫히고 빌드·관련 검증이 가능한 지점에서 별도 correction target을 동결하고 중단한다. CAP-01/CAP-02, 순차 effects/result-prefix, WP03, publication은 이번 타깃에서 제외한다. 자체 검증은 재검토 PASS 또는 WP/Phase/Goal 완료 판정이 아니다.
