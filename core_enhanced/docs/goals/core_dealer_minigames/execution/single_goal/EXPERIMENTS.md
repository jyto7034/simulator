# U16-3 실행·교정 기록

2026-10-15 착수. P2G40 candidate의 Parallelization 문구를 validator가 요구하는 `none approved`(병렬구현없음)로 교정하고, 사용자 승인 독립 서브에이전트는 구현 후 별도 감사라고 명시했다. 최초2개실패원문과candidate를보존했다. 이는제품검증아님. P2G50/60/GEX00의실행결과·수정전baseline을인수한뒤제품구현에진입한다.

## pilot001–004

check001 exit0(33.91s) 제품컴파일; build002 exit0(1m50s) libtest빌드, 기능시험0. focused003은 순수게임10통과/공개경로6실패/ignore0; 6실패는 dealer Dialogue의 speakers누락을정상로더가거절한동일작성오류였다. 실패RON/로그를보존하고정상speaker를추가했다. 게임기대값·로더검사를완화하지않았다. 제품검토에서포인트전체판돈을i64로표현하여u32이상잔액도다룰수있게했고손익/preview는checked연산·i128natural중간계산을사용한다. FlipSecondChance의즉시버림/재셔플과forfeit종료표시를같은판계약대로구현했다.

## 005 후속 교정

기존55결과 projection목록이새DealerState를누락한fixture불일치를확인해기존55개를그대로두고56번째왕복/정확projection대조를추가했다. 지원RON의기타rarity 유물도기존normal grant/상점으로획득가능하므로, 미장착담보정책을3등급으로좁히는초기구현은제품결함이었다. 기본등급표외에는authored default_pledge_value(초기10)를공개원금으로사용하며지원입력을거절하지않는다. 이는기존'모든미장착사본'범위와CF-D06 초기평가액위임을실행하는교정이고게임판정/기존기득권은변경하지않는다. 실제정상loader→사본획득→게임→기타rarity17원금담보→원본회수대조를추가했다. 교정전빌드/실행은원문을보존하며동일·비영향범위만재사용한다.

## 최종 자체검증과 정상host

core005 all-targets는2138pass/2fail/6ignore(exit101)로 lib에서 멈췄다. projection은 기존55보존+DealerState56 fixture로 고쳤고 socket은sandboxEPERM이며 exact source/test와 실제 허용환경socket002 재사용으로 분리한다. 최종focused007은35pass/0fail/0ignore, integration009는8target72pass/0fail/0ignore. 고유2213이며 한 번의 fullgreen주장이 아니다. integration008은 존재하지않는target이름 오류로0test; 이름을실제8target으로교정했다. prepare-build001도자동bin탐색시점의존재명칭오류로0test/build실패, 실제새이름을확인한002만성공이다. diagnostic-lock001은offline yankedslab로 resolver실패; 기존고정lock을보존해동일pin사용후 --locked로성공했다.

package_diagnostic001은출력parent누락, preparation_pilot001은그미출판pack사용오류로실행실패; parent생성뒤package_diagnostic002/pilot002통과. public28commandpilot로실제도달을확인한후최종normal002/instrumented002/diagnostic002빌드·복사본을사용했다. 첫calibration001의dealer준비step18은아직Unavailable인3선택으로실패했다. 공개map에서Event2→Save5→dealer8을확인하고route교정. prepare002는그경로로step43의10M work한도도달; completion command의map을재사용하고end조회SelectedEvent로바꿔 동일거래/10battleclocks를유지한prepare003은battle3,852,152/dealer9,227,138work로통과했다. calibration002/review002/adoption002 성공. 사후한도상향/실행중입력수정없음.

host001은17freshprocess/111commands/4actualOS21및같은명령성공재시도. 초기Corrupt사본을정상API로만들고Offer/Gambit/loan/purchase미완료→Continue전체복원,부분상환·포기·묶음/단품·중복거절·완료Continue·SavePoint·설명이력·음수·전부포기를대조했다. repeatnode는3이아직Unavailable임을pilot공개map으로확인해정상Available9로고쳤다. public_games001은publicview로유한한실행계획을만든후같은계획을실제승인host에서재생하여Blackjackdouble/Flip7stand완료와새processContinue를확인했다(각21commands,4freshprocess). 총21freshprocess/153commands/IO4이며diagnostic은host승인으로세지않는다.

## 비용

activity_cost001 normal3/expanded3과expanded_collateral_cost0013으로9process/64commands씩/1152command+Runquery창. 정상4상품/3사본→68상품/67사본,추가경로는67동시담보후1회수/66포기. 모든실행성공·requestedbytes정확회수. 수집기최초는일부operation의retainedNone을dict로오인해집계만실패; 성공rawnormal0을보존·재사용하고None을분리하는resume집계로나머지를실행했다. 실패원문/원수집기보존. 측정diagnostic dev와정상release CMVA를구분하며COST_REVIEW에actualheap/RSS/nativework/time을각각기록했다. source/설정은실행중고정했다.
