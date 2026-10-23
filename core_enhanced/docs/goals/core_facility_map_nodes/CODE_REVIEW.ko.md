# U13 코드 검토와 구현 인계

입력 WR-CFMN-004 / PIS-CFMN-002. 읽기 전용 검토이며 runtime 시험은 실행하지 않았다. 사용자 답변의 단순 균등 위치 추첨·보상5종 각20%와30수용 문장을 변경하지 않는다. 정확 source와 관찰 지점은 P2G_20_REVIEW_001.json에 결속한다.

## 현재 경계와 필요한 전환

| 경계 | 현재 실제 코드 | 전환·검증 seam |
| --- | --- | --- |
| RON/등록 | live_ron GameplayMapCatalogs가 FacilityTemplateDatabase와 NodeAssignmentProfileDatabase를 검증한다. content_pack/raw는 typed record로 분해/합성하고 semantic identity에 결속한다. 현재 template은 내부 slot/edge이며 전체 시설의 room graph가 없다. | 기존 catalog에 authored 시설 layout·room 참조·표시/예약·활동 개수 profile을 추가한다. pack 분해/합성/namespace/ref검증과 embedded/정상loader가 같은 입력을 사용해야 한다. 문서 JSON은 제품 입력으로 직접 읽지 않는다. |
| 배정 | FacilityMapGenerator::generate_with_requirements는 floor schedule/pool에서 template을 선택하고 slot category를 배정한다. FixedSlotAssignment는 있지만 최신 지원총수 선추첨/유효 조합 조건부 분포와5종 유형 계약은 없다. | 고정 내부 template을 재사용하고 room별 예약·전조 호환·정예 후보를 적용한다. 균등 shuffle로 가능한 자리를 선택하며 추가 경로 유도는 없다. 지원총수는 먼저, 나머지 개수의 유효 조합은 곱가중치로 선택한다. 적격자리가 부족하면 입력을 거절한다. 같은 room/seed에서 결정론이며 표시 좌표는 seed에 쓰지 않는다. |
| 이동/깊이 | RunProgression의 Expedition floor_index는 증가만 가능하다. handle_confirmed_gate_transition은 advance_floor 후 새 map으로 RunState를 교체하고 floor-local session/cache를 초기화한다. entry_receipts도 advance_floor에서 비운다. | authored 방 선택 명령·깊이 기반 조회를 도입하고 이동은 현재 Gate 도달/실제 연결을 검사한다. 현재 방과 비현재 방문방의 map/progression을 단일 소유자로 보관하고 전역 node UUID로 session/최초입장을 유지한다. 동층 재방문은 회복/비용 없이 보존 상태를 선택한다. |
| 귀환/공개/지식 | U12 prepare_expedition_node_entry는 Gate>=3/억제력 소진 선택을 만들고 Return이 settle_expedition_terminal로 끝난다. snapshot의 map은 현재 내부 map만 제공한다. profile에는 도감/층기록/lifecycle이 있고 room 영구 지식은 없다. | 전체 지도와 내부 지도 DTO를 분리하고 조회에서 정보 공개를 적용한다. Gate 떠남·Gate Return만 room 완료를 확정하며 소진 귀환은 제외한다. 영구 room/final 지식은 profile의 monotonic merge에만 넣고 가변 배정을 넣지 않는다. SavePoint/실패가 지식을 되감지 않아야 한다. |
| 지원 상태 | 현재 node payload는 실제 조우/event/reward 참조다. 미구현 최종방을 표현하는 명시적 whole-room 지원 계약은 없다. 기존 forced boss/omen map 교체 경로는 고정 graph 전조와 의미가 다르다. | authored 지원 상태/불가 사유와 실제 참조를 분리한다. 고정 graph에서는 기존 강제 boss map 교체를 호출하지 않는다. U20이 아직 구현하지 않은 자격/전조 producer를 성공 처리하지 않는다. 공개 조건/영구 지식으로 보이는 미구현 방도 명령에서 입장 거절한다. 지원하지 않는 활동/고정 콘텐츠는 구체 사유로 거절하고 정상 일반전·지원·선택보상 경로를 실행한다. |
| 저장/원자성 | RunCheckpointPayload macro가 run 필드를 capture/restore하고 Continue는 current와 nested SavePoint를 보관한다. node_flow/continuation validator는 여러 node 참조를 현재map에서만 찾는다. profile_store V4는 typed unsigned payload checksum을 비교한다. application draft와 paired profile/active publication이 기존 원자 경계다. | room별 보관 상태를 같은 checkpoint/Continue에 포함하고 실제 session/receipt/attempt 참조를 현재 또는 보관방에서 찾는다. 새 필드는 clone-shape 계량에도 포함한다. 호환 가능한 old writer 저장은 빈 신규 지식/구지도 profile로 보존하고 current identity가 다른 저장은 기존 원본보호 경로를 따른다. 새필드가 checksum을 바꾸는 구형 reader 경계는 version/명시적 converter로 처리한다. 임의변조 방어를 새로 추가하지 않는다. |
| 브리핑/도감 | combat preview는 실제 node/floor/difficulty/seed로 생성하며 실제 전투 시작 때 재생성/검증한다. map view는 threat/readiness를 제공하지만 U13의 비용/유형/표시계약은 미완성이다. 승리만 validated_encounter_registrations를 거쳐 profile에 merge한다. | 현재 실제 전장 preview를 재사용하고 node 첫입장 비용/소진·유예 위험/활동유형·지원 사유를 공개 query에 추가한다. 미공개 wave는 public projection에서 차단한다. 도감은 실제 spawn 후보와 victory merge seam을 재검증하고 미인식 ID보존은 기존 validate_persisted/merge를 보존한다. |

## 상세 설계 권고와 owner

시설 layout은 정적 catalog, 방문방 진행/배정은 run, 확정 탐사 정보는 profile이 소유한다. `RunState.map`을 현재 방의 유일한 가변 map으로 유지하고 비현재 방문방만 별도 보관하면 중복된 현재 상태의 동기화 문제가 없다. session/preview/attempt는 전역 node ID로 유지하되 저장검증이 모든 방문방 identity를 소비하도록 함께 바꾼다. 구현 단계에서 동등한 단일 소유 설계로 바꿀 수 있으나 상태·비용·복원 의미는 유지해야 한다.

최신 배정89방/846활동·15개 내부동선·최종 F6 구조를 제품 RON에 작성한다. 기존 실제 일반전/event/shop/reward catalog 참조와 고유 미지원 콘텐츠를 구분한다. 전체 구조 검증과 지원 개발profile은 별도 결과로 보고하며 미지원 노드 때문에 없어지는 참조/예약을 만들지 않는다. 최종방 고유 실행과 U20 producer는 후속 owner다.

표시 자산은 안정된 ID 참조만 전달하고 실제 이미지/Unity 표현은 U26-B다. 임의 위치 기반 연결 생성·매방전조 생성·임의 추첨 재시도에 따른 지원분포 왜곡·가짜 실행기를 금지한다. U13 새 API/serde 필드명과 구체 schema version은 구현에서 결정하되 정상 작성 예제와 실패 원본 보호를 함께 검증한다.

## 최소 동시 전환과 기반 개선

새 schema와 generator만 먼저 지원 완료로 표시하면 이동/복원이 구형 floor 교체로 상태를 잃는다. layout/assignment, navigation/knowledge, persistence의 세 작업은 하나의 atomic group으로 구현한다. 공개 조회/문서와 영향 검증은 같은 single Phase의 후속 node다. 다른 Phase의 미구현 작업이 현재 지원 계약을 완성해 줄 것으로 가정하지 않는다.

필수 기반 개선은 현재방만 찾는 저장검증과 floor 교체의 run/local 수명 분리다. 방상태 lookup과 이동 owner를 명시하여 재방문·복원 오류를 없앤다. 범용 graph 프레임워크·무관한 battle/economy 재설계는 하지 않는다.

## 검증과 남은 한계

각30수용 조건에 실제 정상loader/application/host 증거 또는 정확 비영향 재사용을 배정한다. 작은 방의 수작업 조합·연결 oracle로 배정/validator를 먼저 검증하고 공식 전체 입력→공개 첫진입→활동/실제Gate→왕복/하강→Continue/SavePoint를 유한 실행한다. 지원 입력 오류·거절 후 비용/지식/진행 불변, 정상 old writer/현재writer·실제 IO 실패를 포함한다. source hash만으로 기능 PASS를 주장하지 않는다.

U12의 단독 구현 지침·normal approved host/paired store·clone 계량 계약을 유지한다. 최종89방 입력과 확대 입력에서 시간/메모리peak·종료후 회수/내부작업량을 따로 측정한다. 수치상한은 host 소유 유한 profile이며 기존 관측 없이 임의로 늘리지 않는다. 기존 U12 감사·동결은 과거 증거로 보존한다. 실제 게임 검증/측정·정식target동결/복원은 GEX-10 잔여다.
