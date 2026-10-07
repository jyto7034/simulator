# U05 F01 문서 이동 인수 후 독립 영향 기반 GEX-20 재감사

원 연속 요청은 구현 비참여 GPT-6 Astra / High의 독립 감사·교정·재감사와 선행 충족 후 다음 작업을 명시 승인한다. core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md, GEX §5/6과 GEX-20, 원 P2G-50의 whole single-phase 범위를 따른다. 새 감사자는 구현 비참여다. Root는 이 대상 고정 후 source/docs/evidence를 변경하지 않는다.

현재 대상은 execution/single_goal/target/u05_002/MANIFEST.json이다.
Manifest SHA256: 0d31d3036b42a088bc764cb7841d36f1544324ab56a8929db4437ce03b85319c
Product368 SHA256: 3129db450b14515564da3845313cf0ce1539e9f99c032c89788a3a5fe840d283
Combined source/canonical SHA256: 44cb47a6d5a09934534e8b797c4263e0d9b9fdb017f8106d0e31246917203437

현재 실행 locator는 execution/single_goal/CURRENT_EXECUTION.json과 GEX_10_RESULT_002.json이다. 원 GEX_10_RESULT/PLAN/P2G/target001/audit001은 당시 bytes 그대로 보존했다. 현재 교정 계획·실행·관찰은 execution/single_goal/corrections/cf_d09_001/{PLAN.md,EXPERIMENTS.md,EXPERIMENT_NOTES.md,DOCUMENT_PATH_MIGRATION.json,EVIDENCE_REUSE.json,EXTERNAL_DOCUMENT_MIGRATION.diff}다. COVERAGE.json/PREDECESSOR_IMPACT.json의 14개 승인 조건·127개 상속 조건·별도31개 R06 조건과10개 검증 차원의 의미는 변경하지 않았다.

선행 독립 감사 post_audit/u05_20261004_001/COMPLETION_AUDIT.md(SHA256 f89dd050d8343f911d4532ace8330a87d9035e66842a7129cc2f86e935ae994f)는 Not Complete / High이며 유일한 F01은 감사 중 외부 CF-D09가11개 현재 문서 binding을 바꾼 것이다. 별도 제품/source 결함은 없었다. CONDITION_REVIEW.json은 모든 조건/차원과 원 의미·source 판정을 기록하고, EVIDENCE_VERIFICATION/PC_ACCOUNTING_RECHECK/DOCUMENT_BODY_CHECK/PRODUCT_EXIT는 실제 독립 확인을 기록한다. 이 근거를 직접 검토하여 적용 가능한 조건의 판단을 재사용하고, 이번 변경/미확인 부분만 새로 검토한다. 단순 문서 확인만으로 전체 verdict를 주거나 반대로 fresh audit라는 이유로 제품 전체 시험을 재실행하지 않는다.

현재 권위는 CF-D09 / USER18: 최신 정책은 이제 core_enhanced/docs/game_rulebook.md 및 game_rules/다. 과거 룰북은 archive/game_rulebook_legacy_2026-10-04.md다. 옛 next 경로는 제거되었고 현재 파일명으로 과거 기록을 재해석하면 안 된다. 이동 근거는 core_first/execution/rulebook_path_migration_20261004_001의 RECORD/EVIDENCE/BEFORE다. 이 외부 사용자 변경을 되돌리거나 원래Goal/Freeze/audit를 수정하지 않는다.

Root의 교정:46문서의 이동 전 archive/current hash,10상세규칙 및 최신/과거 룰북의12본문 비교,11기존mismatch 각각의전후mapping을 확인했다. 상세규칙은 정확한 parent-link 및 명시한 historical comparison link/label 변환만 허용했고 최신본문은 상세규칙 디렉터리 이름만 변했다. 전체diff를 남겼다. Product368 membership/digest와 기존43receipt/log/current+archivedcommand-input hash가 원래와 같아 실행근거를 재사용했다. 제품/테스트/data/정책 acceptance는 변경하지 않았고 product rerun은 하지 않았다. 새 target은 기존sourcebefore375와current377의정역fuzz0복원, 현재records/references, 선행U04target003 및 U05target001의8개largearchive actualmembers/hash를 결속했다. 현재형role에는외부문서귀속을명시했다.

검토 요구: F01 원인·이행의 적용성, 현재 모든 source/canonical/reference/evidence binding과 entry/exit currentness, 정확한 문서 정책 의미 보존, source/receipt 재사용 근거, 원14승인/127상속/31R06/10차원의 전체 현재 적용성을 추적해 Complete / High | Not Complete | Blocked를 독립 판정한다. GEX-20 single-phase 전체 Goal verdict여야 하며 F01만 닫았다고 U05를 자동 완료하지 않는다. 신규 source gap이 있으면 actual impact에 따라 직접검토/안전한read-onlyprobe를 추가한다.

성능·범위 조건도 그대로다. 원PC게임플레이oracle576행/36조건(31성공5거절)은pass. 실제비용은64bytes/unit 및sourceclone증가로상승; explicitaccountedcomparison이모든non-timing필드를대조하며unchangedcost는주장하지않는다. strictfailure와원래96logmismatch는보존하고explicitnonemployeefixturepin이역사적입력을복원한것이다. 선행독립감사자가모델과raw576을재검증했다. 기존public/R05상한과onlyapprovedconcurrentpeak경고(+466821bytes),계측latency관찰한계,20분시험제외를유지한다. 새Unity/원격서비스/직원소환API/과거저장자동호환주장은없다.

감사 중 product/docs/data/test/evidence/status/target은 읽기 전용이다. 산출물은 이 post_audit/u05_20261004_002 디렉터리와 필요한격리/tmp만쓴다. 수정하지않고finding이면정확한source/위반조건/책임delta/교정범위를남긴다. 새에이전트추가위임은하지않는다. COMPLETION_AUDIT.md와조건별reuse/새확인근거,entry/exit currentness 및auditclose를기록하고freeze해제가능시점을알린다. U06는후속전체Complete/High전에는구현하지않는다.
