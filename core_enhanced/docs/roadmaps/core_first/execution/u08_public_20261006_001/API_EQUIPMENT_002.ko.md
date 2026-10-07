# A prospective equipment 타입 구체화 — 부록002

실제 EquipmentSourceSnapshot.runtime_item_instance_id는 전투 생성 전 존재하지 않는다. 가짜 UUID 없이 readonly와 prospective 의미를 보존하기 위해 신규 RelicProspectiveEquipmentDto{definition_uuid,persistent_copy_uuid}를 사용한다.
SkillDamage provider.equipment:Option<RelicProspectiveEquipmentDto>, BasicAttackDamage.weapon:RelicProspectiveEquipmentDto가 부록001의 EquipmentSourceSnapshot 참조를 대체한다.
shared selector는 실제 equipment definition ID의 borrowed context를 받고 runtime adapter만 기존 실행 snapshot을 해석한다. FragmentSourceSnapshot은 persistent copy/selected form의 실제 정보로 사용한다.
이는 수용 조건 변경이 아니라 실제 타입 수명에 맞춘 공개 DTO 구체화다. root 승인 후 Sol과 Astra 양측에 같은 변경을 전달했다. 실행 HOLD 유지.
