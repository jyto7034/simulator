# U16-3 구현 자체 검토

실제 수정 전527파일 baseline과 현재27파일 product/test/data/document delta를 SOURCE_DELTA_REVIEW.json에 연결했다. 기존 dirty·U16-2동결/감사 원본은 보존한다. 이 검토는 비참여 감사가 아니다.

핵심 호출을 직접 검토했다. Gambling 배정은 optional validated policy가 있을 때 정상 Reward resolver로 진입하고 blocking VN→선택 hook→typed DealerAct→한 owner/route→transaction→query에 도달한다. claim/skip/complete 일반경로로 정산을 우회할 수 없다. revision·phase·player operation 자격과 VN source/node가 확인되며 실패는 application draft와 기존 state transaction에서 원복된다. stale/duplicate/insufficient 및 실제 저장 실패 대조가 코드·원시 결과에 있다.

카드/주사위 기대값은 RULES의 판본·수작업 Ace/natural/push,8조합/kicker,94장/SecondChance/지연Freeze·FlipThree/점수공동1위로 검토했다. 두 NPC의 의사결정 입력은 자신의 손과 공개 tableau뿐이다. 공개 view는 BJ hole·미래덱·seed/cursor를 만들지 않는다. 한 판/마지막결과와 현재 deck만 보존하며 반복 시 조우 이력 벡터가 늘지 않는다. signed 손실·음수잔액을 clamp하지 않고 natural은i128 중간계산, 갬빗은 실제게임delta에만 적용된다. 게임전 취소는 무차감이고 기왕 선택한갬빗은 유지된다.

담보는 기존 미장착 사본을 escrow로 이동해 live copy API에서 찾을 수 없게 한다. 지급후양수/원금/선택을 확정전 검증하고 실제그사본 UUID/variant를 그대로 돌려준다. 지원RON의 다른rarity도 기본10 또는authored원금으로 담보가능하다. 정산은 돌이킬 수 없고 나머지사본을 포기한 뒤에만 상점으로 이동한다. 구매는 기존 EconomyGrantState와 RewardEffect를 사용하여 가격·재고·새 Normal사본을 함께확정한다. 공식 emptyoffers는 그대로 표시하는 빈상점이고 fixture분기/강제공급은 없다.

공유 변경은 새 command/result 한 개와 optional boxed session, default/skip false 설명flag, normalVN hook, returned escrow helper에 한정했다. 구현 전 사본/일반reward/U16-2소스는 제거하지 않았다. global85command/56result exhaustive projection과72integration을 확인했다. 최종소스의35집중 사례, Core고유2213와21fresh host153commands가 현재 경계를 검증한다. 기존native4/server31/socket1 재사용은 BASELINE_REUSE에 input/nonimpact근거로 분리했다.

원시 실패는 EXPERIMENTS에 제품/fixture/도구/환경으로 구분했다. 처음 작성한speaker/VN위치/transport fixture와 준비노드전략을 바로잡았고 검사를완화하지않았다. 기본10M 준비초과는 중복Run 관측 제거·scoped query로 해결했으며 실제거래/전투10clocks/관측조건은 유지했다. 제품기본한도는 바꾸지않았다.

상태/비용은 STATE_INVENTORY/COST_REVIEW, 시험선정·재사용은 TEST_RESULTS_INDEX/BASELINE_REUSE, 전체21/10/5/2은 COVERAGE를 따른다. 별도 구현 비참여 서브에이전트의 GEX20 verdict만 Goal 완료를 확정할 수 있다.
