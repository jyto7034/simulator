# 위키 이관·보존 기록

[콘텐츠 위키](../README.md)

2026-10-18 이관. 대상은 환상체 조우·파편·유물 설계 노트와 분해 검토 초안 네 문서다. 현재 작업 트리의 원문을 기준으로 하며, Git 기준으로 되돌리거나 다른 작업의 변경을 합치지 않았다.

## 보존 원칙

- 원문 네 파일은 이관 직전 바이트 그대로 보존했다. 원문의 상대 링크를 유지하기 위해 원래 디렉터리에 `pre_wiki_2026-10-18` 사본을 두었다. 이 사본은 수정하지 않는다.
- 원문 전체 행을 겹침·누락 없이 구간으로 나누어 현재 설계·검토·이전 기록 중 한 곳에 배치했다. 본문 변환은 제목 수준과 링크 목적지 정리에 한정하며, 수치·조건·미정 표현은 보존했다.
- 중복된 합의표는 현재 상세 페이지 한 곳에 두고, 다른 원문의 사본은 명시적으로 이관 전 기록에 분리했다.
- 이전 네 파일은 절별 링크 색인으로 바꿨다. 과거 경로와 헤딩 앵커에서 새 상세로 이동할 수 있다.
- 최신 사용자 합의, 현재 설계, 제안, 미정, 이전 설계와 구현 상태를 서로 바꾸지 않았다.

## 이관 원문

| 원문 | 행 수 | 보존 사본 SHA-256 |
| --- | --- | --- |
| [abnormality_design_notes.ko.md](../../skills/abnormality_design_notes.ko.pre_wiki_2026-10-18.md) | 3185 | `83148ba6df527d0a316f52bfc1785fb89462011d83fe6e5fd78297b60d79ae72` |
| [abnormality_skill_fragments_design_notes.md](../../skills/abnormality_skill_fragments_design_notes.pre_wiki_2026-10-18.md) | 4623 | `8278adfc330503517f62264374a7f030defdaea36efb6fc1cac18808116dae80` |
| [run_relic_content_design_notes.ko.md](../../skills/run_relic_content_design_notes.ko.pre_wiki_2026-10-18.md) | 1681 | `1267630512de25017d109baa68490ba215dc699441268dc18f27198c96552b3b` |
| [SKILL_DECOMPOSITION_REVIEW_DRAFT.ko.md](../../skills/SKILL_DECOMPOSITION_REVIEW_DRAFT.ko.pre_wiki_2026-10-18.md) | 981 | `524787fb51da4e2382b24401d531449cf83464f39182618f9ea618e20aa0d302` |

총 10,470행. 상세 구간·소유 페이지·원문 해시·새 본문 해시는 [기계 판독 대응표](migration-map.json)에 기록했다.

## 내용 판정

| 발견한 문제 | 처리 |
| --- | --- |
| 절망의 기사 파편 원문의 검 6개·깨진 맹세와 후속 리워크 충돌 | [최신 파편](../abnormalities/knight-of-despair/fragment.md)에 검 4개 리워크 합의를 이관. 구안은 [이전 기록](../abnormalities/knight-of-despair/review.md)으로 분리 |
| 절망의 기사 장비가 이전 파편의 보호 선택·자기 관통을 참조 | 장비 단독 설계를 보존하고 [장비 페이지](../abnormalities/knight-of-despair/equipment.md)에서 새 파편과의 연결을 미정으로 표시. 임의 재설계하지 않음 |
| 증오의 여왕·음양 태그표가 여러 문서에 반복 | 각 환상체 태그·유물 연결 페이지로 상세 단일화. 이전 표는 대조 기록에 보존 |
| 조우 문서 안에 최종 보스의 파편·장비 상세가 존재 | 백야·고요한 오케스트라·종말새의 상세를 각 파편·장비 페이지로 이관 |
| 조우 문서의 이전 파편 요약과 최신 파편 상세가 섞임 | 조우 본문과 파편 본문을 분리하고 이전 요약은 환상체별 검토 기록에서 대조 |
| 옛 파편 공통 침식 설명과 최신 공통 정책의 범위 차이 | 옛 설명을 보존 기록에 분리하고 현재 공통 규칙을 참조. 전용 용 유물은 에고 침식과 별도 유지 |
| 분해 초안의 조립안·추천 수치가 확정처럼 읽힐 가능성 | 미합의 카드는 검토 페이지로 이동. 합의된 세 리워크와 구분 |
| 원래 문서의 구현·placeholder 관련 설명 | 당시 상태 기록으로 보존. 새 구현 검증이나 현재 실행 결과를 주장하지 않음 |

## 검증

저장소 루트에서 실행한다.

```bash
python3 core_enhanced/docs/content/maintenance/validate_wiki.py --migration-baseline
```

`--migration-baseline`은 이관 직후 본문을 원문 구간과 대조하는 일회 보존 검사다. 이후 합의에 따라 본문을 갱신한 뒤에는 이 옵션을 빼고 링크·원문 사본·구간 대응의 건전성을 검사한다.

검사 결과: [이관 검증 보고서](validation-report.json).

## 구간별 대응

| 원문 구간 | 새 위치 | 구분 |
| --- | --- | --- |
| enc 1–2 | [조우 문서의 도입·이전 요약](../history/enc-context.md#source-enc-1-2) | history |
| enc 3–45 | [환상체 조우 설계 원칙](../principles/encounters.md#source-enc-3-45) | design |
| enc 46–95 | [조우 문서의 도입·이전 요약](../history/enc-context.md#source-enc-46-95) | history |
| enc 96–239 | [백야 · 조우](../abnormalities/white-night/encounter.md#source-enc-96-239) | design |
| enc 240–284 | [백야 · 장비](../abnormalities/white-night/equipment.md#source-enc-240-284) | design |
| enc 285–314 | [백야 · 스킬 파편](../abnormalities/white-night/fragment.md#source-enc-285-314) | design |
| enc 315–369 | [고요한 오케스트라 · 조우](../abnormalities/silent-orchestra/encounter.md#source-enc-315-369) | design |
| enc 370–411 | [고요한 오케스트라 · 장비](../abnormalities/silent-orchestra/equipment.md#source-enc-370-411) | design |
| enc 412–449 | [고요한 오케스트라 · 스킬 파편](../abnormalities/silent-orchestra/fragment.md#source-enc-412-449) | design |
| enc 450–462 | [고요한 오케스트라](../abnormalities/silent-orchestra/README.md#source-enc-450-462) | design |
| enc 463–686 | [종말새 · 조우](../abnormalities/apocalypse-bird/encounter.md#source-enc-463-686) | design |
| enc 687–701 | [종말새 · 스킬 파편](../abnormalities/apocalypse-bird/fragment.md#source-enc-687-701) | design |
| enc 702–746 | [흉탄의 사수 · 조우](../abnormalities/der-fluchschutze/encounter.md#source-enc-702-746) | design |
| enc 747–748 | [조우 문서의 도입·이전 요약](../history/enc-context.md#source-enc-747-748) | history |
| enc 749–871 | [증오의 여왕 · 조우](../abnormalities/queen-of-hatred/encounter.md#source-enc-749-871) | design |
| enc 872–980 | [웃는 시체의 산 · 조우](../abnormalities/mountain-of-smiling-bodies/encounter.md#source-enc-872-980) | design |
| enc 981–989 | [웃는 시체의 산 · 검토·이전 기록](../abnormalities/mountain-of-smiling-bodies/review.md#source-enc-981-989) | history |
| enc 990–1001 | [웃는 시체의 산 · 조우](../abnormalities/mountain-of-smiling-bodies/encounter.md#source-enc-990-1001) | design |
| enc 1002–1103 | [녹아내리는 사랑 · 조우](../abnormalities/melting-love/encounter.md#source-enc-1002-1103) | design |
| enc 1104–1115 | [녹아내리는 사랑 · 검토·이전 기록](../abnormalities/melting-love/review.md#source-enc-1104-1115) | history |
| enc 1116–1248 | [절망의 기사 · 조우](../abnormalities/knight-of-despair/encounter.md#source-enc-1116-1248) | design |
| enc 1249–1259 | [절망의 기사 · 검토·이전 기록](../abnormalities/knight-of-despair/review.md#source-enc-1249-1259) | history |
| enc 1260–1277 | [절망의 기사 · 검토·이전 기록](../abnormalities/knight-of-despair/review.md#source-enc-1260-1277) | history |
| enc 1278–1372 | [마탄의 사수 · 조우](../abnormalities/der-freischutz/encounter.md#source-enc-1278-1372) | design |
| enc 1373–1378 | [마탄의 사수 · 검토·이전 기록](../abnormalities/der-freischutz/review.md#source-enc-1373-1378) | history |
| enc 1379–1429 | [파고드는 천국 · 조우](../abnormalities/burrowing-heaven/encounter.md#source-enc-1379-1429) | design |
| enc 1430–1436 | [파고드는 천국 · 검토·이전 기록](../abnormalities/burrowing-heaven/review.md#source-enc-1430-1436) | history |
| enc 1437–1446 | [파고드는 천국 · 조우](../abnormalities/burrowing-heaven/encounter.md#source-enc-1437-1446) | design |
| enc 1447–1574 | [붉은 두건의 용병 · 조우](../abnormalities/little-red/encounter.md#source-enc-1447-1574) | design |
| enc 1575–1580 | [붉은 두건의 용병 · 검토·이전 기록](../abnormalities/little-red/review.md#source-enc-1575-1580) | history |
| enc 1581–1599 | [붉은 두건의 용병 · 조우](../abnormalities/little-red/encounter.md#source-enc-1581-1599) | design |
| enc 1600–1668 | [죽은 나비들의 장례 · 조우](../abnormalities/funeral-of-dead-butterflies/encounter.md#source-enc-1600-1668) | design |
| enc 1669–1740 | [죽을 나비의 장례 · 조우](../abnormalities/funeral-of-dying-butterflies/encounter.md#source-enc-1669-1740) | design |
| enc 1741–1815 | [아무 것도 없는 · 조우](../abnormalities/nothing-there/encounter.md#source-enc-1741-1815) | design |
| enc 1816–1832 | [아무 것도 없는 · 검토·이전 기록](../abnormalities/nothing-there/review.md#source-enc-1816-1832) | history |
| enc 1833–1841 | [아무 것도 없는 · 조우](../abnormalities/nothing-there/encounter.md#source-enc-1833-1841) | design |
| enc 1842–1924 | [탐욕의 왕 · 조우](../abnormalities/king-of-greed/encounter.md#source-enc-1842-1924) | design |
| enc 1925–1934 | [탐욕의 왕 · 검토·이전 기록](../abnormalities/king-of-greed/review.md#source-enc-1925-1934) | history |
| enc 1935–1946 | [탐욕의 왕 · 조우](../abnormalities/king-of-greed/encounter.md#source-enc-1935-1946) | design |
| enc 1947–2043 | [커다랗고 나쁠 늑대 · 조우](../abnormalities/big-bad-wolf/encounter.md#source-enc-1947-2043) | design |
| enc 2044–2049 | [커다랗고 나쁠 늑대 · 검토·이전 기록](../abnormalities/big-bad-wolf/review.md#source-enc-2044-2049) | history |
| enc 2050–2063 | [커다랗고 나쁠 늑대 · 조우](../abnormalities/big-bad-wolf/encounter.md#source-enc-2050-2063) | design |
| enc 2064–2135 | [음양 · 조우](../abnormalities/yin-yang/encounter.md#source-enc-2064-2135) | design |
| enc 2136–2150 | [음양 · 검토·이전 기록](../abnormalities/yin-yang/review.md#source-enc-2136-2150) | history |
| enc 2151–2161 | [음양 · 검토·이전 기록](../abnormalities/yin-yang/review.md#source-enc-2151-2161) | history |
| enc 2162–2174 | [음양 · 조우](../abnormalities/yin-yang/encounter.md#source-enc-2162-2174) | design |
| enc 2175–2176 | [조우 문서의 도입·이전 요약](../history/enc-context.md#source-enc-2175-2176) | history |
| enc 2177–2234 | [따뜻한 마음의 나무꾼 · 조우](../abnormalities/warm-hearted-woodsman/encounter.md#source-enc-2177-2234) | design |
| enc 2235–2240 | [따뜻한 마음의 나무꾼 · 검토·이전 기록](../abnormalities/warm-hearted-woodsman/review.md#source-enc-2235-2240) | history |
| enc 2241–2251 | [따뜻한 마음의 나무꾼 · 조우](../abnormalities/warm-hearted-woodsman/encounter.md#source-enc-2241-2251) | design |
| enc 2252–2298 | [래티씨아 · 조우](../abnormalities/laetitia/encounter.md#source-enc-2252-2298) | design |
| enc 2299–2304 | [래티씨아 · 검토·이전 기록](../abnormalities/laetitia/review.md#source-enc-2299-2304) | history |
| enc 2305–2316 | [래티씨아 · 조우](../abnormalities/laetitia/encounter.md#source-enc-2305-2316) | design |
| enc 2317–2362 | [여왕벌 · 조우](../abnormalities/queen-bee/encounter.md#source-enc-2317-2362) | design |
| enc 2363–2368 | [여왕벌 · 검토·이전 기록](../abnormalities/queen-bee/review.md#source-enc-2363-2368) | history |
| enc 2369–2377 | [여왕벌 · 조우](../abnormalities/queen-bee/encounter.md#source-enc-2369-2377) | design |
| enc 2378–2407 | [눈의 여왕 · 조우](../abnormalities/snow-queen/encounter.md#source-enc-2378-2407) | design |
| enc 2408–2412 | [눈의 여왕 · 검토·이전 기록](../abnormalities/snow-queen/review.md#source-enc-2408-2412) | history |
| enc 2413–2420 | [눈의 여왕 · 조우](../abnormalities/snow-queen/encounter.md#source-enc-2413-2420) | design |
| enc 2421–2449 | [이름이 없는 태아 · 조우](../abnormalities/nameless-fetus/encounter.md#source-enc-2421-2449) | design |
| enc 2450–2454 | [이름이 없는 태아 · 검토·이전 기록](../abnormalities/nameless-fetus/review.md#source-enc-2450-2454) | history |
| enc 2455–2462 | [이름이 없는 태아 · 조우](../abnormalities/nameless-fetus/encounter.md#source-enc-2455-2462) | design |
| enc 2463–2510 | [흑조의 꿈 · 조우](../abnormalities/dream-of-black-swan/encounter.md#source-enc-2463-2510) | design |
| enc 2511–2523 | [흑조의 꿈 · 검토·이전 기록](../abnormalities/dream-of-black-swan/review.md#source-enc-2511-2523) | history |
| enc 2524–2533 | [흑조의 꿈 · 조우](../abnormalities/dream-of-black-swan/encounter.md#source-enc-2524-2533) | design |
| enc 2534–2601 | [포큐버스 · 조우](../abnormalities/porccubus/encounter.md#source-enc-2534-2601) | design |
| enc 2602–2616 | [포큐버스 · 검토·이전 기록](../abnormalities/porccubus/review.md#source-enc-2602-2616) | history |
| enc 2617–2627 | [포큐버스 · 조우](../abnormalities/porccubus/encounter.md#source-enc-2617-2627) | design |
| enc 2628–2671 | [샤덴프로이데 · 조우](../abnormalities/schadenfreude/encounter.md#source-enc-2628-2671) | design |
| enc 2672–2681 | [샤덴프로이데 · 검토·이전 기록](../abnormalities/schadenfreude/review.md#source-enc-2672-2681) | history |
| enc 2682–2690 | [샤덴프로이데 · 조우](../abnormalities/schadenfreude/encounter.md#source-enc-2682-2690) | design |
| enc 2691–2735 | [풍운승려 · 조우](../abnormalities/clouded-monk/encounter.md#source-enc-2691-2735) | design |
| enc 2736–2745 | [풍운승려 · 검토·이전 기록](../abnormalities/clouded-monk/review.md#source-enc-2736-2745) | history |
| enc 2746–2755 | [풍운승려 · 조우](../abnormalities/clouded-monk/encounter.md#source-enc-2746-2755) | design |
| enc 2756–2805 | [썰매의 루돌타 · 조우](../abnormalities/rudolta/encounter.md#source-enc-2756-2805) | design |
| enc 2806–2813 | [썰매의 루돌타 · 검토·이전 기록](../abnormalities/rudolta/review.md#source-enc-2806-2813) | history |
| enc 2814–2822 | [썰매의 루돌타 · 조우](../abnormalities/rudolta/encounter.md#source-enc-2814-2822) | design |
| enc 2823–2873 | [지혜를 구하는 허수아비 · 조우](../abnormalities/scarecrow-searching-for-wisdom/encounter.md#source-enc-2823-2873) | design |
| enc 2874–2883 | [지혜를 구하는 허수아비 · 검토·이전 기록](../abnormalities/scarecrow-searching-for-wisdom/review.md#source-enc-2874-2883) | history |
| enc 2884–2893 | [지혜를 구하는 허수아비 · 조우](../abnormalities/scarecrow-searching-for-wisdom/encounter.md#source-enc-2884-2893) | design |
| enc 2894–2897 | [조우 문서의 도입·이전 요약](../history/enc-context.md#source-enc-2894-2897) | history |
| enc 2898–2937 | [도구 공통 규칙](../principles/tools.md#source-enc-2898-2937) | design |
| enc 2938–2963 | [거목수액](../tools/sap.md#source-enc-2938-2963) | design |
| enc 2964–2998 | [이계의 초상](../tools/portrait.md#source-enc-2964-2998) | design |
| enc 2999–3022 | [발광 팔찌](../tools/luminous-bracelet.md#source-enc-2999-3022) | design |
| enc 3023–3050 | [살점 우상](../tools/flesh-idol.md#source-enc-3023-3050) | design |
| enc 3051–3080 | [부서져 가는 갑옷](../tools/crumbling-armor.md#source-enc-3051-3080) | design |
| enc 3081–3098 | [피의 욕조](../tools/bloodbath.md#source-enc-3081-3098) | design |
| enc 3099–3121 | [오래전의 믿음과 약속](../tools/old-faith-and-promise.md#source-enc-3099-3121) | design |
| enc 3122–3152 | [지옥행 급행열차](../tools/express-train.md#source-enc-3122-3152) | design |
| enc 3153–3185 | [뚜껑 따인 웰치어스](../tools/opened-can-of-wellcheers.md#source-enc-3153-3185) | design |
| frag 1–23 | [파편 문서의 도입·이전 공통 설명](../history/frag-context.md#source-frag-1-23) | history |
| frag 24–88 | [스킬 파편 설계 원칙](../principles/fragments.md#source-frag-24-88) | design |
| frag 89–225 | [파편 문서의 도입·이전 공통 설명](../history/frag-context.md#source-frag-89-225) | history |
| frag 226–227 | [백야 · 검토·이전 기록](../abnormalities/white-night/review.md#source-frag-226-227) | history |
| frag 228–229 | [파편 문서의 도입·이전 공통 설명](../history/frag-context.md#source-frag-228-229) | history |
| frag 230–234 | [절망의 기사 · 검토·이전 기록](../abnormalities/knight-of-despair/review.md#source-frag-230-234) | history |
| frag 235–247 | [절망의 기사 · 원작 참고](../abnormalities/knight-of-despair/reference.md#source-frag-235-247) | design |
| frag 248–367 | [절망의 기사 · 검토·이전 기록](../abnormalities/knight-of-despair/review.md#source-frag-248-367) | superseded |
| frag 368–440 | [절망의 기사 · 장비](../abnormalities/knight-of-despair/equipment.md#source-frag-368-440) | design |
| frag 441–469 | [절망의 기사 · 검토·이전 기록](../abnormalities/knight-of-despair/review.md#source-frag-441-469) | superseded |
| frag 470–477 | [증오의 여왕](../abnormalities/queen-of-hatred/README.md#source-frag-470-477) | design |
| frag 478–560 | [증오의 여왕 · 스킬 파편](../abnormalities/queen-of-hatred/fragment.md#source-frag-478-560) | design |
| frag 561–577 | [증오의 여왕 · 검토·이전 기록](../abnormalities/queen-of-hatred/review.md#source-frag-561-577) | reference-copy |
| frag 578–600 | [증오의 여왕 · 스킬 파편](../abnormalities/queen-of-hatred/fragment.md#source-frag-578-600) | design |
| frag 601–666 | [증오의 여왕 · 장비](../abnormalities/queen-of-hatred/equipment.md#source-frag-601-666) | design |
| frag 667–687 | [증오의 여왕](../abnormalities/queen-of-hatred/README.md#source-frag-667-687) | design |
| frag 688–703 | [음양](../abnormalities/yin-yang/README.md#source-frag-688-703) | design |
| frag 704–750 | [음양 · 스킬 파편](../abnormalities/yin-yang/fragment.md#source-frag-704-750) | design |
| frag 751–776 | [음양 · 검토·이전 기록](../abnormalities/yin-yang/review.md#source-frag-751-776) | reference-copy |
| frag 777–794 | [음양 · 검토·이전 기록](../abnormalities/yin-yang/review.md#source-frag-777-794) | reference-copy |
| frag 795–852 | [음양 · 장비](../abnormalities/yin-yang/equipment.md#source-frag-795-852) | design |
| frag 853–873 | [음양](../abnormalities/yin-yang/README.md#source-frag-853-873) | design |
| frag 874–893 | [음양 · 스킬 파편](../abnormalities/yin-yang/fragment.md#source-frag-874-893) | design |
| frag 894–902 | [붉은 두건의 용병](../abnormalities/little-red/README.md#source-frag-894-902) | design |
| frag 903–997 | [붉은 두건의 용병 · 스킬 파편](../abnormalities/little-red/fragment.md#source-frag-903-997) | design |
| frag 998–1117 | [붉은 두건의 용병 · 장비](../abnormalities/little-red/equipment.md#source-frag-998-1117) | design |
| frag 1118–1128 | [붉은 두건의 용병](../abnormalities/little-red/README.md#source-frag-1118-1128) | design |
| frag 1129–1140 | [커다랗고 나쁠 늑대](../abnormalities/big-bad-wolf/README.md#source-frag-1129-1140) | design |
| frag 1141–1240 | [커다랗고 나쁠 늑대 · 스킬 파편](../abnormalities/big-bad-wolf/fragment.md#source-frag-1141-1240) | design |
| frag 1241–1367 | [커다랗고 나쁠 늑대 · 장비](../abnormalities/big-bad-wolf/equipment.md#source-frag-1241-1367) | design |
| frag 1368–1380 | [커다랗고 나쁠 늑대](../abnormalities/big-bad-wolf/README.md#source-frag-1368-1380) | design |
| frag 1381–1388 | [죽은 나비들의 장례](../abnormalities/funeral-of-dead-butterflies/README.md#source-frag-1381-1388) | design |
| frag 1389–1393 | [죽은 나비들의 장례 · 스킬 파편](../abnormalities/funeral-of-dead-butterflies/fragment.md#source-frag-1389-1393) | design |
| frag 1394–1402 | [죽을 나비의 장례](../abnormalities/funeral-of-dying-butterflies/README.md#source-frag-1394-1402) | design |
| frag 1403–1500 | [죽을 나비의 장례 · 스킬 파편](../abnormalities/funeral-of-dying-butterflies/fragment.md#source-frag-1403-1500) | design |
| frag 1501–1601 | [죽을 나비의 장례 · 장비](../abnormalities/funeral-of-dying-butterflies/equipment.md#source-frag-1501-1601) | design |
| frag 1602–1608 | [죽을 나비의 장례](../abnormalities/funeral-of-dying-butterflies/README.md#source-frag-1602-1608) | design |
| frag 1609–1617 | [녹아내리는 사랑](../abnormalities/melting-love/README.md#source-frag-1609-1617) | design |
| frag 1618–1805 | [녹아내리는 사랑 · 스킬 파편](../abnormalities/melting-love/fragment.md#source-frag-1618-1805) | design |
| frag 1806–1917 | [녹아내리는 사랑 · 장비](../abnormalities/melting-love/equipment.md#source-frag-1806-1917) | design |
| frag 1918–1925 | [녹아내리는 사랑](../abnormalities/melting-love/README.md#source-frag-1918-1925) | design |
| frag 1926–1934 | [파고드는 천국](../abnormalities/burrowing-heaven/README.md#source-frag-1926-1934) | design |
| frag 1935–1972 | [파고드는 천국 · 스킬 파편](../abnormalities/burrowing-heaven/fragment.md#source-frag-1935-1972) | design |
| frag 1973–2037 | [파고드는 천국 · 장비](../abnormalities/burrowing-heaven/equipment.md#source-frag-1973-2037) | design |
| frag 2038–2049 | [파고드는 천국](../abnormalities/burrowing-heaven/README.md#source-frag-2038-2049) | design |
| frag 2050–2060 | [아무 것도 없는](../abnormalities/nothing-there/README.md#source-frag-2050-2060) | design |
| frag 2061–2125 | [아무 것도 없는 · 스킬 파편](../abnormalities/nothing-there/fragment.md#source-frag-2061-2125) | design |
| frag 2126–2210 | [아무 것도 없는 · 장비](../abnormalities/nothing-there/equipment.md#source-frag-2126-2210) | design |
| frag 2211–2224 | [아무 것도 없는](../abnormalities/nothing-there/README.md#source-frag-2211-2224) | design |
| frag 2225–2238 | [웃는 시체의 산](../abnormalities/mountain-of-smiling-bodies/README.md#source-frag-2225-2238) | design |
| frag 2239–2335 | [웃는 시체의 산 · 스킬 파편](../abnormalities/mountain-of-smiling-bodies/fragment.md#source-frag-2239-2335) | design |
| frag 2336–2436 | [웃는 시체의 산 · 장비](../abnormalities/mountain-of-smiling-bodies/equipment.md#source-frag-2336-2436) | design |
| frag 2437–2463 | [웃는 시체의 산](../abnormalities/mountain-of-smiling-bodies/README.md#source-frag-2437-2463) | design |
| frag 2464–2476 | [탐욕의 왕](../abnormalities/king-of-greed/README.md#source-frag-2464-2476) | design |
| frag 2477–2562 | [탐욕의 왕 · 스킬 파편](../abnormalities/king-of-greed/fragment.md#source-frag-2477-2562) | design |
| frag 2563–2686 | [탐욕의 왕 · 장비](../abnormalities/king-of-greed/equipment.md#source-frag-2563-2686) | design |
| frag 2687–2711 | [탐욕의 왕](../abnormalities/king-of-greed/README.md#source-frag-2687-2711) | design |
| frag 2712–2747 | [흉탄의 사수](../abnormalities/der-fluchschutze/README.md#source-frag-2712-2747) | design |
| frag 2748–2848 | [흉탄의 사수 · 스킬 파편](../abnormalities/der-fluchschutze/fragment.md#source-frag-2748-2848) | design |
| frag 2849–2870 | [흉탄의 사수 · 장비](../abnormalities/der-fluchschutze/equipment.md#source-frag-2849-2870) | design |
| frag 2871–2875 | [흉탄의 사수 · 스킬 파편](../abnormalities/der-fluchschutze/fragment.md#source-frag-2871-2875) | design |
| frag 2876–2892 | [흉탄의 사수 · 장비](../abnormalities/der-fluchschutze/equipment.md#source-frag-2876-2892) | design |
| frag 2893–2907 | [흉탄의 사수](../abnormalities/der-fluchschutze/README.md#source-frag-2893-2907) | design |
| frag 2908–2909 | [파편 문서의 도입·이전 공통 설명](../history/frag-context.md#source-frag-2908-2909) | history |
| frag 2910–2918 | [따뜻한 마음의 나무꾼](../abnormalities/warm-hearted-woodsman/README.md#source-frag-2910-2918) | design |
| frag 2919–2945 | [따뜻한 마음의 나무꾼 · 스킬 파편](../abnormalities/warm-hearted-woodsman/fragment.md#source-frag-2919-2945) | design |
| frag 2946–2956 | [따뜻한 마음의 나무꾼 · 검토·이전 기록](../abnormalities/warm-hearted-woodsman/review.md#source-frag-2946-2956) | history |
| frag 2957–3014 | [따뜻한 마음의 나무꾼 · 장비](../abnormalities/warm-hearted-woodsman/equipment.md#source-frag-2957-3014) | design |
| frag 3015–3032 | [따뜻한 마음의 나무꾼](../abnormalities/warm-hearted-woodsman/README.md#source-frag-3015-3032) | design |
| frag 3033–3042 | [래티씨아](../abnormalities/laetitia/README.md#source-frag-3033-3042) | design |
| frag 3043–3088 | [래티씨아 · 스킬 파편](../abnormalities/laetitia/fragment.md#source-frag-3043-3088) | design |
| frag 3089–3112 | [래티씨아 · 검토·이전 기록](../abnormalities/laetitia/review.md#source-frag-3089-3112) | history |
| frag 3113–3183 | [래티씨아 · 장비](../abnormalities/laetitia/equipment.md#source-frag-3113-3183) | design |
| frag 3184–3208 | [래티씨아](../abnormalities/laetitia/README.md#source-frag-3184-3208) | design |
| frag 3209–3219 | [여왕벌](../abnormalities/queen-bee/README.md#source-frag-3209-3219) | design |
| frag 3220–3248 | [여왕벌 · 스킬 파편](../abnormalities/queen-bee/fragment.md#source-frag-3220-3248) | design |
| frag 3249–3296 | [여왕벌 · 장비](../abnormalities/queen-bee/equipment.md#source-frag-3249-3296) | design |
| frag 3297–3317 | [여왕벌](../abnormalities/queen-bee/README.md#source-frag-3297-3317) | design |
| frag 3318–3328 | [눈의 여왕](../abnormalities/snow-queen/README.md#source-frag-3318-3328) | design |
| frag 3329–3359 | [눈의 여왕 · 스킬 파편](../abnormalities/snow-queen/fragment.md#source-frag-3329-3359) | design |
| frag 3360–3376 | [눈의 여왕 · 검토·이전 기록](../abnormalities/snow-queen/review.md#source-frag-3360-3376) | history |
| frag 3377–3429 | [눈의 여왕 · 장비](../abnormalities/snow-queen/equipment.md#source-frag-3377-3429) | design |
| frag 3430–3448 | [눈의 여왕](../abnormalities/snow-queen/README.md#source-frag-3430-3448) | design |
| frag 3449–3468 | [이름이 없는 태아](../abnormalities/nameless-fetus/README.md#source-frag-3449-3468) | design |
| frag 3469–3591 | [이름이 없는 태아 · 스킬 파편](../abnormalities/nameless-fetus/fragment.md#source-frag-3469-3591) | design |
| frag 3592–3670 | [이름이 없는 태아 · 장비](../abnormalities/nameless-fetus/equipment.md#source-frag-3592-3670) | design |
| frag 3671–3699 | [이름이 없는 태아](../abnormalities/nameless-fetus/README.md#source-frag-3671-3699) | design |
| frag 3700–3714 | [흑조의 꿈](../abnormalities/dream-of-black-swan/README.md#source-frag-3700-3714) | design |
| frag 3715–3767 | [흑조의 꿈 · 스킬 파편](../abnormalities/dream-of-black-swan/fragment.md#source-frag-3715-3767) | design |
| frag 3768–3787 | [깨어난 흑조](../relics/awakened-black-swan.md#source-frag-3768-3787) | design |
| frag 3788–3844 | [흑조의 꿈 · 장비](../abnormalities/dream-of-black-swan/equipment.md#source-frag-3788-3844) | design |
| frag 3845–3858 | [흑조의 꿈](../abnormalities/dream-of-black-swan/README.md#source-frag-3845-3858) | design |
| frag 3859–3870 | [포큐버스](../abnormalities/porccubus/README.md#source-frag-3859-3870) | design |
| frag 3871–3881 | [포큐버스 · 원작 참고](../abnormalities/porccubus/reference.md#source-frag-3871-3881) | design |
| frag 3882–3953 | [포큐버스 · 스킬 파편](../abnormalities/porccubus/fragment.md#source-frag-3882-3953) | design |
| frag 3954–3974 | [포큐버스 · 검토·이전 기록](../abnormalities/porccubus/review.md#source-frag-3954-3974) | history |
| frag 3975–4036 | [포큐버스 · 장비](../abnormalities/porccubus/equipment.md#source-frag-3975-4036) | design |
| frag 4037–4058 | [포큐버스](../abnormalities/porccubus/README.md#source-frag-4037-4058) | design |
| frag 4059–4077 | [샤덴프로이데](../abnormalities/schadenfreude/README.md#source-frag-4059-4077) | design |
| frag 4078–4108 | [샤덴프로이데 · 스킬 파편](../abnormalities/schadenfreude/fragment.md#source-frag-4078-4108) | design |
| frag 4109–4164 | [샤덴프로이데 · 장비](../abnormalities/schadenfreude/equipment.md#source-frag-4109-4164) | design |
| frag 4165–4174 | [샤덴프로이데](../abnormalities/schadenfreude/README.md#source-frag-4165-4174) | design |
| frag 4175–4191 | [풍운승려](../abnormalities/clouded-monk/README.md#source-frag-4175-4191) | design |
| frag 4192–4225 | [풍운승려 · 스킬 파편](../abnormalities/clouded-monk/fragment.md#source-frag-4192-4225) | design |
| frag 4226–4282 | [풍운승려 · 장비](../abnormalities/clouded-monk/equipment.md#source-frag-4226-4282) | design |
| frag 4283–4302 | [풍운승려](../abnormalities/clouded-monk/README.md#source-frag-4283-4302) | design |
| frag 4303–4319 | [썰매의 루돌타](../abnormalities/rudolta/README.md#source-frag-4303-4319) | design |
| frag 4320–4359 | [썰매의 루돌타 · 스킬 파편](../abnormalities/rudolta/fragment.md#source-frag-4320-4359) | design |
| frag 4360–4437 | [썰매의 루돌타 · 장비](../abnormalities/rudolta/equipment.md#source-frag-4360-4437) | design |
| frag 4438–4459 | [썰매의 루돌타](../abnormalities/rudolta/README.md#source-frag-4438-4459) | design |
| frag 4460–4472 | [지혜를 구하는 허수아비](../abnormalities/scarecrow-searching-for-wisdom/README.md#source-frag-4460-4472) | design |
| frag 4473–4526 | [지혜를 구하는 허수아비 · 스킬 파편](../abnormalities/scarecrow-searching-for-wisdom/fragment.md#source-frag-4473-4526) | design |
| frag 4527–4611 | [지혜를 구하는 허수아비 · 장비](../abnormalities/scarecrow-searching-for-wisdom/equipment.md#source-frag-4527-4611) | design |
| frag 4612–4623 | [지혜를 구하는 허수아비](../abnormalities/scarecrow-searching-for-wisdom/README.md#source-frag-4612-4623) | design |
| rel 1–47 | [유물 논의·진행 기록](../history/rel-context.md#source-rel-1-47) | history |
| rel 48–275 | [유물 설계 원칙](../principles/relics.md#source-rel-48-275) | design |
| rel 276–424 | [효과 분해와 15개 검토 축](../principles/effect-review.md#source-rel-276-424) | design |
| rel 425–572 | [태그 사전](../tags/README.md#source-rel-425-572) | design |
| rel 573–605 | [증오의 여왕 · 태그·유물 연결](../abnormalities/queen-of-hatred/synergies.md#source-rel-573-605) | design |
| rel 606–622 | [음양 · 태그·유물 연결](../abnormalities/yin-yang/synergies.md#source-rel-606-622) | design |
| rel 623–625 | [태그 사전](../tags/README.md#source-rel-623-625) | design |
| rel 626–634 | [음양 · 태그·유물 연결](../abnormalities/yin-yang/synergies.md#source-rel-626-634) | design |
| rel 635–684 | [태그 사전](../tags/README.md#source-rel-635-684) | design |
| rel 685–728 | [유물의 발동·출처·연쇄](../principles/relic-effects.md#source-rel-685-728) | design |
| rel 729–730 | [유물 논의·진행 기록](../history/rel-context.md#source-rel-729-730) | history |
| rel 731–799 | [음양 전용 이스터에그 유물](../relics/yin-yang-dragon.md#source-rel-731-799) | design |
| rel 800–913 | [무기 수치 강화 유물 9종](../relics/weapon-stat-bonuses.md#source-rel-800-913) | design |
| rel 914–1016 | [깊은 쐐기](../relics/deep-wedge.md#source-rel-914-1016) | design |
| rel 1017–1044 | [붉은 실타래](../relics/red-thread.md#source-rel-1017-1044) | design |
| rel 1045–1075 | [소환물 피해 후 약화](../relics/summon-vulnerability.md#source-rel-1045-1075) | design |
| rel 1076–1107 | [깨진 균형](../relics/broken-balance.md#source-rel-1076-1107) | design |
| rel 1108–1149 | [다음.](../relics/next.md#source-rel-1108-1149) | design |
| rel 1150–1177 | [가하는 회복량 증가](../relics/healing-increase.md#source-rel-1150-1177) | design |
| rel 1178–1229 | [이로운 스택당 공격력 증가](../relics/beneficial-stack-attack.md#source-rel-1178-1229) | design |
| rel 1230–1265 | [눈의 여왕 전용 유물](../relics/snow-queen-remnant.md#source-rel-1230-1265) | design |
| rel 1266–1350 | [단일 대상 전환·효과 증폭](../relics/single-target-conversion.md#source-rel-1266-1350) | design |
| rel 1351–1360 | [이름이 없는 태아 · 태그·유물 연결](../abnormalities/nameless-fetus/synergies.md#source-rel-1351-1360) | design |
| rel 1361–1373 | [이름이 없는 태아 · 검토·이전 기록](../abnormalities/nameless-fetus/review.md#source-rel-1361-1373) | reference-copy |
| rel 1374–1396 | [이름이 없는 태아 · 태그·유물 연결](../abnormalities/nameless-fetus/synergies.md#source-rel-1374-1396) | design |
| rel 1397–1409 | [긴 호흡](../relics/long-breath.md#source-rel-1397-1409) | design |
| rel 1410–1420 | [울림통](../relics/resonator.md#source-rel-1410-1420) | design |
| rel 1421–1458 | [이름이 없는 태아 · 태그·유물 연결](../abnormalities/nameless-fetus/synergies.md#source-rel-1421-1458) | design |
| rel 1459–1468 | [저체력 유물 조합](../builds/low-hp.md#source-rel-1459-1468) | design |
| rel 1469–1481 | [착용자 받는 피해 증가](../relics/increased-damage-taken.md#source-rel-1469-1481) | design |
| rel 1482–1498 | [잃은 체력 비례 공격력 증가](../relics/missing-hp-attack.md#source-rel-1482-1498) | design |
| rel 1499–1569 | [빌려온 댓가](../relics/borrowed-life.md#source-rel-1499-1569) | design |
| rel 1570–1609 | [저체력 유물 조합](../builds/low-hp.md#source-rel-1570-1609) | design |
| rel 1610–1656 | [이전 유물 검토 순서](../history/relic-design-order.md#source-rel-1610-1656) | history |
| rel 1657–1681 | [효과 분해와 15개 검토 축](../principles/effect-review.md#source-rel-1657-1681) | design |
| draft 1–20 | [분해 검토의 도입·이전 목차](../history/draft-context.md#source-draft-1-20) | history |
| draft 21–125 | [분해 검토의 공통 판정·조립 제안](../review/common-rules.md#source-draft-21-125) | proposal |
| draft 126–127 | [분해 검토의 도입·이전 목차](../history/draft-context.md#source-draft-126-127) | history |
| draft 128–223 | [절망의 기사 · 스킬 파편](../abnormalities/knight-of-despair/fragment.md#source-draft-128-223) | design |
| draft 224–252 | [절망의 기사 · 태그·유물 연결](../abnormalities/knight-of-despair/synergies.md#source-draft-224-252) | design |
| draft 253–254 | [절망의 기사 · 스킬 파편](../abnormalities/knight-of-despair/fragment.md#source-draft-253-254) | design |
| draft 255–290 | [절망의 기사 · 검토·이전 기록](../abnormalities/knight-of-despair/review.md#source-draft-255-290) | superseded |
| draft 291–329 | [증오의 여왕 · 검토·이전 기록](../abnormalities/queen-of-hatred/review.md#source-draft-291-329) | reference-copy |
| draft 330–371 | [음양 · 검토·이전 기록](../abnormalities/yin-yang/review.md#source-draft-330-371) | reference-copy |
| draft 372–398 | [붉은 두건의 용병 · 검토·이전 기록](../abnormalities/little-red/review.md#source-draft-372-398) | proposal |
| draft 399–423 | [커다랗고 나쁠 늑대 · 검토·이전 기록](../abnormalities/big-bad-wolf/review.md#source-draft-399-423) | proposal |
| draft 424–450 | [죽을 나비의 장례 · 검토·이전 기록](../abnormalities/funeral-of-dying-butterflies/review.md#source-draft-424-450) | proposal |
| draft 451–500 | [녹아내리는 사랑 · 검토·이전 기록](../abnormalities/melting-love/review.md#source-draft-451-500) | proposal |
| draft 501–524 | [파고드는 천국 · 검토·이전 기록](../abnormalities/burrowing-heaven/review.md#source-draft-501-524) | proposal |
| draft 525–548 | [아무 것도 없는 · 검토·이전 기록](../abnormalities/nothing-there/review.md#source-draft-525-548) | proposal |
| draft 549–572 | [웃는 시체의 산 · 검토·이전 기록](../abnormalities/mountain-of-smiling-bodies/review.md#source-draft-549-572) | proposal |
| draft 573–598 | [탐욕의 왕 · 검토·이전 기록](../abnormalities/king-of-greed/review.md#source-draft-573-598) | proposal |
| draft 599–625 | [흉탄의 사수 · 검토·이전 기록](../abnormalities/der-fluchschutze/review.md#source-draft-599-625) | proposal |
| draft 626–649 | [따뜻한 마음의 나무꾼 · 검토·이전 기록](../abnormalities/warm-hearted-woodsman/review.md#source-draft-626-649) | proposal |
| draft 650–682 | [래티씨아 · 검토·이전 기록](../abnormalities/laetitia/review.md#source-draft-650-682) | proposal |
| draft 683–703 | [여왕벌 · 검토·이전 기록](../abnormalities/queen-bee/review.md#source-draft-683-703) | proposal |
| draft 704–728 | [눈의 여왕 · 검토·이전 기록](../abnormalities/snow-queen/review.md#source-draft-704-728) | proposal |
| draft 729–754 | [이름이 없는 태아 · 검토·이전 기록](../abnormalities/nameless-fetus/review.md#source-draft-729-754) | proposal |
| draft 755–779 | [흑조의 꿈 · 검토·이전 기록](../abnormalities/dream-of-black-swan/review.md#source-draft-755-779) | proposal |
| draft 780–802 | [포큐버스 · 검토·이전 기록](../abnormalities/porccubus/review.md#source-draft-780-802) | proposal |
| draft 803–823 | [샤덴프로이데 · 검토·이전 기록](../abnormalities/schadenfreude/review.md#source-draft-803-823) | proposal |
| draft 824–846 | [풍운승려 · 검토·이전 기록](../abnormalities/clouded-monk/review.md#source-draft-824-846) | proposal |
| draft 847–870 | [썰매의 루돌타 · 검토·이전 기록](../abnormalities/rudolta/review.md#source-draft-847-870) | proposal |
| draft 871–894 | [지혜를 구하는 허수아비 · 검토·이전 기록](../abnormalities/scarecrow-searching-for-wisdom/review.md#source-draft-871-894) | proposal |
| draft 895–911 | [회사 지급 스킬](../skills/company-skills.md#source-draft-895-911) | proposal |
| draft 912–981 | [장비·유물 연결과 남은 분해 검토](../review/cross-content.md#source-draft-912-981) | proposal |
