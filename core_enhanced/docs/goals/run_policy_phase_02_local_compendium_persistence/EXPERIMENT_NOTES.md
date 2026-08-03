# Local Encounter Compendium Persistence Experiment Notes

Status: complete notes

Implementation complete: true

Created: 2026-07-27

이 문서는 P02 조사, 판단, 실험, persistence 위험과 범위 밖 후보를 기록한다.
Durable gameplay/save/wire source of truth가 아니다.

## 1. Dependency

- Master: `run_persistence_facility_monolith_policy_master`
- Phase: P02
- 직접 선행: P01 completion review `Complete / High`
- 후속 consumer: P03~P05, 특히 P05 monolith/hidden-boss producer

## 2. 승인 경계

- profile은 local-install singleton이며 transport player UUID별 map이 아니다.
- discovered/defeated는 run 밖에서 단조 union되고 `defeated ⊆ discovered`다.
- dormant ID와 current catalog 신규 commit validation은 다른 검증 단계다.
- active-run과 profile은 별도 file/lock/transaction/failure surface다.
- Unity는 Core authored projection/status를 표시할 뿐 battle log/catalog에서 set을 재구성하지 않는다.

## 3. Current/Target Gap

- current profile field `abnormality_ids`/`corrosion_profile_ids`는 discovered만 표현한다.
- current `validate`는 dormant ID까지 missing catalog 오류로 만들어 target과 충돌한다.
- current snapshot은 discovered 항목만 BTreeSet 순서로 보내며 미발견 authored slot을 보내지 않는다.
- current server `encounter_compendium_profiles: HashMap<Uuid, ...>`는 local-install scope와 충돌하고
  process restart에 소실된다.
- current Unity model에는 discovered/defeated flag, placeholder와 persistence status가 없다.
- 실제 spawn discovery seam은 재사용할 수 있지만 victory primary와 non-primary death extractor가 필요하다.

## 4. Dirty Worktree

Core/server/Unity 모두 P00/P01과 독립 사용자 변경이 누적된 dirty 상태다. P02는 current 파일 위에
국소 patch하고 전체 reset/stash/restore를 하지 않는다. `cargo fmt --all` 같은 repository-wide
mechanical rewrite는 추가로 실행하지 않는다.

## 5. 정책 질문

현재 없음. Current catalog 배열 순서를 P02 authored order로 사용할 수 없거나 reset/fatal UX가 승인
정책만으로 결정되지 않을 때만 중단한다.

## 6. 범위 밖 후보

- P01 최종 fixture 감사 중 Rust generator와 Unity fixture 사이의 기존 `event_aptitude` metadata 차이를
  발견했다. P01 Treatment legacy field만 수렴시켰고 이 독립 employee fixture drift는 P02 범위에
  포함하지 않는다.

## 7. 구현 판단

- battle event fixture가 primary spawn을 생략하는 기존 경로도 있으므로 승리한 primary는 authoritative
  battle result의 primary identity로 발견과 격파를 함께 commit한다. Non-primary는 실제 spawn과
  `UnitDied` identity가 모두 확인된 경우에만 격파한다.
- dormant 보존과 신규 catalog validation을 단일 `validate`로 합치지 않았다. Persisted payload는 ID
  형식과 subset을, gameplay commit은 현재 catalog membership을 검증한다.
- profile persistence 상태는 Core gameplay state가 아니라 host lifecycle이다. Server가 exact root
  status를 덧붙이고 Unity는 이를 재해석하지 않는다.
- reset은 profile만 비우며 active run은 유지한다. Unity에서 명시적 확인 없이는 canonical reset
  command를 보내지 않는다.
- recovery notification은 singleton store에서 한 번만 소비해 transport reconnect마다 반복하지 않는다.

## 8. Transaction·rollback 감사

- active-run export에는 profile copy가 없다.
- active-run import/rollback은 현재 local profile을 보존하고 stale snapshot으로 덮지 않는다.
- profile publish 실패 뒤 gameplay와 active-run file은 유지되며 profile은 최신 전체 union을 memory에
  dirty로 보존한다.
- active-run publish 실패도 이미 commit된 profile publish를 되돌리지 않는다.
- 두 store는 filesystem replace/sync primitive만 공유하고 canonical file, backup, lock, envelope,
  bootstrap state와 retry queue를 공유하지 않는다.

## 9. 플랫폼과 잔여 위험

- Linux/WSL 실제 filesystem에서 current/backup/lock/failure test를 실행했다. Windows host replace는
  `MoveFileExW(REPLACE_EXISTING | WRITE_THROUGH)`, directory sync는 backup-semantics handle을 사용하도록
  구현했다.
- target OS 전원 차단을 실제로 유발하는 crash test와 Windows Rust cross-target compile은 이번
  환경에서 실행하지 않았다. Strict envelope 검증, fsync ordering, 실제 failure tests와 Windows API
  경계 검토상 P02 완료를 막는 material gap으로 판정하지 않는다.
- P05는 새 remanifestation/hidden-boss identity만 기존 generic commit seam에 연결해야 하며 profile
  저장 의미나 primary/non-primary 규칙을 다시 정의하지 않는다.

## 10. 최종 판단

- Completion review: `Complete / High`
- 정책 질문: 없음
- 즉시 correction 후보: 없음
- 범위 밖 `event_aptitude` fixture drift는 그대로 보존했다.
