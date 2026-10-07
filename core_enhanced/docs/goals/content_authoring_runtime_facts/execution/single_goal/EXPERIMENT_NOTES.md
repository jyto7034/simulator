# R04 current implementation notes

현재 `(GS-CARF-P2G50-002, GOAL-CARF-01)` / PHASE-CARF-01 / HOF-CARF-P2G60-002, Freeze003 / PP004. Correction002 GEX10 구현·검증 종료, source mutation 중지. 상위 AUTORUN R00–R24는 계속 미완료이며 R05 정식 활성화·구현은 아직 없다.

- Initial fresh 감사는 Not Complete F01/F02로 닫혔고 모든13개 출력·원 대상의 해시는 보존됐다. 같은 owner 교정이 pre-formula AdmittedStrike와 실제 delayed-hit fixture를 추가했다. 기존 formula/basic trigger/gates/RNG/area assertions와 원래 한도는 유지됐다.
- 새 경계206/208와 전체209의1,363개, all-target211/release212/examples213, 원래 public/kernel plain+allocator guards210, native67-search/exact/one-less와 current consumers214–231이 통과했다. 실제 자료는 수용 증거002/EXP019에 연결된다. 203은0 tests로 미검증,204는의도한 old-source red,205/207은새 fixture/oracle 오류 뒤206/208로 수정됐다. 기존 실패를 삭제하지 않았다.
- Source attribution: 교정3개 파일, INPUT008 나머지227개 unchanged; 원353-file dirty baseline 대비 전체R04는63개 changed/eight original additions. 현재 문서와 새 시험 출력은 해당 owner 실행 증거다. 사용자의 다른 dirty는 성과에 포함하지 않는다.
- 기존 R01 target005/audit005, R02 target005/audit004, R03 target003/audit003과 R04 target002/initial audit는 immutable historical. 각 대상의6개 member와 rebuild binaries가 달라 applicability stale이다. 과거 external inventories31/39/39/39 및 모든54개 감사 출력은 byte-exact로 확인했다.
- 다음: 새 stable target에서 original exact criteria 기준 fresh R01→R02→R03 restoration, 이어 새 R04 GEX20. Initial report를 성공으로 덮어쓰지 않는다. 새 reviewer의 실제 finding이면 감사 pass를 먼저 닫고 동일 owner 교정으로 반환한다. 구현 evidence는 감사 verdict가 아니다.
- Durable active BattleCore/TrialSession/ActionHandle writer가 없는 기존 범위는 source-proven N/A다. 기존 inventory/V6/live clone/result/retreat 적용성은 유지한다. Requested allocator bytes/RSS와 sampled/instantaneous high water를 구별한다. 미래 R05 ResourceStore/Flow/fragment/root/join을 이미 구현한 것으로 세지 않는다.

현재 material policy/access blocker 없음. 필수 선행/R04 감사가 아직 미복원이므로 정상 후속 완료로 진행하지 않는다. 상세 source/실험/제한은 CORRECTION002, DESIGN001, ACCEPTANCE_EVIDENCE002, PREDECESSOR_IMPACT002와 EXPERIMENTS가 소유한다.
