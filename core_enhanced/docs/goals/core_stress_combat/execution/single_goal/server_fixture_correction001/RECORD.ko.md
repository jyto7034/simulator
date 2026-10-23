# 서버 node-flow 메시지 fixture 교정

사용자 요청: 기존 실패 deserializes_snake_case_node_flow_requests 교정. 지시서 §4/5/7/8에 따라 감사004와 분리된 일반 테스트 교정이다. 제품/공개 계약 변경은 예정하지 않는다.

실행 전 근거: core_runtime_contract.md의 필수 loan_copy_ids 배열(0–2), behavior.rs의 Vec<Uuid>, ApplicationCommand::from_document와 서버 deserialize_command를 대조했다. 이 검사는 gameplay 상태전이가 아니라 JSON envelope→ApplicationCommand→transport의 파싱·kind·원문보존이 목적이다. 직원ID a/b/c의 실제로스터 자격은 이 seam의 검증대상이 아니다. 기존 assert_command가 payload 전체를 비교한다. 정상 입력은 빈 배열/고유UUID2개를 명시하고, 옛 누락입력은 부정 대조로 사용한다. API에 default를 추가하거나 assertion을 완화하지 않는다.

원본 실패: ../runs/review-server001.log의9실행/8pass/1fail, exit101. 같은 계약·시험입력의 유효 원문을 재사용하여 중복 red빌드를 하지 않는다. 변경 전 파일은 messages.before.rs로 보존했다. 변경 후 같은9개 메시지 검사를 단일Cargo 호출로 실행한다. 기존dev 빌드1m41s 근거, 첫60초 로그진단/10분 초과 원인확인. 관련입력은 실행중 고정한다.

공유가정 점검: game_server/src 및 Core통합시험의 select_starter_employees 입력을 검색했다. handlers.rs와 Core공개fixture는 loan_copy_ids를 명시한다. 별도 runtime_measurement/playback.rs의 구형 enter_combat에는 동일 누락과 제거된 Standard 모드가 함께 남아 있다. 이는 이번 node-flow 파싱검사의 호출경로가 아니며 별도 진행기 마이그레이션이다. 이번 테스트 수정의 검증으로 해당 측정기 정상진입이나 서버전체green을 주장하지 않는다.

동결 영향: target u164_current_001의 messages.rs 한 member가 바뀌므로 unchanged로 유지하지 않는다. cfg(test) 입력만 변경되어 U16-4의 제품/계약/기존기능증거는 비영향이다. 최종 diff와9개실행결과로 current/proven non-impacting delta를 기록하고 과거target/감사를 보존한다.

## 결과

messages001.log: **9pass/0fail/0ignore, exit0**,111filtered, 빌드1m24s/실행0.03s. 이전 실패1건을 닫았다. 테스트를 지우거나 기대값을 완화하지 않고 정상2형태와 누락거절을 기존검사에 연결했다. cfg(test) 이전 제품부분과 해당함수 밖 전체내용은 변경전과 동일하며 git diff --check도 통과했다.

재현은 RESULT.json의 command를 저장소 root에서 실행한다. 다른검사111개/전체서버/별도legacy측정기는 이번에 실행하지 않았다. U16-4 감사004와 당시실패로그는 불변으로 보존하며 현재성은 해당시험1파일의 proven non-impacting delta다. 전체재동결·재감사를 수행한 것으로 표기하지 않는다. 본 실행자는 이제 이시험교정 참여자이며 이교정의 비참여감사자가 아니다.
