# Documentation–Code Alignment Reaudit Report

Status: complete

Audit date: 2026-07-18 KST

Audit scope: current dirty worktree

## 결론

현재 `core_enhanced`의 주요 문서는 대체로 실제 runtime/live data/server/Unity 동작과 같은
방향을 말하지만, **완전히 일치하지는 않는다**.

- 총 48개 독립 claim을 producer → data → mutation → projection → server → Unity consumer →
  test/probe 순서로 추적했다.
- 37개는 정합했다.
- 3개 claim은 문서가 현재 구현보다 오래됐고, 4개는 명시된 계약을 구현이 지키지 않으며,
  4개는 어느 쪽을 최종 정책으로 삼을지 사용자 결정이 필요하다.
- material `Insufficient evidence` claim은 남기지 않았다.
- finding은 14개다. P0는 없고, P1 1개, P2 8개, P3 5개다.

가장 중요한 문제는 Unity가 canonical battle event stream을 수신·보관하면서도
`SkillAreaDeclared` 타일 경고/효과, 철수·사망 위치, 공격·캐스트·투사체·버프 계열 표현을 실제
렌더링하지 않는 점이다. 전송 순서, revision, resync와 actor/HUD/range projection 자체는
정합했으므로, 이 문제는 transport failure가 아니라 **presentation consumer 구현 누락**이다.

## 감사한 문서와 구현 표면

Durable Core 문서:

- `docs/README.md`
- `docs/game_rulebook.md`
- `docs/skill_target_contract.md`
- `docs/core_runtime_contract.md`
- `docs/data_loading_contract.md`
- `docs/audit/live_skill_catalog_evidence_contract.md`
- `docs/engineering_workflow.md`
- `docs/probe_registry.md`

콘텐츠·결정 문서:

- `docs/skills/lobotomy_content_catalog.md`
- `docs/skills/abnormality_skill_design_notes.ko.md`
- `docs/skills/skill_fragment_system.md`
- `docs/skills/live_content_index.md`
- `docs/decisions/*.md`

External Unity canonical/consumer:

- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- `/mnt/f/unity projects/ark/Assets/Scripts/**`
- `/mnt/f/unity projects/ark/Assets/Tests/**`
- `/mnt/f/unity projects/ark/docs/probe/**`

Implementation/data/test:

- `core_enhanced/src/game/**`
- `core_enhanced/tests/**`
- `game_resources/data/**`
- `game_server/src/**`

완료 goal과 archived audit는 현재 authority로 사용하지 않고 regression checklist와 과거
disposition 근거로만 사용했다.

## Coverage 요약

| Claim verdict | 수 |
| --- | ---: |
| Aligned | 37 |
| Document drift | 3 |
| Implementation drift | 4 |
| Policy decision needed | 4 |
| Insufficient evidence | 0 |
| 합계 | 48 |

한 claim에 둘 이상의 독립 finding이 있을 수 있으므로 claim 수와 finding 수는 같지 않다.

| Finding severity | 수 |
| --- | ---: |
| P0 | 0 |
| P1 | 1 |
| P2 | 8 |
| P3 | 5 |
| 합계 | 14 |

| Finding verdict | 수 |
| --- | ---: |
| Implementation drift | 3 |
| Document drift | 4 |
| Policy decision needed | 6 |
| Aligned + Test gap | 1 |

## Finding Registry

| ID | 심각도 | 판정 | 요약 |
| --- | --- | --- | --- |
| DCA2-WIRE-001 | P1 | Implementation drift | Unity가 필수 battle presentation event 대부분을 렌더링하지 않는다. |
| DCA2-DATA-002 | P2 | Implementation drift | maximally strict 계약과 달리 live-reachable nested RON type들이 unknown field를 수용한다. |
| DCA2-WIRE-003 | P2 | Implementation drift | corroded-only setup의 actual-abnormality preload 목록에 빈 문자열이 들어간다. |
| DCA2-GAME-001 | P2 | Policy decision needed | 룰북의 weapon/armor block 조정 약속을 equipment schema/stat pipeline이 지원하지 않는다. |
| DCA2-GAME-002 | P2 | Policy decision needed | 룰북은 파편 변화를 active-skill 결과로 설명하지만 live에는 직접 BasicAttackModifier가 다수다. |
| DCA2-BATTLE-001 | P2 | Policy decision needed | Boss/Elite/특수 unit authored range 계약과 30개 live generic fallback이 충돌한다. |
| DCA2-CONTENT-001 | P2 | Policy decision needed | WhiteNight 12-mark/apostle/boss-clear 파편 정책과 one-step/research placeholder가 다르다. |
| DCA2-GAME-003 | P2 | Policy decision needed | generic Maintenance 설명과 `standard_armor` 한정 live equipment recipe 범위가 다르다. |
| DCA2-TEST-001 | P2 | Aligned + Test gap | Endless all-response-complete RunComplete branch를 직접 고정하는 focused regression이 없다. |
| DCA2-DATA-001 | P3 | Document drift | data contract는 23개 source라 하지만 loader/live test는 21개다. |
| DCA2-DOC-001 | P3 | Document drift | current runtime canonical이 historical `G15`/`G24`를 현재 도메인 용어로 쓴다. |
| DCA2-WIRE-002 | P3 | Document drift | 외부 canonical이 MapNodeCategory 3개를 누락하고 제거된 SplashClusterFirst를 유지한다. |
| DCA2-DOC-002 | P3 | Document drift | 외부 프로젝트에 서로 다른 세 canonical-looking `docs - 복사본` 파일이 남아 있다. |
| DCA2-BATTLE-002 | P3 | Policy decision needed | GrenadeLauncher는 enum-only로 남지만 현재 문서는 grenade를 future-only로 분류한다. |

## Material Finding 상세

### DCA2-WIRE-001 — Unity battle presentation consumer 누락

외부 battle transport canonical은 `events_delta`를 배치, 이동, 공격, 캐스트, 투사체,
TileArea, HP/stat/resonance, 철수/사망, buff와 battle-end 표현의 source of truth로 지정한다.
특히 `SkillAreaDeclared.shape.affected_tiles`는 Unity가 재계산하지 않고 경고/효과 타일로 표시해야
하며, `UnitWithdrawn`/`UnitDied`의 event position은 inactive transition 연출 위치다.

Core와 server는 이 정보를 생산·전달한다. 그러나 Unity production consumer 검색 결과:

- movement start/stop은 actor movement에 사용한다.
- `HpChanged`는 damage/heal/immune feedback에 사용한다.
- `UnitSpawned`는 checkpoint actor identity 검증에 사용한다.
- `UnitDeployed`는 pending deploy 확인에 사용한다.
- `SkillAreaDeclared`는 display duration만 읽어 checkpoint reconcile을 늦출 뿐,
  `affected_tiles`를 읽거나 그리지 않는다.
- 철수/사망 위치, attack/cast/projectile, buff/stat/resonance, battle-end presentation consumer는
  찾지 못했다.

전체 Unity EditMode/PlayMode suite는 통과했지만 해당 event render assertion이 없으므로 이 누락을
반증하지 않는다. 플레이어-visible telegraph와 lifecycle 표현의 광범위한 누락이므로 P1이다.

### DCA2-DATA-002 — nested RON strictness 회귀

outer database/raw type과 selected negative test는 strict하지만, live-reachable shared nested type
일부에 unknown-field rejection이 전이되지 않는다.

확인한 수용 type:

- `MovementDef`
- `ResonanceDef`
- `TileRangePattern`
- `DeliveryDef`
- `SkillEffectDef`
- `StatModifier`
- `MapNodePayload`

임시 `/tmp/dca2_schema_probe`는 실제 `AbnormalityDatabase`, `SkillDatabase`,
`EquipmentDatabase`, `MapNodeDefinitionDatabase`를 통해 추가 `legacy_*` field가 조용히 무시되는
것을 확인했다. 현재 live RON에 그런 field가 있다는 뜻은 아니지만, durable
`maximally strict` 계약과 accepted PD-09를 위반한다. 과거 DCA-H-04/G12 완료 disposition의 실제
회귀다.

### DCA2-WIRE-003 — setup catalog ref에 빈 ID

외부 contract는 `catalog_refs.abnormality_ids`에 non-empty actual abnormality catalog ID만
허용한다. CorrodedEmployee의 빈 `abnormality_id` sentinel과 `profile_id`는 금지한다.

Core setup projection은 spawn-wave entries를 올바르게 필터링하지만 `enemy_briefing`의
`abnormality_id`는 kind/empty 검사 없이 복사한다. 등록된 live smoke probe에서 corroded-only
`normal_black_box_pressure` setup이 실제로 다음을 보냈다.

```json
"catalog_refs": {
  "abnormality_ids": [""]
}
```

현재 Unity는 이 preload 목록을 생산에서 소비하지 않아 즉시 crash하지 않지만, contract meaning과
producer output이 다르고 기존 test도 “profile ID가 없음”만 확인할 뿐 “모든 ID가 non-empty하고
catalog-resolved”임을 고정하지 않는다.

### DCA2-TEST-001 — Endless completion direct regression 부재

runtime branch와 인접 테스트는 일치하지만, 모든 response가 완료된 Endless 상태에서 다음
완료가 `RunComplete`로 전환되는 정확한 branch를 직접 구성·단언하는 focused test는 없다. 현재
동작을 문서와 코드 추적으로 확인했지만 회귀 안전망은 별도 보강이 필요하다.

## Policy Decision Needed

감사 목표에서 아래 정책을 임의로 선택하지 않았다.

1. Equipment block:
   룰북의 무기/방어구 block 조정을 실제 schema/stat pipeline으로 구현할지, 현재 약속을 제거할지.
2. Fragment semantics:
   BasicAttackModifier를 정식 fragment 결과로 인정해 문서를 바꿀지, 모든 변화를 active-skill
   실행으로 통합할지.
3. Abnormality range:
   30개 placeholder의 generic range를 현재 정책으로 인정할지, G05를 재개해 entry별 값을
   승인·authoring할지.
4. WhiteNight:
   12-apostle/third-faction/boss-clear fragment 설계를 명시적 future concept로만 둘지, 별도
   runtime/data/Unity 구현 목표로 활성화할지.
5. Maintenance breadth:
   `standard_armor` 한정 live equipment recipe를 현재 범위로 명시할지, G22를 재개해 recipe와
   balance를 승인할지.
6. GrenadeLauncher:
   reserved enum으로 공식화할지, 현재 schema에서 제거할지, 실제 grenade content goal까지
   보류할지.

## Cross-Component 판정

- Core → server → Unity battle setup/update/revision/resync/reconnect/final-result ordering은
  정합했다.
- Actor identity, canonical route cells, Active checkpoint units, HUD, player-only final range
  preview와 skill catalog v2는 정합했다.
- Unity는 Core-owned targeting/range를 authored metadata에서 재계산하지 않는다.
- 반면 event transport와 presentation consumer 사이에는 DCA2-WIRE-001이 있다.
- setup preload collection의 의미와 Core producer 사이에는 DCA2-WIRE-003이 있다.
- live bundle은 현재 semantic validation을 통과하지만 strict authoring guarantee와 nested serde
  surface 사이에는 DCA2-DATA-002가 있다.
- current canonical과 인접 stale copy/removed enum vocabulary 사이에는 DCA2-WIRE-002와
  DCA2-DOC-002가 있다.

## Historical Regression 결과

과거 disposition을 goal checkbox로 상속하지 않고 현재 code/data/consumer/test로 다시 확인했다.

| Historical set | 현재 결과 |
| --- | --- |
| DCA 52개 | 47개 완료/비적용 유지, 1개 회귀, 3개 사용자 보류 유지, 1개 제외 유지 |
| CPR 15개 | 9개 DCA alias와 6개 unique finding 모두 완료 disposition 유지 |
| MSTR-01..07 | 현재 result/legacy/Shop/compendium/authority/Unity test evidence에서 완료 유지 |

회귀는 DCA-H-04/PD-09/G12 → DCA2-DATA-002 하나다.

사용자 보류 유지:

- DCA-M-08/G22 → DCA2-GAME-001
- DCA-M-21/G05 → DCA2-BATTLE-001
- DCA-M-28/G22 → DCA2-GAME-003

DCA-M-26/EX-01은 현재도 non-actionable/excluded다.

## Validation Evidence

Final/broad:

- `cargo test -p game_core --all-targets -- --test-threads=1`:
  776 passed, 0 failed, 0 ignored.
- `cargo check -p game_server`: pass.
- `cargo test -p game_server -- --test-threads=1`: 30 passed, 0 failed.
- Unity EditMode full suite: 241 passed, 0 failed, 0 skipped.
- Unity PlayMode full suite: 61 total, 57 passed, 0 failed, 4 explicit live-server tests skipped.

Focused Core evidence:

- `ron_loading`: 20 passed.
- `live_skill_catalog_audit`: 4 passed.
- `skill_runtime_contract`: 13 passed.
- world combat: 59 passed.
- snapshots/start: 22 passed; map flow: 32 passed; node sessions: 16 passed.
- equipment: 26 passed; Support: 12 passed; Shop: 6 passed.
- data: 125 passed; map: 23 passed; combat preview: 25 passed.
- targeting: 11 passed; range preview: 7 passed; Core battle focused group: 75 passed.
- live item-skill activation: 3 passed.

Registered live probe:

- Normal local game_server plus `unity_ws_smoke_probe.py`: success.
- Covered auth, pre-run VN, starter selection, map, preview, battle setup/update, resync, authority
  recovery, deployment preview/deploy, HUD/unit source/range preview and ping/pong.
- `WS_CHECK_COMBAT_RESULT=1` was not enabled, so this probe run does not claim live combat-result
  completion coverage.
- Initial sandboxed socket attempt failed with `PermissionError`; the exact command succeeded after
  approved localhost access. This was an environment restriction, not a product failure.

Diagnostic probes:

- Production-shaped nested RON probe: demonstrated DCA2-DATA-002.
- Loader-backed catalog derivation: 30 abnormalities, 86 total equipment, 66 placeholder E.G.O
  entries, 31 fragments, 30 abnormality fragments (2 ActiveSkill, 28 BasicAttackModifier).

## Recommended Follow-Up Goals

우선순위와 owner가 다른 correction을 한 goal로 섞지 않는다.

1. `unity_battle_presentation_event_consumption` — P1
   - TileArea telegraph/effect tiles, attack/cast/projectile, withdraw/death positions,
     buff/stat/resonance/battle-end consumers와 event-by-event EditMode/PlayMode tests.
2. `gameplay_ron_transitive_strictness_correction` — P2
   - shared nested types의 fail-closed unknown-field rejection과 production-shaped negative tests.
3. `battle_setup_catalog_ref_filter_correction` — P2
   - enemy briefing/wave kind-aware non-empty catalog resolution과 Core/server/live probe regression.
4. `documentation_alignment_corrections` — P3/document-only
   - source count 23→21, G15/G24 도메인 용어화, external enum inventory 정리.
   - `docs - 복사본` 정리는 user-owned backup 가능성을 확인한 뒤 별도 승인한다.
5. `alignment_test_hardening` — P2 evidence
   - Endless RunComplete direct regression과 위 correction별 missing assertions.
6. Policy/content goals — 사용자 결정 뒤에만 활성화
   - G05 abnormality ranges.
   - G22 equipment block/Maintenance recipes.
   - fragment/WhiteNight policy and content.
   - GrenadeLauncher reserved/removal/content disposition.

## Remaining Limitations

- Unity의 4개 explicit live-server PlayMode test는 이 broad run에서 실행하지 않았다. 별도 live
  smoke로 주요 wire 흐름을 확인했지만 두 evidence는 동일하지 않다.
- 실제 화면 캡처나 사람이 보는 animation 품질 평가는 하지 않았다. DCA2-WIRE-001은 source와
  consumer 부재로 판정했으며 visual polish 수준은 별도 UX 검증 대상이다.
- production credential, remote environment, durable external account/save backend는 사용하지
  않았다. 감사 범위는 현재 local workspace와 제공된 외부 Unity project다.
- design-only 콘텐츠 문서는 구현 완료로 해석하지 않았다. WhiteNight처럼 current wording이 live
  acquisition과 직접 충돌하는 항목만 별도 finding으로 분리했다.
- dirty worktree의 현재 상태를 감사했으며 기존 변경의 작성자나 의도는 추정하지 않았다.

## Audit Safety

이 감사에서 production code, live RON, tests, durable canonical, external Unity docs/code를
수정하지 않았다. 변경은 이 goal의 `PLAN.md`, `EXPERIMENT_NOTES.md`, `EXPERIMENTS.md`,
`AUDIT_REPORT.md`에만 한정했다. 실제 correction은 사용자 승인 후 별도 goal에서 수행해야 한다.
