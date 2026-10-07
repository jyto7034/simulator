# R06 WP01 콘텐츠 입력·검증·컴파일 실험 기록

이 파일은 `PLAN.md`가 정한 구간의 시간순 명령·실패·재검증 원본 locator다. 이전 checkpoint/correction/review 증거는 수정하거나 이번 직접 실행으로 다시 세지 않는다.

## E00 — 인수·currentness·baseline

- correction_002 manifest SHA-256: `1bedd79cf745bb489db18172b3c29bc3477f728fa9849d2b4e36f357224ff85a` — 사용자 제공값과 실제값 일치.
- `python3 .../r06_verify_correction_002_target.py`: exit 0, source 311 / changed 4 / documents 3 / evidence 38 / validation receipts 17, verification PASS.
- latest review: `correction_002_review_001/REVIEW.md`, 계산 커널 부분 범위 PASS, 신규 finding 0. 전체 R06 판정으로 재사용하지 않음.
- 새 baseline: `baseline/pre_implementation_001/MANIFEST.json`, SHA-256 `e884bc38014d1ce05dcede78a495c9ae6c888b2e9d99ca10c1d58ba4a22fe6ec`; actual source 311, explicit impact 22, planned absent 8.
- baseline capture는 제품 source mutation 전에 완료했다. 생성 명령은 predecessor의 current source rows를 실제 bytes와 재검증하고 새 tar/manifest/status를 작성했다.

## E01 — dependency/interface 조사

- Cargo.lock/current metadata: `game_core`는 RON 0.5.1, Serde 1.0.228을 사용한다. lock에는 다른 workspace dependency용 RON 0.8.1도 있으나 이번 실제 crate 호출 경로가 아니다.
- RON 0.5.1 `Deserializer`는 public Serde visitor/seed를 호출하지만 depth 설정이 없다. `CommaSeparated`는 untrusted sequence size hint로 선할당하지 않는다.
- current production path: `content_pack::load -> snapshot::acquire -> CatalogKind::Skills::parse -> RawSkillDatabase -> resolve/assemble -> GameDataBase::from_validated_live_catalogs/new -> ValidatedGameData publication`.
- snapshot은 file/total byte와 delimiter nesting을 검사한 뒤 source string을 receipt용으로 clone한다. expression span byte 검사는 이 clone보다 앞에 들어가야 한다.
- existing `SkillEffectDef`는 literal `Damage/Heal`만 갖고 runtime command builder와 target usefulness가 이를 직접 match한다. 새 expression은 WP02 전 publication에서 차단하고 match 우회에는 explicit invariant failure를 둔다.

후속 명령·결과는 구현과 함께 아래에 추가한다.

## E02 — schema/parser/catalog 1차 구현과 실패

- `cargo fmt --all -- --check`: exit 1. 새 파일뿐 아니라 baseline부터 존재한 광범위한 포맷 차이를 보고했다. repo 전체를 기계적으로 재포맷하면 사용자 변경을 덮으므로 수정하지 않았다. 이 첫 실행은 recorder 도입 전이라 raw log 파일이 없으며 결과를 PASS로 세지 않는다.
- 첫 `cargo check --all-targets`: exit 101. private `SkillDatabase` import, damage/heal narrowing match 결과형, Serde `Error::custom` trait import, 네 test caller의 새 limits 인자 누락 등 구현 중 compile 오류를 확인했다. 모두 target 이전에 수정했다. recorder 도입 전이라 raw log 파일은 없고 실패 사실을 유지한다.
- 수정 후 `cargo check --all-targets`: exit 0, 기존 dead-code warning 6개. 이후 동일 범위 실행을 `stage_001_all_targets_check`로 원시 보존했다.
- 첫 `cargo test -p game_core game::value_expr:: --lib -- --nocapture`: exit 101, 29개 중 23 passed / 6 failed. RON 0.5.1의 `i128 is not supported`와 test expression의 전체 nesting 128 선행 거부를 실제 호출 경로에서 확인했다. recorder 도입 전이라 raw log 파일은 없고 실패를 PASS로 재분류하지 않는다.
- 교정: `TypedInteger` authored input에 custom integer visitor를 두어 RON signed/unsigned 64-bit를 i128로 무손실 확장했다. expression depth 경계 시험은 production에서 실제 도달 가능한 제한 8을 사용해 7/8/9를 검증하고, node 기본 경계는 1023/1024/1025를 실제 skill RON으로 검증했다. 교정 직후 같은 selector는 29 passed / 0 failed였다.

## E03 — checkpoint 001 직접 검증

모든 receipt/log는 이 구간의 `evidence/validation/`에 있으며 predecessor evidence를 덮어쓰지 않는다.

| label | 명령 요약 | exit | 실제 결과 |
| --- | --- | ---: | --- |
| `stage_001_value_expr_unit` | `cargo test -p game_core game::value_expr:: --lib -- --nocapture` | 0 | 29 passed, 0 failed, 1360 filtered |
| `stage_001_all_targets_check` | `cargo check -p game_core --all-targets` | 0 | dev all-target build 성공, 기존 warning 6 |
| `stage_001_focused_fmt` | value_expr 6개 파일 `rustfmt --check --config skip_children=true` | 0 | output 0 bytes |

신규 7개 시험은 실제 `SkillDatabase` RON을 사용해 정상 Add compile/evaluate, literal lowering, unit/resource/rounding/source 진단, runtime publication block, depth/node/member/input/nesting 경계를 검증한다. 기존 계산 커널 및 correction 회귀 22개도 같은 selector에서 직접 재실행했다.

## E04 — checkpoint 자체검토와 교정

- `checkpoint_001` 동결 뒤 자체검토를 별도 review 문서에서 수행했다. verdict는 `CORRECTION_REQUIRED`였으며 review PASS로 바꾸지 않았다.
- SR01: catalog가 compiler admission 전에 authored `AmountExpr`를 clone했다. `AmountInput::Authored(&AmountExpr)` borrow로 교정하고 legacy literal만 고정 크기 owned lowering을 유지했다.
- SR02: primary bounded parse 실패 뒤 진단 보강이 `ron::from_str::<Locations>`로 전체 RON tree를 다시 만들었다. 두 번째 serde parse를 제거하고 string/comment/raw-string을 opaque하게 건너뛰는 bounded lexical range locator로 교정했다.
- `stage_001_correction_value_expr_unit`: 29 passed / 0 failed. `stage_001_correction_all_targets_check`: exit 0, 기존 dead-code warning 6. `stage_001_correction_probe`: forbidden clone/tree-reparse pattern 부재와 borrow/locator 존재를 확인했다.
- 교정 재검토는 `ACCEPTED_FOR_WP01_CONTINUATION_ONLY`다. 이는 checkpoint 이후 external pack 연결을 계속할 근거일 뿐 WP01/R06 완료 판정이 아니다.

## E05 — external pack 연결 중 발견·교정한 실패

아래 구현 중 실패는 target 전 모두 교정했지만 recorder 도입 뒤에도 일부 focused 명령을 수동 실행했으므로 raw log가 없는 항목을 숨기지 않는다.

- external expression test 최초 compile: exit 101. `ValidatedGameData`가 `Debug`가 아니어서 `expect_err` 6곳이 compile되지 않았다. `.err().expect(...)`로 바꿨다.
- external expression 3-test 최초 실행: 1 passed / 2 failed. compile diagnostic의 pack이 누락됐다. `SourceTable::relocate`가 final definition의 실제 source identity와 정확히 일치하는 error에 pack/definition kind를 결합하도록 교정했다.
- final-winner 시험 최초 실행: replacement assertion 실패. compact RON serializer가 `Damage(amount:60,damage_type:Magic,)`처럼 trailing comma를 보존함을 확인하고 fixture의 실제 bytes를 대상으로 바꿨다.
- content-pack 23-test 최초 실행: 22 passed / 1 failed. 기존 unsupported losing-writer 회귀에서 `object_id`가 비었다. compact raw struct가 `SkillDef(...)` 이름 없는 tuple을 쓰므로 locator를 type-name 검색에서 root `skills` → definition tuple → `steps` → action tuple → `effects` 경계 탐색으로 교정했다. exact 재실행 1/1과 기록된 23/23 및 59/59에서 복구를 확인했다.

## E06 — 실제 입력·호환성 직접 검증

| label | exit | 실제 결과 |
| --- | ---: | --- |
| `stage_002_value_expr_unit` | 0 | 29 passed; actual skill RON의 depth 7/8/9, nodes 1023/1024/1025, members 2/3/4, input limit-1/limit/limit+1 포함 |
| `stage_002_content_pack_tests` | 0 | 23 passed; external 정상 compile/source, invalid unit/reference/moment/rounding/syntax, publication block, winner 경계 포함 |
| `stage_002_content_pipeline_regression` | 0 | 59 passed; snapshot/resolver/raw/diagnostic/packaging 포함 |
| `stage_002_skill_runtime_regression` | 0 | 15 passed; 기존 literal authored RON 전투 동작과 exact effect source 유지 |
| `stage_002_authoring_regression` | 0 | 15 passed; authoring validation/publication budget 경계 유지 |
| `stage_002_checkpoint_save_regression` | 0 | 18 passed; V7 checksum/round-trip/continue/external generation 저장 회귀 유지 |
| `stage_002_v7_checksum_regression_rerun` | 0 | 1 passed; exact old-build V7 envelope/checksum 시험 실제 실행 |
| `stage_003_preserved_fixture_hashes` | 0 | preimplementation metadata, V7 JSON, literal RON 3개 SHA-256 모두 일치 |

`stage_002_v7_checksum_regression`은 잘못 붙인 `--exact` 때문에 exit 0이지만 0 tests였다. 유효 검증으로 세지 않으며 `_rerun`만 직접 실행 결과로 사용한다.

## E07 — 최종 품질·current-source 검증

- 최종 source code mutation 뒤 `stage_003_content_pipeline_regression`: 59 passed / 0 failed.
- `stage_003_all_targets_check`: exit 0, 기존 dead-code warning 6.
- `stage_003_clippy_with_baseline_allows`: exit 0. predecessor와 동일하게 `clippy::never_loop`, `clippy::eq_op`만 CLI allow했다. 355 warnings(259 duplicates)은 raw log에 보존했으며, 새 parser/catalog의 `result_large_err`는 기존 large diagnostic value를 그대로 반환하는 경로에서 발생한다. 이를 숨기기 위한 추가 allow나 API boxing은 이번 범위에 넣지 않았다.
- `stage_002_focused_fmt`: 새/커널 `value_expr` 6개 파일 rustfmt check exit 0, output 0 bytes.
- `stage_002_touched_legacy_fmt_audit`와 `_reaudit`: exit 1. 첫 감사에는 새 test block 포맷 차이도 있어 해당 block만 교정했다. 재감사의 잔여 diff는 baseline부터 있던 `skill_data.rs` resource-validation 블록과 untracked `content_pack/tests.rs`의 기존 resource/lifetime 블록뿐이다. 사용자 소유 dirty bytes를 전체 rustfmt로 바꾸지 않았다.
- `stage_003_allocation_and_locator_probe`: compiler 전 authored clone/전체 Locations tree 재parse가 없고, acquisition scan이 `bytes.clone()`보다 먼저이며 lexical tuple locator가 존재함을 확인했다.
- 동결 직전 ad-hoc currentness 감사의 첫 실행은 임시 after row에서 `path`를 누락해 313개를 모두 변경으로 오판했다. 두 번째는 baseline porcelain v2와 current v1 status를 비교해 33,389개를 오판했다. 동일 row shape와 `--porcelain=v2 --branch`로 바로잡은 뒤 planned-absent parser/catalog 2개를 명시적으로 허용해 재실행했고, source 313 / expected changes 19 / unexpected global added 0 / removed 0을 확인했다. 이 감사 실패들은 제품·target drift가 아니며 raw recorder를 쓰지 않은 read-only 보조 검사였다.

## E08 — 재사용 증거와 종료 경계

- 재사용: correction_002 manifest 및 `correction_002_review_001/REVIEW.md`의 계산 커널 부분 PASS. 이 구간 시험 수로 합산하지 않았다.
- 직접 실행: 이 segment `evidence/validation/` receipt/log. 각 receipt는 command, cwd, UTC start/end, duration, exit, raw-log SHA/bytes를 가진다.
- known limitation: authored expression은 parse/validate/compile/catalog까지만 가능하다. production publication은 `content_capability_unsupported`로 거부된다. battle `EvalFrame`, Snapshot capture, Live/resource/result execution은 WP02이며 이번 구간에서 구현하지 않았다.
- 최종 상태는 `부분 구현·검증 종료 / 단계검토 요청 대기`다. target 동결 뒤 mutation·자체 PASS 선언·다음 구현을 하지 않는다.
