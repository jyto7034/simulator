from record_sources import add,ROOT
import json,hashlib
G='core_enhanced/src/game/'
for p,r,a in [
('data/boss_runtime_data.rs',[(1,420)],['pub enum BossTaskData','pub enum BossTaskResultData']),
('battle/core/boss_runtime.rs',[(480,730),(1605,1990)],['fn admit_state_tasks','fn cancel_current_state','fn cancel_boss_owned_work','let mut staged = self.clone();']),
('battle/core/spawn_kernel.rs',[(1,565)],['pub fn execute_spawn_request<F>','SpawnRequestSettlement::Rejected { reason }']),
('battle/core/summon_runtime.rs',[(1,660)],['view: BossSummonTaskView','side: Side::Opponent','fn remove_summoned_unit']),
('data/summoned_unit_data.rs',[(1,290)],['pub struct SummonedUnitDefinitionData','pub actor_behavior_id: BossActorBehaviorId','definition.movement.speed_units_per_ms == 0']),
('battle/core/field_work.rs',[(1,330),(390,580)],['pub(super) fn settle_field_work_progress','FieldWorkCompletionEffect::RepairProtectedUnit']),
('battle/core/wave_runtime.rs',[(1,330)],['fn note_wave_actor_death','fn settle_wave_triggers','death_epoch_at_bind'])]:print(add(G+p,r,a))
R='core_enhanced/docs/roadmaps/content_authoring/roadmap/'
for p,n in [('GOALS.json',946),('CONTRACTS.ko.md',20),('POLICY_GATES.ko.md',53),('ACCEPTANCE.ko.md',124),('COVERAGE.ko.md',83)]:print(add(R+p,[(1,n)],[],kind='roadmap'))
# Prefer exact stable symbol anchors where the retrieved declaration's visibility was not copied.
p=ROOT/'evidence/source_ledger.json';d=json.loads(p.read_text())
for row in d['sources']:
 if row['id']=='S033':row['anchors']=[{'text':'EquipmentLoadoutTransition','sha256_utf8':hashlib.sha256(b'EquipmentLoadoutTransition').hexdigest()}]
 if row['id']=='S034':row['anchors']=[{'text':'OwnedEquipment','sha256_utf8':hashlib.sha256(b'OwnedEquipment').hexdigest()}]
p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n')
with (ROOT/'evidence/RESEARCH_LOG.ko.md').open('a') as f:f.write('''
## 조사 5 — 보스·소환·전장과 전체 작업 요구
- BossTaskData와 실제 admit_state_tasks는 RouteCombat/AcquireTarget/Move/Wait/Summon/EmitSignal을 구현하며 명시 Skill task는 이 경로에 없다. state cancel은 CancelOwnedWork→cancel_boss_owned_work로 이어진다. 최소 스킬 요청/결과/방향/cancel adapter는 기존 state machine에 추가할 수 있다는 설계 판단이며, 공유 shield/아군 설치물 전체가 선행이라는 실행 소비 근거는 없다.
- run_boss_runtime_boundary는 clone-backed microstep 제한과 commit을 이미 갖춘다. 새 ActionHandle 작업은 해당 정지·실패 경계와 stale activation 검증에 함께 연결해야 한다.
- SpawnKernel.execute_spawn_request는 점유·지형 전검사/정렬/중복 요청 settlement/생성 실패 복원을 공통 제공한다. 새로 만들 것이 아니라 재사용 대상이다.
- 상위 summon은 BossSummonTaskView·boss state active·boss_runtime_data 조회·boss route/scale·Side::Opponent·actor behavior에 결속. 정지·아군 trap의 입력만 추가해 일반화된 것으로 주장할 수 없다. summoned_unit compile은 nonzero move speed/radius와 actor_behavior를 요구한다.
- Summon Kill은 finalize_unit_death, Despawn은 withdraw 후 lifecycle=Despawned; alive counter 단일 회수와 batch clear가 별도다. 매개체 확장 시 owner/lifetime/진행 권한을 명시하고 공용 하위 kernel을 분리해 재사용한다.
- field work는 scenario object의 시간×worker 수를 누적하고 완료 effect는 현재 RepairProtectedUnit이다. 작업 신호로 보스 억제하는 확장은 별도 schema→compile→runtime 신호 소비를 요구한다.
- wave는 timestamp bucket 뒤 priority/ID 정렬로 안정화하고 death_epoch_after_bind로 ActorDefeated를 판단한다. 침식 kill은 이 bookkeeping을 유지해야 한다.
- 기존 roadmap GOALS.json 946줄 전체와 K01–10, PG00–09/U01–13, T00–29, 15축/C01–C07 표를 모두 읽었다. 기존 카드의 “not_executed”는 roadmap 작성 상태이지 현재 저장소 M0 성과를 취소하는 값이 아니다.
- 기존 CA19/CA22 본문도 상태별 저장/예산을 초기부터 동반하도록 이미 요구한다. 새 설계는 이 요구를 새 단위와 관계에 명시하는 강화이며, 기존 문서가 모두 뒤로 미뤘다고 왜곡하지 않는다.
''')
