# U16-4 상태와 수명

| 상태 | 소유 | 수명·복원 |
| --- | --- | --- |
| 작성 조우/5라운드 wave IDs/보상 | validated FacilityLayout.stress_combat | 입장 전 첫 보상/배율 조회, ActiveBattleSession이 고정 사본 소유 |
| 현재·완료 라운드/선택 대기 | BattleCore WaveRuntimeState | 같은 execution과 UUID. 현재까지 wave만 활성화. timestamp 정리 후 완료, 선택 중 모든 elapsed/next-bucket 시간 진행 중지 |
| HP/배치/자원/쿨다운/전투당 기록/보호 대상 | 기존 BattleCore/ActiveBattleSession | 라운드 전환에 새 core·초기화·회복 없음. 공통 목표·효과 대상 자격 재사용 |
| 미수령 보상 | 고정 policy + 완료 라운드로 유도 | 실제 inventory/XP를 미리 변경하지 않음. query만 무억제 배율 적용, 성공 settle은 원량→공통 배율1회 |
| 성공·패배 XP/직원 손실 | 기존 common combat result/grant | 전체 전투1회. 성공 생존 XP는 자격 배율, 패배는 기본량. 개별 라운드 settle 없음 |
| 수동 후퇴 | entry snapshot + common node completion | 직원/소비/장비/유물 복원 뒤 비용/receipt를 유지하며 노드 소비. 같은 지원 application candidate에서 원자 확정, XP 없음 |
| 같은 프로세스 재접속 | 공개 read-only reconnect/query | 현재 라운드·미수령·고정 보상·전체 battle 유지 |
| fresh Continue/SavePoint | 기존 paired persistence/checkpoint | 미완료는 입장전 전체 복원. 성공/패배/후퇴는 완료된 결과와 소비를 함께 보존. 새 저장소/새 codec version 없음 |
| round 선택 명령 | StressCombatAct + battle authority | battle UUID/완료 라운드/선택 단계 확인. 계속 후 같은 round 재전송 거절, 종료 뒤 재정산 거절 |

StressCombat 조우 class를 ordinary selector에서 제외해 일반 전투가 라운드 gate 없이 이 콘텐츠를 소비하지 않게 한다. 공개 snapshot은 일반 전투의 남은 후퇴2회 안내를 이 활동에 표시하지 않는다. optional DTO/정책은 None일 때 기존 출력에서 생략한다.
