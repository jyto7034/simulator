# U08 최종 도구 준비 교정과 G6 계약 023

## 2026-10-06/023 — 최종 도구 준비 교정 인수 및 G6 실행 계약

보조 Astra의 finalizer_prep002를 인수했다. MANIFEST SHA bf519a16949f4d68151deb9af914510c572c9648439faedef55d0260f90f4bac, HANDOFF SHA f95a616ff898054dad9b8bf7a24addd0212cf834ac8c36fcc82b189ab41ea7a3이다. root는23 actual file hash/size, 원001디렉터리6파일, 신구 TOOL_INPUTS.tar.gz 각각10 exact member와 정확2개 신규도구 변경을 대조했다. evidence_common.py와 freeze_current.py만 교정했고 제품·test·기존 실행 harness·raw는 바꾸지 않았다. 최초 preparation 실패도 PREP_CHECK_FAILURE_001.json으로 보존했다. 작성자의42개 준비검사(32음성) 및9py 메모리문법검사는 실행 증거가 아니며 제품시험/계측/finalization/실제복원 실행은0이다.

measurement_support는 별도 JSON 산출물이 없고 cargo 로그 뒤에 process_heap JSON 한 줄을 출력한다. 새 measurement_support_stdout_json_line 형식은 선택된 자기receipt/raw 해시, 정확 cargo example/feature/command, 성공·source불변 및 실제 launch 뒤 정확 한 JSON 객체를 요구한다. instrumented=false와 live/peak=null을 보존하고0으로 바꾸지 않는다. 없거나복수/깨진/중단/다른실행/잘못된state는 거부한다. 일반 JSON 문서 검사는 유지한다. source_valid와 새 before/after 복원목록은 실제 run.identity 및 rows가 쓰는 sorted(Path)에 맞췄다. 기존 receipt의 원래 list/digest는 재정렬하거나 바꾸지 않는다. actual006/G2matrix003와 잘못된 digest/재hash한 문자열순서 음성을 준비검사에 사용했다.

stage순서,191+31/32수용문장, fresh48+ harness 인수와 Sol 실행전후 bytes/UTC/receipt구간, 기능종료 뒤 PC/R05 quiescence, 자체검토/문서인수, 직접 unwrapped freeze 계약은001과 같다. 실제 최종 config는 G6/G7/통합/성능이 닫힌 뒤 작성한다. 보조 작성자는 현재 writer/runner0이며 최종 독립감사자가 될 수 없다.

G6는 기존 Astra에게 value_expr_runtime/tests/relic_amount.rs 및 world/tests/u08_carriers.rs 두 testfile의 단독 편집·작은 집중실행을 위임했다. 기존 cfg(test) 자식에서 private owner/delivery/EvalCapture/source/binding/generation/plan의 Weak+extent만 관찰하고 public API/제품가시성은 바꾸지 않는다. 실제 Snapshot projectile release→withdraw 뒤 피해210→실terminal→Complete→다음entry/terminal을 유한3회 검증한다. 최소1회 연결을 먼저 확인한 뒤 확장하며 현재 최초 selector 실행 중이다. 아직 G6 PASS가 아니다.

전투owner는 activeNone 뒤 strong 소유가 끝나야 하고 payload/capture 소비와 최근종료캐시의 허용보존을 구분한다. Complete는 confirmed receipt를 없애지만 last_terminal은 다음entry에 남고 다음terminal에 교체된다. 장기 GameData/compiledcatalog의 generation/plan 보유는 누수로 오판하지 않는다. Weak::upgrade=None는 strong owner와 T의 drop을 검증하며 Weak가 남은 Arc backing allocation 전체가 해제됐다는 뜻은 아니다. 현재 retainedextent와 누적admission budget refund 또는 allocator heap상계를 혼동하지 않는다. fixture 일반교정은 승인된범위에서 자율반복하되 원실패/새label/실행중 입력불변을 지킨다.

Sol docs_checkpoint023 SHA baf589132d52d888efe35ae25efa944e3d9abe716c612bd676ec20a25e0be623의48중첩 파일참조와현행문서2개를 root가 대조했다. Sol은 통합004의 새label/빌드binder/실행순서만 읽기 사전확인하며 runner0이다. G6 후 G7 공개describe/Relic definition의 실제 typed recipe 및 fresh harness 인수, 현재통합, 기능종료 후성능과동결/독립감사를 계속한다. 전체U08 완료가 아니다.
