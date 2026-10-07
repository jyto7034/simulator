# U08 통합 스택 회귀 교정과 수용 조건 보강 — 2026-10-06/015

integrated_core_default003는 컴파일 뒤1872시험을 시작했으나 기존 R07 공개 RON/fresh-process 시험에서 stack overflow/SIGABRT로 종료했다. 부분 ok157줄이 있어도 전체 summary가 없으므로 suite PASS가 아니다. core exit101/182.216450초, parent matrix193.109662초이며 source438/fa734235cef0c4b9ab73c9acac1785d0cdc9e9dd6123d70ff36efd0fe42b18a0은 불변이다. verify_remaining003은 시작하지 않았다. 같은 env/default stack의 exact1thread diagnostic001도5.843204초/exit101로 재현했다. LLDB는 debuggee 시작 전 helper handshake ack 오류로 실패해 동적 backtrace를 얻지 못했다. 권한거절로 단정하거나 승격하지 않았다.

Sol·Astra가 기존 r07_copies.rs20,456bytes와 명령을 U07-M 성공 receipt에 대조했다. 시험 및 기록된 CARGO 환경은 동일하고 이전 raw에는 해당 selector ok가 있다. 현재 RUST_MIN_STACK는 unset이나 과거 receipt에 그 변수가 없으므로 과거까지 동일이라고 단정하지 않는다. 부모 시험은 baseline simulate와 추가 TrialSession을 보유하고 child 결과를 capture/상태assert하므로 단순 child 실패로 현재 전체 SIGABRT를 설명하지 않는다.

실제 바이너리 정적 분석에서 prepare_application_internal의129,880byte frame이 run_deferred/run closure를 통해 같은 함수로 들어가 outer/inner 두 번 중첩되는 경로를 확인했다. 마지막 동적 overflow frame은 미확인이다. root가 실제 source292–460을 읽고 scope 선택 wrapper와 scoped 실행 body를 나누는 제품 내부 교정을 승인했다. 새 힙 allocation/한도상향/fixture축소 없이 profile·warning·deferred/run 순서,FnOnce1회,rollback·계량·반환/ownerlayout을 유지한다.

Sol의 world/application.rs 한파일 교정 뒤 source438/73a3d7b6b22a49e6632bcc3b83a5842d614ae0cae55dda324b573ff22ab20425다. root는 전체438파일과 patch를 대조해 다른437파일·membership 변경0을 확인했고 실행 body가 그대로임을 읽었다. product_world_scope_split001/RECORD.json은 원본과 교정을 보존한다. scopedfmt 후 같은 R07 exact diagnostic002를 실행 중이며 결과는 이 기록에서 미확정이다. 통과해도 전체 통합을 즉시 반복하지 않고 아래 수용 누락 보강 뒤 재개한다.

Astra의 coverage_preflight001은32문장에62 exact 시험/probe 참조를 연결한 읽기 전용 자기검토다. 실행0/입력수정0이며 supported는 assertion 존재를 뜻하고 현재 통합 PASS나 최종독립감사가 아니다. root가 실제 행·근거·보강안을 읽어 다음 기존 요구의 직접 검증 부족을 인수했다.

| 묶음 | 보강할 기존 수용 조건 |
| --- | --- |
| G1 | 일반 활성전투 유물 변경의 정확거절/전체roots 보존, 무기해제 후 실제유물slot 보존, Normal/Corrupt 중복정책 양방향·휴면 |
| G2 | 신규 외부RON의 잘못된reward/variant/selector/penalty/closure·수치/schema 진단 및 실제 삭제/변경catalog 복원거절 |
| G3 | 실제startup 효과 추가/제거·form/weapon 전환 재평가, slotpermutation 실제피해, 기존modifier·Physical/Magic저항과의 수작업 oracle |
| G4 | CorruptRelic 자체 early/late admission 실패의 사본·촉매·저장 원자복구, 비용부족·부활촉매 대체금지·부적격variant |
| G5 | 유물을 가진 실제 같은프로세스 reconnect·전투retreat의 복원시점, forgedslot public decode |
| G6 | 유한 반복전투·계획·지연payload 종료 뒤 과거owner 누적없음과 허용된 현재/최근결과 소유 구분 |
| G7 | CLI describe/definition-explain의 성공/nonempty를 넘어 신규정의 내용과 authored 상수·query 의미 대조 |

G1부터 작은 묶음으로 진행한다. Astra가 실제 API를 읽고 src/game/relic/tests.rs, tests/u08_relic_public/prospective.rs, src/game/world/tests/u08_carriers.rs 세파일 범위를 확인했다. 세 relic command는 requires_battle_authority=false이므로 normal core.execute가 적합하며 ordinarybattle의 정확 기대는 ActionUnavailable(ContextIrrelevant)다. domainfallback InvalidAction이나 authority오류를 기대하지 않는다. inventory4조건(reverse×dormant), weapon해제Normal/Corrupt, 실제battle3명령의 반복거절/전체roots를 검사한다. 현재는 Sol 진단이 끝날 때까지 Astra writer/runner0이다. 이후 작성/포맷/집중실행/일반교정 토큰을 넘기며 제품 변경·기대값 완화는 별도 원인으로 반환한다.

G1→G4→G2/G3→G5/G6→G7을 기본으로 실제 상태·수명 계약을 맞춘 뒤 순차 보강한다. 기존 성공·실패와작성 이력은 보존한다. 이 보강은 신규 정책이 아니라 frozen32수용문장의 누락 해소다. 이후 현재 source의 필요한 통합/public/실제measurement/PC/R05 및 evidence→impact→adaptation→coverage→자체검증→정확동결/복원→전체독립감사 순서를 유지한다. U08은 GEX10 partial이다.
