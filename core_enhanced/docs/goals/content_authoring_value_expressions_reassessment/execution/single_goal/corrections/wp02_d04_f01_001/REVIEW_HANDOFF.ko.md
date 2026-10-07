# R06 WP02 D04-F01 교정 재검토 인계

상태: **D04-F01 교정·자체 검증 완료, 독립 재검토 대기**. 시작점은 D04 manifest SHA-256 `24bd3b7e748b94037b2ec57822a7b7ef2b67406b9d2635a3a23a6b3eac7ba8bf`, 333개 source digest `4c63ca018cd39d50e4a896711debaaf7970dc2636841a77eb74457f83313582f`다. [독립 검토](../../reviews/wp02_d04_resolved_target_001_review_001/REVIEW.md)의 D04-F01 한 건만 교정했다. successor는 `target/d04_f01_correction_001/MANIFEST.json`이며 최종 제품 source digest는 `34ceedbebc84b9396d42f92fa31033fe0b831a90e5325c8c4463e0c88f067145`다.

## 원인·변경·관찰 근거

| 경계 | 교정과 시험 |
| --- | --- |
| 빈 target impact | `sim.rs`의 expression amount 준비에서 targets=[]이면 `Some(empty)`를 만든다. capture target lookup·Live member buffer·대상별 평가를 건너뛴다. carrier와 해당 effect row의 존재는 확인한다. effect command·signal·projection·prefix/action witness settlement·step result·delivery close 경로는 계속 실행한다 |
| 실제 target impact | 첫 실제 target의 capture 검사, effect당 Live member 1회 준비, target별 exact lookup과 모든 amount의 mutation 전 평가를 유지한다. 기존 Snapshot Attack5/Live Attack9 hit 시험과 area tick 시험이 각 1건 통과했다 |
| 검토 재현 4조합 | 상수식·ResolvedTargets Sum × Snapshot/Live의 실제 `invoke_ability`→release→queued empty impact를 제품 시험으로 편입해 4/4 통과. 각 case에서 terminal empty impact log, cast/projectile/queue 정리, settled action의 delivery·pending step 0, damage/HP loss/death 0, active action 0을 확인했다 |
| 관련 경로 | inactive target과 발사 후 이동으로 미명중 timeout된 active target의 empty impact도 통과. 빈 DamageExpr·HealExpr 뒤 SourceEmployee Grant는 한 번만 적용돼 balance/fact delta 3. 별도 step 결과는 resolved/damage/kill count 0 및 후속 effect count 1. release target 외 capture는 선택되지 않는다 |
| 실제 오류 | 후속 TargetEmployee 자원 InvalidOwner가 candidate 오류로 전달된다. 앞선 commit +2는 남고 실패 candidate의 +3·queue/log 변경은 복원된다 |

ResourceCatalog를 `GameDataBuilder::build()` validation 전에 설정할 수 있는 fixture 전용 `with_resources`를 추가했다. resource runtime과 projectile 소실·미명중 정책은 수정하지 않았다. `NOTES.md`에 호출·transaction 소유권을 기록했다. 교정 전 4건 실패와 선행 상수식 2건 통과의 원본은 `evidence/predecessor_review/` 및 `PRIOR_EVIDENCE.json`에 결속했다.

## 최종 source에 결속된 검증

`EXPERIMENTS.md`와 동명 `evidence/*.json`·`.log`에 command/cwd/env/toolchain/selector, exit, 전후 333개 hash와 전체 출력을 보존했다. 최종 revision에서 all-targets check, 신규 9건, value-expression 74건, 정상 target-locked 및 area 각 1건, 전체 제품 **1,509/1,509**, Clippy와 변경 영역 format이 통과했다. 전체 시험은 이전 D04의 Unix socket sandbox EPERM 근거에 따라 승인된 환경에서 실행했다. PC 36조건은 성공31/typed refusal5, 이전 gameplay oracle·상태·오류 code 차이 0이다. PC의 시험 전용 격리 빌드 source와 hook은 `pc_isolate_002.json`·`pc_isolate_hooks_002.tar.gz`에 묶었다.

R05는 원 concurrent cap **9,030,188 bytes**를 유지했다. 최종 교정 소스 직접 실행의 첫 두 peak는 **9,040,041 / 9,040,546 bytes**로 실패했고, 동일 도구의 동결 D04 source 대조도 **9,050,125 bytes**로 실패했다. 마지막 최종 소스 직접 실행은 **9,020,761 bytes**로 guard를 통과했다. 모든 완성 실행의 allocation 횟수·retained delta·결과 parity는 동일했다. 따라서 마지막 PASS만 제시해 peak 변동을 숨기지 않는다. `R05_SUMMARY.json`에 원 sample과 대조를 모두 둔다. latency gate는 미적용이다.

새 구간의 `baseline/entry_001`은 수정 전 실제 333개 source bytes와 branch/HEAD/dirty 상태를 보존한다. successor target은 수정 후 archive, 4개 변경 경로, before→after/after→before patch, 증거 archive와 manifest를 보존하며 `freeze.py verify`가 currentness·양방향 byte 복원을 확인한다. 기존 D04 동결본과 독립 검토는 덮어쓰지 않았다.

## 재검토와 잔여 범위

재검토에서는 빈 impact 후 공통 effect/step/delivery 완료, 고정 target capture 불일치 거부, D04 기존 positive 회귀, R05 peak 변동의 의미를 확인해 달라. resolved ByOwner의 명시 domain, 실제 unit scalar producer, D07 preview/explain, WP03/04, production publication 차단은 그대로 남는다. 자체 검증을 독립 PASS나 WP02/R06 완료로 표시하지 않는다. 이번 교정은 여기서 중단한다.
