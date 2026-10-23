# U12 상태 소유·수명 인계

GEX00 설계 인수 표다. 현재 source 사실은 CRA-CEXL-002, target은 CODE_REVIEW T01–T07이며 아직 제품 구현 완료가 아니다. 구체 새 field/type은 product preparation/source review 때 결속한다.

| 상태 | 현재 owner/표면 | target owner와 수명 | 검증 seam |
| --- | --- | --- | --- |
| player/manager | GameCoreState.player | 전체 새 run reset·terminal 뒤 유지 | repeat/return/failure/start |
| mode/scenario seed | GameMode,RunProgression,GameCore.run_seed | Tutorial/Expedition;map/queue 재현 seed | normal command/codec/official data |
| expedition instance | 현재 없음;UUID seed/counter만 있음 | profile sequence/active instance 및 run namespace;newrun 증가,same-run restore 유지;tutorial 영구 비영향 | same seed next run/SavePoint/overflow |
| 직원 정의·성격 | employee definition/profile | authored risk/social/표시 적성,role/loadout | six-role validation/query/restore |
| 직원 개인·condition | Employee,roster,checkpoints/battle snapshot | run instance 내부;terminal 제거,새 직원 새 개인 | death/recruit/retreat/Continue |
| memory/trust | Employee psychology/default/nodeadvance/DTO | 제거;지원 저장/조회/전용 입력에 잔존 shape 없음 | nested valid-checksum reject/otherstate preserve |
| 원정 wallet/items/relic/UUID | GameCoreState 자원들과 clone/rollback | 시작 전체 reset 후예산/기본set한번;SavePoint/retreat는runlocal;terminal 차기 미계승 | duplicate start/select/restore |
| recruitment queue/offer | RecruitmentState/session | seed 고정 단일소유,priced max2 hire,declineFIFO | price/grant failure/Continue/maxroster |
| vault owned copies | 현재 없음 | profile exact lifecycle,canonical def+copy+forms;0–2loan 이동;return 이동;unloaned 실패보호 | union UUID/dormant/crosskind/history absence |
| encounter/progress knowledge | PlayerProfile OR/max | 별도 monotonic knowledge;expedition writer만 | tutorial with nonempty profile/rollback |
| Gate/exhaustion pending | Gate 자동 하강,exhaustion없음 | authored current depth/cause/entry receipt/pendingactivity 저장 | depth2/3,0transition,reentry,no reward |
| recovery restriction | CombatRecoveryRestriction allowed nodes | 실제 도움이 되는 합법 연쇄와공유 eligibility;all-dead checkpoint decision | treat+reequip vs futile;invalid hire |
| terminal/story | phase-only failure,story없음 | 한 terminal draft;never→pending→consumed exact profile;ack profile-only | duplicate return/ack/restart/NoneCAS |
| SavePoint/retreat | roster/items/UUID capture/restore | 같은 instance runlocal;latest profile 비되돌림;checkpoint횟수 비되돌림 | ack/knowledge after oldsavepoint |
| Continue/current/backup | versions11/12/profile3,legacyconverters | 동시 새 strict version;oldmode/nestedremoved explicit reject;원본 보존 | decode+checksum+preflight+converter |
| accepted profile+active | PreparedApplication,ApprovedPersistence,pairedintent | profile→active lock/CAS;exact accepted next lifecycle;accepted recovery_pending | preintent rollback,postaccept restart/stale |
| approved generation | U11 issuer/host/RequiredConfiguration | 새content/build/consumer identity 정상 재검증;active/future pin 의미 유지 | officialpack→approval→local/server/save |

파일 변경 전 preimage와 새로운 path 부재는 baseline ledger에 추가한다. test/fixture와 제품이 공유하는 파일은 전체 파일을 순차 이관한다. 독립 test author는 이 표를 oracle로 복제하지 않고 exact 정책 수용과 실제 공개 결과를 기준으로 작성한다.
