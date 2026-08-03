# Facility Map, Assignment, And Manifestation Experiment Notes

Status: complete notes

Implementation complete: true

Created: 2026-07-27

이 문서는 P03 조사, 설계 판단, experiment와 범위 밖 후보를 기록한다. Durable gameplay/data/wire
source of truth가 아니다.

## 1. Dependency와 전달 seam

- P01 checkpoint/active-run은 새 facility run-local 결정 상태를 round-trip해야 한다.
- P02 local compendium은 checkpoint/profile copy 없이 현재 singleton을 계속 보존한다.
- P04는 Floor generation transaction에 difficulty reference와 계산을 후속 추가한다.
- P05는 generic Shop-tag requirement와 capstone switch에 content eligibility만 공급한다.

## 2. Current/Target Gap

- current `map_template_id`/`slot_id`는 DTO foundation이지만 `slot_id == depth/lane` validation 때문에
  authored semantic slot identity가 아니다.
- current runtime `edges`와 node depth/lane가 topology authority다. Target은 live template connection
  lookup이며 runtime은 선택 template와 assignment state만 가진다.
- current generator는 weights 뒤 row repair로 category를 바꾼다. Target은 full constrained matching
  가능성 검증과 fail-fast다.
- current BossOmen은 일반 map을 mutation한다. Target은 authored capstone template 원자 교체다.
- current Unity placeholder는 exact live template resource도, reveal/ForwardOnly visual도 아니다.

## 3. Readiness 판단

Decision은 final art를 비범위로 두지만 actual room count, topology와 assignment weight를 live RON
초기 밸런스 authoring으로 명시한다. 따라서 다음은 승인되지 않은 콘텐츠 발명이 아니다.

- Standard intro/branching/final의 최소 실제 room/connection/tag와 증가 관계
- Endless 초기/open-ended pool의 최소 template/profile 세트
- 두 보조 제어실 AND puzzle의 optional manifest slot
- BossOmen Boss -> Exit Gate capstone
- 위 template와 exact 대응하는 functional Unity anchors/reveal/one-way legend

최종 art/sound/VFX를 placeholder로 가장하지 않고 기능 template resource로 검증한다.

## 4. Dirty Worktree

- Core map/BossOmen, live map RON과 Unity Node Map 파일은 P00 이전부터 누적 변경이 있다.
- 기존 변경에는 Support -> SavePoint, live bundle publication, BossOmen exit Gate와 facility node map
  presentation foundation이 포함된다. P03은 이를 target architecture로 수렴시키되 관련 없는 사용자
  변경을 reset/stash/restore하지 않는다.
- repository-wide format이나 snapshot rewrite를 하지 않는다.

## 5. 정책 질문

현재 없음. Actual authoring이 policy의 progression/validation/visibility 조건을 동시에 만족할 수 없거나
P04/P05 전용 수치·content가 필수로 드러날 때만 중단한다.

## 6. 범위 밖 후보

- P02에서 기록한 employee fixture `event_aptitude` drift는 P03과 무관하며 계속 보존한다.
- 기존 screenshots와 final facility art polish는 functional template/cross-project contract 완료 뒤 별도
  visual content 범위다.

## 7. 초기 구현 판단

- pool state는 `cycle`, deterministic remaining order와 run-wide used template ID를 checkpoint-owned
  `FacilityRunState`에 저장한다. 같은 live input에서 재계산할 수 있더라도 load 뒤 pool 재구성을 피한다.
- constrained assignment는 fixed role/tag를 먼저 적용하고 category bound의 남은 수용 가능성을
  backtracking마다 검사한다. 불가능한 template에서 category repair를 하지 않는다.
- semantic slot 전환 뒤 runtime node의 `depth`/`lane`과 `RunMap.edges`는 제거했다. Runtime navigation은
  live template connection을 slot identity로 직접 조회하고 Unity anchor도 `slot_id`만 사용한다.
- Standard/Endless ordinary profiles에는 P04 difficulty reference를 넣지 않았다.

## 8. Typed terminal/capstone/manifestation 판단

- terminal resolution은 room category와 분리된 role이다. Production live template는 Gate/Boss category를
  strict authoring하지만 Core 진행 계약은 Event처럼 별도 content room도 FloorGate role을 수행할 수
  있어 legacy category 추론을 되살리지 않는다.
- BossOmen capstone은 pool state를 소비하지 않는다. `FacilityCapstoneRequest`가 template와 모든 Entry
  payload를 명시하고 generator가 동일 deterministic node-ID/assignment path로 완전한 map을 반환한다.
- BossOmen 승리 Gate는 capstone 생성 시 이미 존재한다. 승리 transaction은 visibility/state만
  monotonic 변경하며 node/slot/edge/payload를 새로 만들지 않는다.
- manifestation의 durable state는 별도 파생 cache가 아니라 checkpoint-owned current `RunMap`의
  prerequisite node Completed와 target visibility/state다. Target assignment 자체도 같은 RunMap에
  저장되어 reconnect/load에서 재추첨되지 않는다.
- Topology authority convergence는 완료됐다. Checkpoint에는 exact template ID/version과 slot assignment를
  저장하되 node-ID edge copy를 저장하지 않으며 load 시 current template contract를 strict 검증한다.

## 9. 최종 감사 판단

- Unity local resource의 connection은 gameplay topology의 두 번째 authority가 아니라 exact visual
  contract다. Direction/reveal을 그리지만 selectable과 transit은 Core projection을 따른다.
- Structural Start는 node definition weighted pool에 넣지 않는다. Template role로 synthetic Start를
  만들며 live RON test도 이 예외를 명시적으로 검증한다.
- P04 difficulty reference와 P05 monolith eligibility/content는 추가하지 않았다. P03가 제공하는 seam은
  generic assignment/capstone/requirement input뿐이다.
- Completion review는 `Complete / High`; immediate correction과 policy question은 없다.
