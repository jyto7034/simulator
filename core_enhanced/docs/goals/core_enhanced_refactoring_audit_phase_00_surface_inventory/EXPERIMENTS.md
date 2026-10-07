# Core Enhanced Refactoring Audit Phase 00 — Experiments

Status: complete

## 2026-08-07 — Phase Goal 초기화

### 검증하려던 가설

제안된 Phase 0 이름이 기존 경로와 충돌하지 않고, Parent Master의 ready dependency와 current dirty
baseline을 확인한 뒤 production 조사 전에 investigation-only 필수 문서 3개를 만들 수 있다.

### 수행한 실험 또는 검증

- Parent Master 필수 문서 3개 전체와 Phase 0 registry/dependency 확인
- current `goal_authoring_and_execution_guide.md`, `goal_completion_review_guide.md`,
  `engineering_workflow.md`, `docs/README.md` 확인
- Phase 0 경로 충돌과 관련 repository dirty 상태 확인

Repository-wide runtime search, candidate 조사, test, formatter와 production mutation은 수행하지 않았다.

### 변경한 내용

- `core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_00_surface_inventory/` 아래 필수 문서
  3개를 생성했다.
- Parent Master의 Phase 0 registry와 activation 기록은 Goal 문서 범위에서 갱신한다.
- Production code, live data, test와 canonical은 수정하지 않았다.

### 실행한 정확한 명령

```bash
rg --files -g 'PLAN.md' -g 'AGENTS.md' -g 'pasted-text-1.txt' . /home/acr0tic/.codex/attachments/a058dbfd-e530-45b8-b81b-66e47b3138fe
sed -n '1,260p' /home/acr0tic/.codex/attachments/a058dbfd-e530-45b8-b81b-66e47b3138fe/pasted-text-1.txt
sed -n '1,320p' core_enhanced/docs/goals/core_enhanced_refactoring_audit_master/PLAN.md
sed -n '1,360p' core_enhanced/docs/goals/core_enhanced_refactoring_audit_master/EXPERIMENTS.md
sed -n '1,420p' core_enhanced/docs/goals/core_enhanced_refactoring_audit_master/EXPERIMENT_NOTES.md
sed -n '1,920p' core_enhanced/docs/goal_authoring_and_execution_guide.md
sed -n '1,620p' core_enhanced/docs/goal_completion_review_guide.md
sed -n '1,430p' core_enhanced/docs/engineering_workflow.md
sed -n '1,360p' core_enhanced/docs/README.md
git status --short -- core_enhanced core_enhanced/docs/goals/core_enhanced_refactoring_audit_master game_resources game_server
```

### 실제 실행된 test 수

Test runner를 실행하지 않았다. 해당 없음.

### Pass / Fail / Skip

Test 기준 해당 없음. 필수 문서, dependency, 경로 충돌과 dirty 상태 확인은 성공했다.

### 실제 결과

- Parent Master는 `active`, Phase 0은 `ready`, direct predecessor는 없음으로 확인됐다.
- 제안 경로는 존재하지 않았고 필수 문서 3개도 없었다.
- 시작 commit은 Parent Master가 기록한 `f318414fb0156a67e04c5479000c2a2683fc0c09`다.
- Core code/test/canonical, supporting live data/server와 기존 Goal 산출물이 모두 dirty한 baseline이다.
- 사용자의 2026-08-07 Master 수행 요청을 제안된 Phase 0 이름과 범위의 activation 승인으로 적용했다.

### 실패 원인

첫 위치 확인에서 사용자가 축약해 지정한 `core_enhanced_refactoring_audit_master/PLAN.md`가 repository
root 바로 아래에는 없어 명령이 중단됐다. `rg --files`로 실제 경로
`core_enhanced/docs/goals/core_enhanced_refactoring_audit_master/PLAN.md`를 확인한 뒤 필수 문서를 읽었다.

### 되돌리거나 수정한 내용

없음. 기존 dirty/untracked 항목을 수정, 이동, 삭제하거나 정리하지 않았다.

### 결과에 따른 판단

- Phase 0은 investigation-only surface/authority inventory로 활성화할 수 있다.
- 이후 runtime/data/test search는 이 Phase 문서의 범위와 evidence 규칙 아래에서만 수행한다.
- 후속 Phase는 Phase 0 완료 감사와 사용자 분할·이름 승인 전에는 만들지 않는다.

## 아직 수행하지 않은 실험과 검증

- source/module/test/doc/manifest inventory
- runtime authority와 live-data/DTO/server seam mapping
- surface disposition과 후속 boundary Phase 제안
- Phase completion audit와 dirty preservation 비교

## 2026-08-07 — Full surface, authority, data, test and consumer inventory

### 검증하려던 가설

현재 `core_enhanced`의 모든 non-generated file surface를 수량으로 reconciliation하고, 실제 module
entry/state owner/loader/server seam을 따라 독립 source-of-truth boundary를 제안할 수 있다.

### 수행한 실험 또는 검증

- visible/hidden non-target file inventory와 source/test/doc/manifest/tooling 분류
- crate/game/world/battle/data/map/resource module export와 authoritative type 조사
- `PlayerBehavior`→`GameCore`→state mutation→`BehaviorResult`→server mapping 경로 조사
- embedded production live RON source set과 `game_resources/data/**` exact file match 확인
- Core Rust test target/module과 test attribute 수량 inventory
- canonical/governance/history/tmp/Goal artifact ownership 분류
- server path dependency, startup loader와 direct Core consumer file 확인
- unreferenced처럼 보이는 config/test/fixture/tooling signal의 workspace reference search

### 변경한 내용

- `SURFACE_INVENTORY.md`에 전체 surface/authority/test/consumer inventory를 기록했다.
- `BOUNDARY_PROPOSAL.md`에 source-of-truth 기반 후속 Phase 1~10과 Final 분할안을 기록했다.
- Production code, test, live data, canonical, server와 tooling은 수정하지 않았다.

### 실행한 정확한 명령

핵심 재현 명령은 다음과 같다. 긴 source read는 각 listed file에 `sed -n`과 `rg -n`을 적용했다.

```bash
rg --files core_enhanced
rg --files --hidden core_enhanced -g '!target/**' -g '!.git/**'
comm -13 <(rg --files core_enhanced | sort) <(rg --files --hidden core_enhanced -g '!target/**' -g '!.git/**' | sort)
rg --files core_enhanced/src/game | awk -F/ '{if (NF==4) key=$4; else key=$4 "/" $5; count[key]++} END {for (k in count) print count[k], k}' | sort -k2
rg -n '^(pub\s+)?mod\s+|^pub\s+use\s+' core_enhanced/src --glob '*.rs'
rg -n '^pub (struct|enum|type|trait) |^impl (World|GameCore|GameDataBase|GameDataBuilder|BattleCore|BattleExecutionState)' core_enhanced/src/game --glob '*.rs'
rg --files core_enhanced/tests core_enhanced/src | rg '(^core_enhanced/tests/|/tests?(/|\.rs$)|test)'
rg -n '^\s*#\[(tokio::)?test\]|^\s*#\[rstest|^\s*#\[test_case' core_enhanced/src core_enhanced/tests --glob '*.rs'
rg -n 'include_(str|bytes)!|game_resources|load_live|live_ron' core_enhanced/src core_enhanced/Cargo.toml --glob '*.rs' --glob '*.toml'
rg --files game_resources/data
rg -o 'game_resources/data/[^"\)]+' core_enhanced/src/game/data/live_ron.rs core_enhanced/src/game/data/mod.rs
rg -l 'game_core::|use game_core' game_server/src game_server/examples --glob '*.rs'
rg -n 'load_live_embedded|GameCore::new|active_run|encounter_compendium' game_server/src game_server/examples --glob '*.rs'
rg --files core_enhanced/docs
rg -n '^Status:' core_enhanced/docs/goals -g 'PLAN.md'
rg --files core_enhanced/docs/goals | awk -F/ '{file=$NF; dir=$0; sub("/" file "$","",dir); if(file=="PLAN.md") p[dir]=1; else if(file=="EXPERIMENTS.md") e[dir]=1; else if(file=="EXPERIMENT_NOTES.md") n[dir]=1} END {for(d in p) if(!e[d]||!n[d]) print d}'
rg -n 'stat_pipeline|game_balance|qliphoth|test_actor|init_logging' core_enhanced/src core_enhanced/tests core_enhanced/Cargo.toml game_server/src game_server/examples --glob '*.rs' --glob '*.toml'
git status --short -- core_enhanced game_resources game_server
```

Read-only source ranges included `Cargo.toml`, `src/{lib,main}.rs`, `game/mod.rs`, `world.rs`,
`world/{state,active_run,combat,node_flow,snapshot}.rs`, `behavior.rs`, `data/{mod,live_ron}.rs`,
`battle/core/{mod,sim,types,commands}.rs`, `battle/{event_log,scenario,types}.rs`, map/resource/domain module
declarations, server startup/actor messages/state/handlers, canonical preambles and all hidden tooling files.

### 실제 실행된 test 수

Test runner는 실행하지 않았다. Static attribute inventory는 source 862개, integration 48개, 합계
910개였지만 이는 실행 test 수가 아니다.

### Pass / Fail / Skip

Test 기준 해당 없음. File-set, module-group, live-source와 consumer inventory 명령은 모두 성공했다.

### 실제 결과

- Non-target files는 visible 873개 + hidden tooling 13개 = 886개다.
- Visible files는 source 176, tests 18, docs 676, manifest/config/tmp 각 1개로 정확히 합계가 맞는다.
- 176 source는 crate/bin/game/world/data/battle/map/resource/preview/setup/domain group으로 모두
  reconciliation됐다.
- 18 integration files는 5 target root와 13 support/module file이다.
- Production live data 23개는 Core가 참조하는 manifest 1 + gameplay source 22개와 exact match다.
- `GameCore`가 command entry/host state, `GameCoreState`가 run/active-battle host,
  `BattleCore`/`BattleExecutionState`가 battle runtime, `GameDataBase::load_live_embedded()`가 production
  data publication authority다.
- Server는 path dependency로 Core를 직접 소비하고 14개 source/example file에서 Core type을 사용한다.
- Unity는 README가 존재를 명시하지만 승인된 비범위라 unverified다.
- Docs는 Goal artifact 647개와 non-Goal 29개다. Goal history는 current runtime source가 아니다.
- Data canonical의 non-manifest source count 21은 actual/code/test constant 22와 불일치한다.
- Config/test actor/fixture binaries/main/logger는 workspace search만으로 consumer를 확정할 수 없어
  named Phase의 unverified signal로 남겼다.

### 실패 원인

대량 병렬 출력 일부가 tool output limit에서 잘려, 필요한 source/guide 범위를 더 작은 명령으로
재실행했다. 잘린 출력은 최종 evidence로 사용하지 않고 후속 targeted 결과를 사용했다.

### 되돌리거나 수정한 내용

없음. Goal artifact 외 파일은 수정하지 않았다.

### 결과에 따른 판단

- 하나의 project-wide candidate Phase는 authority와 lifecycle을 혼합하므로 부적합하다.
- 10개 sequential boundary Phase와 Final consolidation이 전체 surface를 누락 없이 다루는 최소
  실행 구조다.
- Phase 10 transport/projection은 domain producer audit 이후에 두어 DTO가 Core policy를 역정의하지
  않게 한다.
- 후속 Phase directory는 사용자 승인 전 만들지 않는다.

## 2026-08-07 — Phase completion audit

### 검증하려던 가설

Inventory가 모든 approved surface를 disposition하고, 수량 재계산과 initial/final dirty comparison에서
누락 또는 production mutation 없이 `Complete / High`를 받을 수 있다.

### 수행한 실험 또는 검증

- visible/hidden/current docs와 source group 독립 재계산
- test file/attribute 수량 재계산
- live production file/Core include source exact sorted-set comparison
- hidden tooling/direct server consumer/Phase proposal row 재계산
- later child directory 부재 확인
- initial/pre-audit production status set과 path count 비교
- completion condition P0-01~P0-10 evidence matrix 작성

### 변경한 내용

- `COMPLETION_AUDIT.md` 작성
- Phase Status를 `complete`로 전환하고 evidence matrix를 pass로 갱신
- Parent Master Phase 0 registry를 `complete`로 전환하고 후속 제안을 `deferred_by_user`로 기록

### 실행한 정확한 명령

```bash
rg --files core_enhanced | wc -l
rg --files --hidden core_enhanced -g '!target/**' -g '!.git/**' | wc -l
rg --files core_enhanced/src | awk -F/ '<documented disjoint source grouping and total>'
rg --files core_enhanced/tests | wc -l
rg -n '^\s*#\[(tokio::)?test\]|^\s*#\[rstest|^\s*#\[test_case' core_enhanced/src core_enhanced/tests --glob '*.rs'
comm -3 <(rg --files game_resources/data | sort) <(rg -o 'game_resources/data/[^"\)]+' core_enhanced/src/game/data/live_ron.rs core_enhanced/src/game/data/mod.rs | sed 's#^[^:]*:##' | sort -u)
comm -13 <(rg --files core_enhanced | sort) <(rg --files --hidden core_enhanced -g '!target/**' -g '!.git/**' | sort)
rg -l 'game_core::|use game_core' game_server/src game_server/examples --glob '*.rs'
rg -n '^\| (1|2|3|4|5|6|7|8|9|10|Final) \|' BOUNDARY_PROPOSAL.md
git status --short -- core_enhanced/src core_enhanced/tests core_enhanced/docs core_enhanced/Cargo.toml core_enhanced/config core_enhanced/tmp game_resources/data game_server/src game_server/examples
```

### 실제 실행된 test 수

Test runner를 실행하지 않았다. 해당 없음. Static attribute inventory 910은 실행 수가 아니다.

### Pass / Fail / Skip

Test 기준 해당 없음. Completion evidence matrix 10/10 pass다.

### 실제 결과

- Pre-audit current set은 visible 875, hidden-included 888이다. Initial 873/886에서 Phase artifacts 2개만
  늘었다.
- Source 176, integration files 18, static test attributes 910이 재현됐다.
- Live source 두 set은 23/23이고 symmetric diff가 없었다.
- Hidden-only 13, server consumer 14, proposal Phase/Final row 11이 재현됐다.
- Later child directory는 없고 Phase 0만 존재한다.
- Production status path와 category count는 initial snapshot과 동일했다.
- Audit primary verdict는 `Complete`, fit은 `High`다.

### 실패 원인

없음.

### 되돌리거나 수정한 내용

없음. Goal artifact 외 파일은 수정하지 않았다.

### 결과에 따른 판단

Phase 0은 `Complete / High`다. 다음 Phase는 사용자 approval gate 때문에 활성화하지 않는다.
