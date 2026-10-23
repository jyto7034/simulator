# 추가 통합17 교정

사용자가 분류 결과의 실제 교정을 요청했다. 기존 원본17의 고유 assertion을 보존하며16유지·기본 등록장착1통합을 수행한다. 단독 수행, 제품 실행 로직 변경 없이 테스트/fixture와 시작 장비 문서 정합화가 범위다. 기존138 교정·U14 동결 이력은 보존한다.

준비: Standard→Expedition, required loan_copy_ids=[], 활성 런 거절은 false 입력으로 확인, Abandon 후 false로 시작하고 실제 PlayerProfile을 설치한 새 앱에 Continue한다. 현행 시작 Tank shield/last_line 규칙을 기대값과 현재 규칙표에 반영하되 영입 Dealer greatsword/breakthrough는 유지한다.

미완료 이벤트: 공식 시설 템플릿6A와 현재 layout을 작성 입력으로 사용한다. ENTRY→깊이1 단일방, 입구S에 인접한1번 슬롯에 지원 Event를 예약한다. 선택된 유물Normal과 사전 유료타락 유물Corrupt, 촉매2/분진11에서 실제 map select/confirm으로 입장한다. 무료 선택 뒤 미완료 checkpoint는 진입전과 같아야 한다. 해당 실제 profile을 설치한 새 앱 Continue에서 임시 variant/history만 복원하고 사전지급비용 유지·재진입·완료·다시Continue 이력 보존을 검증한다. admin Event 진입이나 Tutorial 모드로 치환하지 않는다.

통합: content()에 파일 존재+2packs 검증, 성공이전 테스트에 최초 단일copy/owner/slot0 검증을 옮긴 뒤 slot2→새직원slot1 기존검증을 유지한다. smoke 독립1만 제거한다. 빈 목록 JSON생략·잘못된 공개명령은 유지한다.

검증: 변경한 통합2target 전체(ron_loading25+u08신규15=40)를 먼저 실행한다. 빌드1–5분/실행1분 내 예상, 기존 runner60초 관측,5분 초과 시 진행/로그 진단. 입력 고정, 실패원문 보존 후 근거 있는 교정. 제품·공유common은 불변이므로 기존 lib2097/계측23/CLI12 근거는 비영향 재사용하며 불필요한 전체 재실행은 하지 않는다. 최종에는 원17→현재16대응 및 전체통합 기존결과 비영향/필요한 추가실행을 기록한다.

완료: correction001은 새 fixture의 typed ID From<&str> 가정으로 빌드 실패(시험0); 공개 ::new 생성자로 교정했다. correction002에서 ron25+U08 15 전체40통과. 코드 입력은 그 뒤 변경하지 않았다. 원17→현재16 대응 모두 PASS. 추가 전체 실행은 비영향 근거가 충분해 하지 않았고 다른통합32 및 기존lib/계측/CLI를 재사용했다. 결과는 REPORT.ko.md와 EXECUTION_INDEX.json에 연결했다.
