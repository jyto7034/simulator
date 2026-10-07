# U08 통합 포맷 교정과 새 실행 — 2026-10-06/014

integrated_matrix002는 첫 fmt check에서 제품 game/mod.rs의 relic module 순서 및 runtime_observation.rs의 두 import 순서 차이로 exit1이었다. 시험0이며 이후 suite는 시작되지 않았다. 원raw/receipt를 보존했다. Sol이 두 파일만 skip_children scoped rustfmt로 교정했다. raw formatter exit0/2.153548초이며 runner의99는 의도된 source 변경을 감지한 wrapper 상태다. 기록용 byte 검증의 초기 assertion 두 실패는 별도 CHECKPOINT_ATTEMPT001/002로 보존했고 제품 교정 실패나 시험 결과로 세지 않는다.

새 source438/fa734235cef0c4b9ab73c9acac1785d0cdc9e9dd6123d70ff36efd0fe42b18a0이다. root는 전체 전후파일 목록과현재digest를 대조해 두 파일 외 변경·membership delta0을 확인했다. 시험/fixture와45검증 입력은 그대로다. product_integrated_format001/RECORD.json과 PREIMAGES/AFTER/DIFFS가 exact 순서 변경을 보존한다. carrier complete001 등 기존 집중 증거는 이 형식 변경의 명시적 비영향 범위에서 재사용한다. 과거 receipt/source는 소급 수정하지 않는다.

fresh integrated_matrix003을 시작했고 integrated_fmt003은 exit0/4.913842초/source unchanged/빈raw로 통과했다. 현재 Sol이 Core 기본부터 나머지 통합을 실행하며 성공 뒤 verify_remaining003을 진행한다. PC/R05는 기능 종료 뒤 전환한다. input writer는0/HOLD다. Astra는 입력을 변경하지 않고16군의32수용문장을 현재 시험·공개probe와 대조하는 coverage 사전 문서만 작성한다. 현재통합 성공·최종독립감사·U08완료로 취급하지 않는다.
