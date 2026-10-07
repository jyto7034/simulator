# S1 F02 교정·자체 검증 결과

S1 감사002는 F01을 해소된 것으로 확인했으나 F02로 Not Complete / High confidence를 판정했다. 기존 SavePoint의 peak204를 장비 강화 후 전투 진입이213으로 갱신한다. 기존 host는 전투를 commit한 뒤 저장에 실패했고, 올바른 F01 active-battle 복원 거부 때문에 진행 중 전투가 남은 채 명령 오류를 반환했다. F01의 최초 무checkpoint 재현만으로 모든 entry의 checkpoint 불변을 주장한 기존 교정 결과는 불충분했다. 원 감사·이전 동결 target·실패 증거를 보존한다.

동결 TD03이 허용한 equivalent prepared application boundary로 수정했다. private GameCore의 기존 bounded draft, validation, resource reconciliation, first-fault poll을 그대로 거쳐 준비하고 commit을 분리한다. 전체 root를 소유한 private 준비 객체는 추가 clone을 만들지 않는다. 기존 실행 경로도 이 준비와 sole live-root commit을 사용한다. 공개 `prepare_non_battle_command`는 active battle을 거부하고, 결과와 checkpoint를 준비한 뒤 originating application을 독점적으로 빌리는 opaque 객체를 반환한다. 객체 폐기는 staged state를 버리고 consuming commit은 gameplay를 재실행하지 않고 준비한 상태를 반영한다. 기존 revision은 준비 시도당 한 번 증가하여 token의 next-attempt 규칙을 유지한다.

실제 host는 battle 밖에서 시작하며 active-run store가 있는 명령을 준비한다. 준비 결과가 battle에 진입하고 checkpoint가 바뀌면 store.publish 성공 뒤 commit한다. 실패하면 원래 gameplay root를 유지하여 재시도할 수 있다. 성공 후 entry 저장을 중복 실행하지 않는다. 기존 non-battle 저장 rollback, profile의 독립 영속화와 monotonic 이미-commit된 state, active battle의 기존 실행 경로 및 F01 복원 거부를 유지한다. 정책 수치·저장 형식·U03 fault-recovery 의미를 새로 정하지 않았다.

실제 release production actor와 외부 R06 seed77 content를 사용한 감사 재현을 강화했다. 기존 checkpoint peak204에서 장비 강화·entry를 준비하고 active_run.tmp 디렉터리로 저장 실패를 주입했다. `active_run_persistence_failed`, battle side messages0, 원래 run snapshot 및 저장 checkpoint의 exact equality를 확인했다. 주입을 제거한 재시도는 성공했고 battle side messages와 durable peak213을 확인했다. 무checkpoint entry의 root 폐기/재시도/직접 실행 parity, 작은 native work cap, 이미 발생한 authoring first-fault/cancellation도 현재 테스트에서 검증했다.

공개 release compile probe8 cases는 valid getters/commit과 private field·Clone·Serialize·Deserialize·중복 commit·명령 interleaving·foreign application 이전 거부를 확인한다. 기존 public boundary13 cases, 실제 profile CLI24 commands/host5 configurations, API/CLI parity21 commands, predecessor 공개 oracle5/trace5와9 packs45 CLI commands도 현재 source에서 다시 실행했다. Core 두 suite와 host의 실제 합계는 `{"s1f02_all_default_001": [1701, 0, 0], "s1f02_all_feature_001": [1701, 0, 0], "s1f02_server_002": [77, 0, 0]}`이며 failed/ignored는0이다. check/event_log_checks/Clippy/fmt/release도 통과했고 Clippy 경고는 원본 그대로 남긴다.

PC36는576 raw rows/36조건의 gameplay·typed refusal을 비교했고 직전 S1과 elapsed_ns만 제외한 모든 fields 및 allocation/native usage가 같았다. R05 raw exit는101이며 분류는 `warning_concurrent_peak_only`다. concurrent peak는9335309 bytes, 기존 cap은9030188 bytes, 초과량은305121 bytes다. 다른 hard guard failures는[]다. 기존 concurrent-peak-only 예외와 raw 결과를 분리하며 cap·latency policy를 변경하지 않았다.

초기 check001은 host 코드의 중복 닫기 구분자로 compile 실패했다. 수정 후 focused/server 및 최종 matrix가 통과했다. PC immediate 비교001은 setup이 없는 row에서도 setup 필드를 요구한 검증 스크립트 오류로 중단됐다. 해당 필드가 있는 경우에만 elapsed_ns를 제외하도록 수정한002를 사용했고 raw 측정은 변경하지 않았다. 모든 실패 receipt도 보존한다. 이번에는 공통 application 경계 변경에 맞춰 suite/public/cost 증거를 현재 소스로 재실행했으며 과거 실행 성공을 current 성공으로 대체하지 않았다. COVERAGE.json은9 validation rows/7 acceptance/10 required dimensions와88 inherited conditions를 현재 selectors/receipts에 연결한다.

다음은 `target/s1_f02_001/MANIFEST.json`에 대한 fresh independent GEX-20이며 output은 ../../../../post_audit/s1_20261004_003이다. 기존 reviewer는 구현에 참여하지 않았으므로 fresh pass에 재사용한다. predecessor semantics/currentness와 exact restoration을 포함한 Complete/High 판정 전까지 S2를 시작하지 않는다. 이 결과는 자체 검증 기록이며 독립 완료 verdict가 아니다.
