# 메인터넌스 작업 정보 하단 이동 v8

2026-09-13. 내장 image_gen 편집으로 3개 화면을 제작했다. 각 v7 파일을 편집 대상으로, 휴식-로드아웃.png를 장착자 초상화 참고로 사용했다. 기존 버전은 유지한다.

공통: 정비 완료는 배낭 왼쪽의 화면 하단으로 이동. 벨 위 툴팁 제거. 작업 전후 정보·비용은 트레이 아래 먹 음영 위에 배치. 우측은 배낭 목록에 집중하며 장착 물품의 실제 착용자 초상화를 슬롯 오른쪽 위에 표시한다. 개화 안내 문장 제거. 분쇄 경고는 하단 정보에서 한 번만 표시한다. 비용·효과·소유 상태는 시안용 예시이며 Unity 구현은 포함하지 않는다. 구현에서는 공통 앵커와 슬롯 크기를 통일한다.

## 메인터넌스-강화-v8.png

[이미지](메인터넌스-강화-v8.png)

생성 원본: `/home/acr0tic/.codex/generated_images/01a08f0d-76ed-7412-8b40-453e72acd06a/exec-3fe60bba-fb06-4686-ad44-41a37ae9b5fe.png`

```text
Use case: precise-object-edit. Edit Image1 maintenance gameplay screenshot. Image2 is ONLY reference for tiny equipment-wearer portrait badges in inventory, do not import its scene/characters/layout. Preserve Image1's 1672x941 dimensions, hatch geometry, small flat input slots, black opening, shelf, gold arrowhead header and two currencies, operation tabs at top-right, bottom-left settings/equipment buttons. No physical giant item, no human/hands. Right UI soft ink grain with no enclosing border. Internal borders allowed.
MANDATORY LAYOUT CHANGES:
1 Remove entire bell tooltip/speech bubble and all its text, restore underlying wall. Keep physical bell, cursor and hover rim alone.
2 Move "정비 완료 ›" from far bottom-right to x59–68%, y92–97%, immediately LEFT of right inventory boundary, at bottom of environmental scene. Only ONE copy, no button inside bag. Preserve bottom-left controls.
3 ALL operation comparisons/results/costs/warnings must be relocated out of right bag into a wide compact INFORMATION AREA BELOW PHYSICAL SHELF, x17–66%, y67–88% (around pixels285–1100 horizontally,630–828 vertically). Soft feathered dark ink shading over wall, no enclosing rectangle; thin brass rules and screen-aligned text. Does not overlap shelf/inputslots, exit button or bottom-left buttons. Use 2-3 columns to fit comfortably.
4 Right bag is now JUST item browsing, category tabs and larger grid, no operation result/cost/ready text below. Extend inventory to four columns x four rows of SAME modest slot size, with mostly empty slots; only existing items populated. Dark ink background continues to bottom, no extra call-to-action. No duplication of lower operation information.
5 Wearer portraits are COMMON INVENTORY feature: small square gold-edged actual anime face badges attached upper-right CORNER of equipped item cells, ~28px. Keep item icon/quantity visible. In weapon bag, spear (row1col2) badge silver-haired Brin, staff(row1col4) badge hooded Noah. Match Image2 portrait appearance. Other items not equipped no badge. Target selection blue edge and preparation label remain independent from wearer badge.
6 Remove repeated filler ready messages and "호출벨을 눌러..." paragraphs to keep calm. Remove "스킬의 효과가 변합니다." anywhere. Keep compact drag-to-cancel instruction on shelf. No repeated warnings.
Create coherent production gameplay UI, crisp Korean text, same old brass/ivory/cyan style. Do not resize or rotate UI sprites.
ENHANCE screen: 강화 active tab. Keep two small central slots target 표준 직검 and cost 파편 가루30 with prepared labels.
Below-shelf information layout:
Heading at x18%, y68% "표준 직검 · 강화 미리보기"
Left column x18–39% compact table:
"항목       현재   강화 후"
"강화 단계   +0     +1"
"공격력      18     21" after values cyan
Middle/right x42–65%:
"고유 효과"
"기본 공격 적중 시"
"추가 피해를 줍니다."
Then compact cost line within SAME BELOW-SHELF area "강화 비용  파편 가루 30 · 보유 480 → 450".
No operation details remain under right grid. Selected sword in bag row1col1 blue with 준비 badge, no wearer portrait on this unequipped example sword. Spear/Staff wearer portraits required.
```

## 메인터넌스-분쇄-v8.png

[이미지](메인터넌스-분쇄-v8.png)

생성 원본: `/home/acr0tic/.codex/generated_images/01a08f0d-76ed-7412-8b40-453e72acd06a/exec-b19a3ecd-6f99-4bbe-9067-62c958cd3436.png`

```text
Use case: precise-object-edit. Edit Image1 maintenance gameplay screenshot. Image2 is ONLY reference for tiny equipment-wearer portrait badges in inventory, do not import its scene/characters/layout. Preserve Image1's 1672x941 dimensions, hatch geometry, small flat input slots, black opening, shelf, gold arrowhead header and two currencies, operation tabs at top-right, bottom-left settings/equipment buttons. No physical giant item, no human/hands. Right UI soft ink grain with no enclosing border. Internal borders allowed.
MANDATORY LAYOUT CHANGES:
1 Remove entire bell tooltip/speech bubble and all its text, restore underlying wall. Keep physical bell, cursor and hover rim alone.
2 Move "정비 완료 ›" from far bottom-right to x59–68%, y92–97%, immediately LEFT of right inventory boundary, at bottom of environmental scene. Only ONE copy, no button inside bag. Preserve bottom-left controls.
3 ALL operation comparisons/results/costs/warnings must be relocated out of right bag into a wide compact INFORMATION AREA BELOW PHYSICAL SHELF, x17–66%, y67–88% (around pixels285–1100 horizontally,630–828 vertically). Soft feathered dark ink shading over wall, no enclosing rectangle; thin brass rules and screen-aligned text. Does not overlap shelf/inputslots, exit button or bottom-left buttons. Use 2-3 columns to fit comfortably.
4 Right bag is now JUST item browsing, category tabs and larger grid, no operation result/cost/ready text below. Extend inventory to four columns x four rows of SAME modest slot size, with mostly empty slots; only existing items populated. Dark ink background continues to bottom, no extra call-to-action. No duplication of lower operation information.
5 Wearer portraits are COMMON INVENTORY feature: small square gold-edged actual anime face badges attached upper-right CORNER of equipped item cells, ~28px. Keep item icon/quantity visible. In weapon bag, spear (row1col2) badge silver-haired Brin, staff(row1col4) badge hooded Noah. Match Image2 portrait appearance. Other items not equipped no badge. Target selection blue edge and preparation label remain independent from wearer badge.
6 Remove repeated filler ready messages and "호출벨을 눌러..." paragraphs to keep calm. Remove "스킬의 효과가 변합니다." anywhere. Keep compact drag-to-cancel instruction on shelf. No repeated warnings.
Create coherent production gameplay UI, crisp Korean text, same old brass/ivory/cyan style. Do not resize or rotate UI sprites.
DISMANTLE screen: 분쇄 active tab. Keep one small central framed input slot 예비 직검 and unframed expected powder icon with 획득 예정 파편 가루+12 and 분쇄 후 지급. Input label below sword ONLY 예비 직검, remove red 소모 예정 here so warning only once below shelf.
Remove red warning from shelf entirely, keep drag cancellation instruction only there.
Below-shelf info layout x17–66%, y67–87%:
Heading "예비 직검 · 분쇄 결과"
Left column: "소모 물품  예비 직검 ×1", "상태  미장착"
Right column: "획득 재료  파편 가루 +12", "보유 480 → 492", "작업 비용 없음"
Under columns show EXACTLY ONE red warning line "분쇄하면 선택한 장비가 사라집니다."
Do not repeat warning near input or bell or right inventory. No ready text or call-bell tutorial paragraph. Right 4x4 inventory selected UNEQUIPPED spare sword no portrait, spear Brin badge, staff Noah badge, other weapons and mostly empties. Bell keeps amber hover only, no tooltip. 정비 완료 moved to lower-left scene's RIGHT end x59–68%, not inside bag.
```

## 메인터넌스-개화-v8.png

[이미지](메인터넌스-개화-v8.png)

생성 원본: `/home/acr0tic/.codex/generated_images/01a08f0d-76ed-7412-8b40-453e72acd06a/exec-80cc8392-c19c-496f-8884-5ca51a0de6ca.png`

```text
Use case: precise-object-edit. Edit Image1 maintenance gameplay screenshot. Image2 is ONLY reference for tiny equipment-wearer portrait badges in inventory, do not import its scene/characters/layout. Preserve Image1's 1672x941 dimensions, hatch geometry, small flat input slots, black opening, shelf, gold arrowhead header and two currencies, operation tabs at top-right, bottom-left settings/equipment buttons. No physical giant item, no human/hands. Right UI soft ink grain with no enclosing border. Internal borders allowed.
MANDATORY LAYOUT CHANGES:
1 Remove entire bell tooltip/speech bubble and all its text, restore underlying wall. Keep physical bell, cursor and hover rim alone.
2 Move "정비 완료 ›" from far bottom-right to x59–68%, y92–97%, immediately LEFT of right inventory boundary, at bottom of environmental scene. Only ONE copy, no button inside bag. Preserve bottom-left controls.
3 ALL operation comparisons/results/costs/warnings must be relocated out of right bag into a wide compact INFORMATION AREA BELOW PHYSICAL SHELF, x17–66%, y67–88% (around pixels285–1100 horizontally,630–828 vertically). Soft feathered dark ink shading over wall, no enclosing rectangle; thin brass rules and screen-aligned text. Does not overlap shelf/inputslots, exit button or bottom-left buttons. Use 2-3 columns to fit comfortably.
4 Right bag is now JUST item browsing, category tabs and larger grid, no operation result/cost/ready text below. Extend inventory to four columns x four rows of SAME modest slot size, with mostly empty slots; only existing items populated. Dark ink background continues to bottom, no extra call-to-action. No duplication of lower operation information.
5 Wearer portraits are COMMON INVENTORY feature: small square gold-edged actual anime face badges attached upper-right CORNER of equipped item cells, ~28px. Keep item icon/quantity visible. In weapon bag, spear (row1col2) badge silver-haired Brin, staff(row1col4) badge hooded Noah. Match Image2 portrait appearance. Other items not equipped no badge. Target selection blue edge and preparation label remain independent from wearer badge.
6 Remove repeated filler ready messages and "호출벨을 눌러..." paragraphs to keep calm. Remove "스킬의 효과가 변합니다." anywhere. Keep compact drag-to-cancel instruction on shelf. No repeated warnings.
Create coherent production gameplay UI, crisp Korean text, same old brass/ivory/cyan style. Do not resize or rotate UI sprites.
BLOOM screen: 개화 active tab, bag 파편 selected ONLY (no underline under 무기). Keep small center input slots 수호의 파편 and 엔케팔린20. Remove "스킬의 효과가 변합니다." from shelf with no replacement; retain drag-cancel line only.
Right bag four columns x four rows, first row cyan/red/white fragments and empty, all other cells empty. Cyan shard selected blue border with 준비 label but no wearer portrait (unequipped example). Red shard upper-right portrait silver-haired Brin, white shard upper-right portrait hooded Noah, showing fragment equip state same common inventory rule. Do not import weapons from portrait reference.
Below-shelf x17–66%,y67–89% create two balanced columns for SKILL COMPARISON:
Heading "수호의 파편 · 개화 미리보기"
Left half:
small shield icon, "개화 전 · 보호막"
"자신에게 보호막을"
"부여합니다."
Right half:
small shield-with-aura icon, cyan "개화 후 · 수호의 장막"
"자신과 주변 아군에게"
"보호막을 부여합니다."
Below both a SINGLE compact line "개화 조건 ✓ 충족    비용 엔케팔린 20 · 보유 140 → 120".
No huge icons, no giant arrows, no needless change-summary phrase. All skill text and costs fit between shelf bottom and exit area, not at right. No bell tooltip. 정비 완료 x59–68%, y92–97%.
Use continuous subtle ink wash behind text, no big frame; keep physical shelf visually unobstructed.
```

