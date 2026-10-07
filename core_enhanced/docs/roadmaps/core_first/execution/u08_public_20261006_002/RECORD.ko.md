# U08 A 집중 검증 회수와 B/C 작성 재개 — 2026-10-06/002

GEX-10 계속. A 공개 조회 국소 경계가 통과했으며 전체 U08 완료 또는 독립 완료가 아니다.
검증 입력 경계는 product417 / 6076ed935e7e99fe26db51bb2bcbc883bdfe6ee63832ef118473d95d41827af1이다.
이 경계에서 기능 runner 종료와 전 writer HOLD를 확인한 뒤 B/C 제품·별도 시험 파일 작성을 해제했다. 이후 소스는 작성 중이므로 이 해시를 실시간 현재 소스로 주장하지 않는다.
root Astra/high 계획·조정, Sol/xhigh 제품/포맷/실행, 별도 Astra/high 시험 작성·oracle 역할을 유지한다. 최종 감사자는 아직 배정하지 않았다.

## 실제 결과와 원본 보존

- A001 compile101/test0: 제품 연결 오류3종8진단. 출력 DTO Deserialize 요구, validated identity 전달, 배열 길이를 Sol이 교정했다.
- A002 compile 성공,7개 중1PASS/6fixtureFAIL: 시작 fragment와 새 weapon 호환성 거절로 query 이전에 실패했다.
- Astra는 A002의 원본을 보존하고6개 setup의 fragment/weapon 장착 순서만 바꿨다. 수치/selector/provider/penalty/불변 기대값과 fixture catalog는 유지했다.
- A003 공개 신규 RON pack→CoreApplication 7/7PASS. 두 직원, 실제 provider/form, 같은 효과 AND, split tags, Heal 제외, 미해결 Live 문맥, final overflow, canonical순서, penalty/certified, 무기 없음, 조회 root 불변을 확인했다.
- 이후 behavior.rs wholefile를 Sol HOLD→Astra에 넘겨 cfg(test) helper/fixture/판정표만 갱신했다. 기존 U08 누락을 포함해 action74/result54를 확인했다. root가 A003 archive 대비 전체 diff와 cfg(test) 경계를 읽어 제품 실행 부분 비영향을 확인했으므로 A003을 재사용한다.
- 같은 최종 경계에서 numeric runtime-observation33/33, default31/31, behavior계약11/11 모두 PASS. exact receipt/raw/hash는 EVIDENCE.json에 연결한다.
- RuntimeFactSource136/EvalCapture192/cold24는 유지됐다. Core runtime_observation SessionState384는 이전 동일 시험과 동일하고, host SessionSnapshot352와는 다른 타입이다.

초기 noRelic PC의 source409/6ea1… 및576행/36조건/31성공5기존거절/U07-M 비시간delta0/17반례 거절은 해당 조기 경계에만 유효하다. 새 A 소스의 최종 PC 성공으로 재명명하지 않는다.
기능/성능 실행은 겹치지 않았으며 이번 순차 실행에는 성능 측정이 없다. owned session0 회신과 실제 receipt를 확인했다. process 조회는 sandbox namespace 한계가 있다.

## B/C 작성과 남은 검증

기존 공개 계약은 ../u08_public_20261006_001/RECORD.ko.md 및 API 부록001→002→003 순서로 읽는다.
B 후보는 확정 eligible-only: 공통 validate_corruption_target가 통과한 Normal+작성된 corruption variant 사본만 copyUUID 정렬한다. effects/penalties는 실제 corrupt 이후 값이다. 후보0에도 can_decline=true/catalyst_cost=0. 미지원/이미 Corrupt copy에 가짜 after 값을 만들지 않는다.
선택 이력의 다중 장면·decline·same historical retry/current snapshot·strict restore path·정상 dismantle 후 absent historical copy 허용·old None canonical을 유지한다.
새 Choose 경로뿐 아니라 기존 Advance/Resolve/State projection의 EventSession clone도 receipt Vec를 새로 소유하므로 pre-admission/CloneShape/retained를 연결한다. active/run/checkpoint/entry/Continue의 각 실제 독립 owner와 decode 전 hint도 포함한다.
C는 정상 shop qty1의 명시 RelicNotSellable와 derived sorted not_sellable_copy_uuids. persistent OwnedRelic 권한은 추가하지 않는다.

Sol이 제품 파일을, Astra가 별도 integration/fixture/world test 파일을 작성한다. behavior.rs/application.rs/host state.rs 등 mixed 파일의 cfgtest 변경은 제품 ready 뒤 wholefile를 순차 인계한다. standalone world/tests/snapshots_and_start.rs의 기존 literal 보강은 Astra 소유다. 포맷/빌드/실행은 양측 ready/HOLD 이후 별도 합의한다.

새 query native exact/one-short/clone/work/retained 오류·게임 root rollback과 실제 battle coupling은 A7만으로 닫히지 않았다. B/C와 함께 후속 시험을 준비한다.
정상 grant는 외부 RewardDatabase→ClaimReward로 직접 AdminGrant와 구분하며, E에서는 기존 공개 playback의 전투→맵 Reward 선택→confirm→claim 경로를 새 authored slot 입력과 연결한다. setup용 admin 진입이 있는 시험을 전체 비관리자 게임 경로 성공으로 재명명하지 않는다.
D의 기존 death reconcile/reset 제품 경로는 존재하며 정상 postbattle/withdraw/living redeploy/reset/Continue 검증이 남는다. 이후 E newRON/CLI/trial/queued/projectile/area, 전체 영향 검사, R05 도구/실행, current PC·impact/adaptation/coverage·exact freeze·독립감사를 계속한다. U09로 아직 진행하지 않는다.
