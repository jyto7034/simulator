# U16-3 실행 계획 — GOAL-CDMG-01

2026-10-15 현재 사용자 요청으로 U16-3 구현·자체검증·복원 가능한 target 고정 후 비참여 서브에이전트 독립감사를 수행한다. 사용자의 직접 예외는 지시서의 서브에이전트 금지/감사요청대기보다 우선한다. 구현은 현재 실행자가 단독 수행하며 감사자는 설계/교정에 참여시키지 않는다. U16-4/전체U16/U21/Unity/개별최종보스는 제외한다.

P2G00–60의21수용/10차원/5IVA, RULES.ko.md의 위임 판본/튜닝을 따른다. 초기baseline은 U16-2 current audited sources와고유2196회귀이고 실제수정전파일을BASELINE_SOURCE_001에보관한다. 기존dirty와모든과거출력/동결은보존한다.

## 구현 순서와 소유

1. 순수게임규칙: BJ·Dice·Flip7의현재판만보존. RNG상태/미래덱/딜러hole은공개view에서제거한다. 각게임의정지/버스트/포기/대상선택과public-only NPC를독립수작업사례로검증한다.
2. validated layout DealerPolicy·VN참조·상품/원금/포인트, optional RewardSession dealer와RunProgression 설명이력. 사본은미장착검사뒤session escrow로이동하고원래UUID/variant로반환하여잠금우회를막는다. 자산/포인트/재고는기존draft/transaction와grant에서처리한다.
3. 정상 typed DealerAct(revision,action), owner/route/capability/DTO·현재자격/금액예고와실제VN choice-hook을연결한다. 일반Claim/Exit/CompleteNode로담보정산을우회하지못한다.
4. 테스트/외부pack/정상host:미완료조우전체입장전복원,완료보존/SavePoint/실제IO실패·재시도. source/fixture/config는실행중고정. 대표/확장비용과기존한도를검토하고후속감사target을한번고정한다.

## 시나리오와 검증 준비

초기정상작성pack은 U16-2대표12유물효과를사용하고Event에서정상자산을준비,SavePoint→Gambling예약경로를사용한다. 제안VN은정상선택명령으로닫고첫게임선택/갬빗VN/Ready/시작을거친다. 각주요행동후단계·잔액·사본/담보·revision·다음자격을assert하며예상밖VN/단계를건너뛰지않는다. 유한pilot로게임진입·정산·나가기·재입장을확인한후큰저장/비용시나리오를실행한다.

- 순수규칙의수작업BJ Ace/natural/push/double/딜러17, Dice8조합/kicker/3회/hold, Flip7숫자중복/SecondChance/Freeze/강제3장의지연action/대상선택/7즉시종료/보정순서/공동1위/버스트제외.
- 정책경로는기본포인트1회·첫/반복설명·게임전취소·갬빗고정과음수/0계속거절·게임변경/다수판을확인한다. 난수결과를맞추기위해seed를바꾸거나플레이도중상태를조작하지않는다.
- 담보는미장착/다른사본/Corrupt를초기에정상명령으로준비한다. 다중선택/취소/지급후양수/중복거절/원금일부회수·전부포기/구매/0점쇼핑·나가기에서정확한자산을대조한다.
- 실제host미완료Continue는중간게임·대출·상환·구매후각기입장전복원해야한다. IO는실제발행되는완료/SavePoint경계에서자격을만족시킨뒤주입한다. 중간publication이없음을IO실패로오판하지않는다.
- 회귀는명령vocabulary·기존3활동·기존일반보상·map/loader·clone/저장·server consumer를영향기반선정. 최종통합1회뒤수정영향만재검증한다. 유효기존native/server/socket결과는비영향근거와구분한다.
- 비용은정상/확장상품·담보와반복판을고정입력/기본profile에서측정하고time/allocatorpeak/RSS/nativework를구분한다. 끝난뒤allocatorlive회수를확인하며단순반복도큰history를남기지않는다.

빌드기존근거는dev약2분/release약6분,집중기능수십초·전체lib약8분이다. 첫60초로그/진행확인후CPU/메모리와소유세션을진단한다. 출력없음만으로중단하지않고입력수정전소유실행을종료/정리한다. 원시실패를보존하며제품/fixture/도구/환경을구분한다.

## 완료 경계

모든21수용/10차원/5IVA와실제공개경로·비용·복원증거가확보되어야GEX10을종료한다. 구현중감사자를소환하지않는다. 마지막target고정뒤서브에이전트에게원문계약·코드·증거·현재성입출구를인수시켜GEX20을수행한다. 발견사항을감사중직접고치지않고별도교정pass와필요한재감사를분리한다.
