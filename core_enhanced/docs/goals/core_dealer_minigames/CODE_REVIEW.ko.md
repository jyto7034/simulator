# U16-3 착수 시 실제 코드 경계

2026-10-15 U16-2 independent001 Complete/High의 그대로인 소스를 인수했다. 아래는 정적 검토이며 기능 PASS가 아니다.

| 경계 | 현재 사실 | 필요한 변경·검증 |
| --- | --- | --- |
| FLOW | facility_rooms가3기본유형만지원, map_content RewardSession진입. VN은authored scene/choice와Core차단을제공. RunProgression은checkpoint·방이동에동일소유 | Gambling 전용session과실제VN시작/choice-hook. 원정설명이력은run,기본포인트/갬빗은session. 기존일반exit로정산우회금지 |
| GAME | determinism seed helpers와일반정상명령포트존재; 세미니게임없음 | 별도순수게임규칙모듈,공개정보NPC,현재판유한상태/고정RNG. 공개view에서dealerhole/난수/덱미노출. 규칙표/수작업oracle |
| LOAN | RelicInventory사본UUID/variant·단일slot, remove·commongrant·preflight존재 | 미장착원래사본을session escrow로옮겨잠금. 내부원래사본반환API와금액계산을기존transaction에서처리. 무관자산/UUID보존 |
| SHOP | EconomyGrantState가7roots capture/grant/commit; reward에서기본grant/노드완료재사용가능 | authored고정상품과조우재고/포인트,revision거절. 지급은commonGrant. 나가기만node완료,중간구매는미완료복원 |
| STORE | RewardSession optional relic_activity, clone shape/active relic extent가저장비용을소유. 기존미완료입장전/완료보존·실제hostpairedpublish존재 | optional boxed dealer state와비용/preflight,RunProgression설명필드. loadercrossref/명령metadata/capability/DTO/VN/host를동시연결. 정상RON/실제IO/freshContinue/SavePoint/공유회귀/정상확장비용 |

일반/기본유물reward구현을유지한다. session입장전기준과grant/완료owner를분리하지않는다. 실제지원작성오류·명령오용·IO는범위안이며임의상태변조방어는추가하지않는다. 정책판본·초기수치는RULES.ko.md에서P2G10이결정하고구현편의로변경하지않는다. 사용자요청의비참여서브에이전트는구현종료/target고정후에만소환한다.
