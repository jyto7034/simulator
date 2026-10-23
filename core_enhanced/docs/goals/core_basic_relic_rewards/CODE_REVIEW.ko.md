# U16-2 현재 경계와 구현 차이

P2G-20 정적 검토. 실제 기능 검증은 아직 하지 않았다. 현재 checkout의 기존 dirty를 보존하며 Released/Deployed 상태는 주장하지 않는다.

| 경계 | 현재 구현 | 필요한 변경 / 검증 |
| --- | --- | --- |
| MAP | FacilityLayout.allocate는5종 각20%, 예약 SimpleChoice를 고정한다. facility_node_support는 모든 reward_activity를 미지원 처리한다. | 세 활동만 실행 지원, 기존 배정 유지. layout에 정상 작성 후보/가루 정책을 연결하고 입장 전 조건 조회. |
| REWARD | RewardSessionState는 mode/candidates/selection/can_skip, 기존 map reward는 일반 reward catalog에서1개를 고른다. Claim은 InRewardClaimed로만 전이한다. | 유물 activity 세션을 기존 reward 세션에 추가. SimpleChoice와 투자 수령은 공통grant 후 같은 transaction 안에서 node 완료. 빈 후보도 안내/퇴장 가능한 세션. 기존 전투/일반 보상은 그대로. |
| FUSION | RelicInventory는 독립 사본/장착 권한, remove와 EconomyGrantState의 일곱 자산 root가 있다. allow_duplicates는 장착 제한이며 획득 후보 제외 조건이 아니다. | 서로 다른 미장착 사본 검증, 입력 종류 제외 후 양수확률 모든등급 검증, 선택 공개와 확정 분리. 입력 정규화/seed로 재추첨 방지. 두 제거+공통grant+노드 완료 원자 처리. |
| AUTHOR | FacilityActivityDefinition/FacilityLayout은 typed loader와 clone_shape, live_ron cross-reference 검증을 거친다. RelicDefinition.rarity는 현재 임의 유효ID이다. | 활동 후보에는 common/rare/special 3등급만 지원하고 참조/중복/양수가중치/합계/가루증가를 검증. 다른 유물의 rarity 계약을 전역 변경하지 않는다. 입력 사본 미지원등급은 이유로 거절. 공식 catalog 공백에서 효과를 창작하지 않고 외부pack 검증. |
| STORE | node_flow·active_run/continuation이 미완료 진입 전 복구/완료 보존을 소유하며 application과 host paired publish가 실패 rollback을 담당한다. | 새 session의 후보/누적투자/선택은 같은프로세스 유지. Continue의 checkpoint에 투자/사본이 같이 들어가는지 확인. 완료 명령은 즉시 완료 경계. wire 추가는 optional/default로 기존 세션 호환 유지. |
| PUBLIC | PlayerBehavior/ActionKind·command metadata·application owner·capability·SelectedEventSnapshotDto와 host 명령 루트가 있다. | 투자/합성선택/합성확정 command와 자격/실패이유, 현재 효과/다음차액/선택확률 DTO를 동일 consumer에 연결. 기존 reward 선택/수령/퇴장을 재사용. |

내부 구현 선택: 후보 가중 비복원 추출, 노드 seed와 단계/정규화 두 copy UUID를 사용하는 결정적 추출. 조회는 RNG를 소비하지 않는다. 임계치는 누적값이며 지급액은 차액이다. 후보 pool의 유효한 참조만 사용하고 미래 후보는 DTO에 노출하지 않는다. 정확한 필드/함수명은 GEX에서 결정한다.

최소 동시 전환은 작성/실행/세션/command/query/reader-writer이다. 세 활동만 먼저 구현하고 실제 consumer나 복원을 다음 Phase로 미루지 않는다. 새 자산엔진이나 범용 리팩토링은 필요하지 않다. 기존 공통grant와 transaction 경계를 재사용한다.

검증 준비: 정상 RON pack으로 세 종류가 예약된 유한 경로, Event/상점의 지원 지급으로 가루와 두 입력 사본 마련, 첫 입장 비용/활동상태 관측, 단계별 부족/실패/재시도와 선택 교체, 완료/미완료 새프로세스 Continue, SavePoint, 실제OS 저장실패를 비교한다. 정상/확장 후보 규모에서 시간·heap/RSS·회수와 내부 작업량을 구분한다. helper-only 단위시험과 정상 host 증거를 구별한다. 정확한 fixture와 행동 순서는 GEX PLAN에서 고정한다.
