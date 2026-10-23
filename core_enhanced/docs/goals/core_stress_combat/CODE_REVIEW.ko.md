# 정적 경계 검토

StressCombat enum은 있지만 resolve_map_reward가 미지원으로 거절한다. 기존 live DefenseRoute는 보호 대상/대상 효과 자격/연속 Wave와 하나의 BattleCore를 제공한다. wave_runtime의 활성 조건과 sim timestamp 경계에 라운드 gate를 연결하면 새 전투/자동회복 없이 진행할 수 있다. ActiveBattleSession은 application candidate clone에 포함되며 fresh Continue는 active 전투를 저장하지 않고 기존 node-entry checkpoint를 복구한다. combat 결과의 공통 settle/XP/grant/노드 소비를 재사용하고 전투 중 후퇴만 snapshot 복원 뒤 노드 소비를 결합한다. 정상 선택 시간과 명령 자격은 battle capability가 소유한다. loader 검증은 live_ron의 layout cross-reference를 사용한다. 실행 증거는 아직 없다.
