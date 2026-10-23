# U17 후속 실행 기록

## 인수

현행 연속 지시서, HANDOFF, 로드맵 후속 계약, P2G/GEX Reader Contract, 저장 정책과 관련 구현을 대조했다. 기존 전체 dirty 작업은 유지했다. 실행 중인 소유 build/host는 없으며, 기존 U17 실패 교정·동결 근거는 재실행하지 않았다. `BASELINE.tar.gz`는 이번 변경 대상의 실제 인수 시점 bytes다. 새 파일은 baseline에 없음을 변경 목록에 기록한다.

로드맵의 상태 타입 분리 가능성과 현행 exhaustive capture를 비교한 결과, wire/타입 전체 재설계 대신 현재 checkpoint owner의 복사·설치와 수명 경계를 정리한다. 정책/완료 기준은 그대로이며 formal P2G 정책 successor가 필요한 변경은 아직 없다.

## core-u17-001 / core-storage-002

첫 변경(공통 clone/설치, source/stage, 완료 정리, fresh application 다음 명령)은 `core-u17-001.log` exit0,24pass/0fail/0ignore. 비용 변경 후 `core-storage-002.log` exit101,29pass/1fail/0ignore. 실패는 기존 alias 시험의 `actual.work == clone.work` 기대였다: 추가된 비유물 Arc admission으로 실제3693/기대3432. clone slots와 정상 공개 복원은 통과했다. 기존 시험이 의도했던 clone-limit exact/one-short는 유지하고 Arc owner 비용이 추가됨을 구별하도록 기대를 교정했다. 기본 한도·workload 완화는 없다. decode와 Value site를 구분한 기존 cumulative/one-short 검사는 통과했다.

상태 타입 전면 분리는 하지 않았고 wire V17은 유지한다. 비용 helper의 역사적 `relic::allocation` 경로는 다른 도메인에서도 이미 사용하며 일괄 rename만으로 동작이 개선되지 않으므로 이번에는 보존했다. Continue 활성화/계량 조건은 실제 유물 유무와 분리했고, 관측 compatibility를 위해 기존 Value/byte site 이름을 유지하되 typed decode만 별도 site로 식별한다.

## 확대 회귀와 codec 교정 003–007

`core-related-003.log`: exit101,238pass/2fail. (1) max_work=2000인 host tick fault fixture가 새 저장 계량을 만나 VN 준비에서 먼저 거절됐다. 목적은 실제 tick fault origin이므로 준비에는 기본 profile을 쓰고 기존 작은 허용량은 목표 연산의 outer native scope에 적용했다. 다른 저예산 fault fixture도 같은 전제였으므로 함께 교정했다. (2) 기존 3500 Run 자원 ledger 시험이 매 entry마다 catalog 전체를 탐색하여 새 누적 계량에서 10M work를 초과했다. 입력3500과 기본 한도를 유지하며 borrowed sorted index를 한 번 구성하는 실제 알고리즘 개선을 적용했다. `core-correction-004.log`:55pass/1fail, fault/resource 기본 검사는 통과했고 큰 SavePoint Value 예약에서21,787,886bytes가 요청되어16MiB를 넘었다.

작은 객체3500개에 대해 전체 member 수를 합친 뒤 B-tree 노드를 더하면 실제로 분할되지 않는 작은 root에 여분 노드를 만든다. 고정 depth128 stack으로 객체별 root/split을 계산하며 serializer/typed decode의 각 객체·sequence·string/scratch 예약을 구분했다. 별도 작은 예제는 3500개의4필드 객체→3500roots, 11필드→1root,12필드→3nodes를 검사한다. generic projection bound는 유지한다. `core-accounting-005.log`:33pass/1fail, 다음 encoded buffer에서17,189,417bytes. 지원 serde_json BTreeMap의 이미 정렬된 key를 다시 Vec로 수집·sort하던 writer를 iterator로 바꿨다. `core-accounting-006.log`:33pass/1fail, 최종 bytes Arc에서17,043,614bytes.

다음 교정은 한도/입력 축소가 아니라 실제 중복 생산 제거다. SHA-256은 canonical writer에서 streaming하여 digest용 전체 Vec를 만들지 않는다. reader는 검증에 이미 만든 normalized wire Value를 envelope로 이동·인코딩하며, 정상 복원된 immutable continuation을 보존해 World 재capture/재encode를 생략한다. gameplay roots와 cached roots는 독립 복사를 유지하고 SavePoint 공유는 공통 preflight를 거친다. Continue 후 reconcile는 이미 검증된 현재 콘텐츠의 지원 상태를 새 게임 진행으로 바꾸지 않는다는 계약이며, 기존 canonical/next-command/fresh/저장 경계 시험으로 확인한다. 누적 admission은 환급하지 않는다.

`core-codec-007.log`:48pass/2fail. U08의 normalization 예약 기대8은 중복 제거 후4로 교정했다. 3500자원 복원에서는 typed RunResourceEntry의 두 wire object(record와 tagged owner)를 JSON B-tree 두 개로 가정한 예약이 남아17,477,772bytes로 거절됐다. 이 닫힌 schema만 `2 * sizeof(RunResourceEntry)`의 Vec 성장 예약으로 바꾸고 문자열·scratch·다른 모든 object bound는 유지한다. Owner는 Employee/EquipmentCopy/FragmentCopy 모두 externally tagged UUID이며 같은 두-object shape다.

## 전체 Core 및 native 회귀 008–012

`core-lib-008.log`:2159pass/1fail/6ignore,204.87s. 유일 실패는 Unix socket 생성의 sandbox EPERM이며 `socket-009.log` 권한 재실행1pass/0fail. 따라서 해당 입력의 기본 Core 유효 결과는2160pass/6ignore다. 기존3500자원 사례가 기본10M work/16MiB에서 통과했고 workload를 줄이지 않았다. `native-u17-010.log`:25pass/0fail, 실제 fresh application의 결과 전/선택/수령/SavePoint 다음 행동 대조와 exact/one-short/retry를 포함한다. 저장/native 및 server persistence는 별도 로그로 수집한다.

## 실제 process와 terminal 준비

새 plain/instrumented release, issuer, calibration/adoption을 만들고 과거 runner를 읽기 전용으로 재사용한다. 정상3후보/확장64후보의 콘텐츠·suite·selection·기본 한도는 그대로다. forward333명령 → 실제 저장 milestone → fresh process 후속 행동 → IO거절/동일재시도를 순차 실행한다.

귀환은 원래 fixture의 B1/B2만으로 정책상 가능하지 않으므로, 별도 terminal authored fixture에 B2와 같은 SavePoint room을 depth3에 추가하고 B2→B3를 연결한다. 동일 수동 경로를 마친 뒤 B1 Gate Continue→B2 SavePoint→Gate Continue→B3 SavePoint→Gate Return으로 실제 terminal publication에 도달한다. Return 직전 IO실패 후 같은 명령 재시도, 새 host에서 active 없음/profile bytes 보존/중복 reward 거절을 관찰한다. 별도 branch의 AbandonRun은 명시적 실패 정산이며 전투 패배로 표기하지 않는다. 기존 전투 패배·SavePoint 결정 검사는 Core 전체 결과에서 별도로 유지한다. 새 fixture는 정상/확장 비용 비교에 섞지 않는다.

정상/확장 각각333명령·fresh14분기·실제IO17거절/동일재시도가 통과했다. 새 plain/instrumented calibration/adoption도 기존 한도로 통과했다. `public001.log`의 마지막 exit1은 terminal 준비 오류다: depth2 room을3으로 옮길 때 depth3..7의 필수 omen-compatible slot을 빠뜨려 package validator가 거절했다. 게임 명령 실행 전에 발생한 제작 fixture 오류이며 실패 원본 `terminal_inputs001`, `terminal-package001.*`를 보존했다. `facility_layout.rs:206–208` 계약에 따라 미예약 slot2를 omen으로 지정한 `terminal_inputs002`를 새로 만든다. 기존 정상/확장 입력과 결과는 다시 실행하지 않는다.

`terminal-forward002`333명령은 통과했다. `terminal-terminal002`는 B2 Gate가 자동 완료된 뒤 진행기가 추가 Continue 선택을 보내 `context_irrelevant`로 중단됐다. 실제 trace에서 Gate 확인 후 viewing_map/pending=None이며, 정책상 Return은 depth3 이상에서만 제공된다. depth2 자동 전이를 명시적으로 assert하고 depth3에서 pending 선택을 assert하도록 진행기를 교정했다. `runs/terminal-failed002`에 실패 요청/응답/store를 보존했고 변경 없는 terminal forward는 재사용하여 `terminal-terminal003`만 재실행한다.

`old-writer-fresh001`은 변경 전 U17이 발행한 원본 정상 저장 milestone을 새 binary/config에서14분기로 복원·소비하여 통과했다. V17 writer 호환성을 새 writer끼리의 왕복으로만 대신하지 않았다. V17 이전 개발 저장의 비호환 원본 보호는 기존 server persistence 검사가 유지하며 임의 변조 방어 범위는 확장하지 않았다.

`terminal-terminal003`: Return과 AbandonRun 각각 실제IO거절/동일재시도→writer 종료→fresh reader active 없음/profile bytes 불변/중복 reward 거절 PASS. terminal fixture 오류 두 건은 제품 실패가 아니며 제작 validator·명령 자격이 올바르게 거절한 결과다. 최종 입력은 `terminal_inputs002`, 진행기는 현재 `terminal_readers.py`; 공개 raw는 `runs/current001/terminal-terminal`이다.

`native-storage-012.log`:6pass/0fail(19.48s), `native-shared-013.log`:45pass/0fail(70.44s), `server-persistence-011.log`:29pass/0fail(0.52s). native 검사끼리 중복 사례가 있으므로 합산 고유 시험 수를 주장하지 않는다. server 검사에는 실제 자식 프로세스16개 crash 지점의 수락 전/후 복구와 profile/active/backup 일치가 포함된다. plain/instrumented release 빌드는 각각6m20s/6m21s exit0, 비용 diagnostic dev build는1m16s exit0이다. 빌드 통과를 기능 시험 수로 세지 않는다.

## 비용 비교와 자체 검토 종료

`cost001.log` exit0, 정상/확장 × forward/Continue ×3반복=12개 fresh diagnostic process PASS. 실제 publish된 Stage2_hatched payload는 base64 decode만 했으며 필드/checksum을 편집하지 않았다. forward는 각각333명령이며 기본 runtime 한도·기존 콘텐츠·3/64후보 규모를 유지한다. 현재 결과 원본은 `runs/current001/cost`, 전후 비교는 `COST_REVIEW.json`이다.

| diagnostic 중앙값 | 변경 전 | 변경 후 |
| --- | ---: | ---: |
| 정상 forward333 연산 시간 | 1.3245s | 1.2602s |
| 확장 forward333 연산 시간 | 1.3044s | 1.2802s |
| 정상 public Continue | 60.171ms | 44.023ms |
| 확장 public Continue | 59.156ms | 46.543ms |
| Continue 추가 heap peak | 904258bytes | 804211bytes |
| Continue native work | 1481111 | 677650 |
| Continue 누적 예약 최대 | 6287138bytes | 2637412bytes |

12개 모두 gameplay requested bytes가 기준점으로 정확히 회수됐다. loader-associated128bytes와 OS RSS retention은 별도 측정이며 전체 프로세스0잔여로 확대하지 않는다. forward는 trace parse를 포함한 큰 process baseline과 단일 gameplay window를 구별한다. 진단 dev opt0/debug0 측정이므로 release latency 보장으로 해석하지 않는다. 현재 native 계량은 비유물 저장까지 포함하므로 회계 범위와 실제 시간/heap의 변화도 구별한다. 기본 한도 상향은 없다.

코드·호출·owner 경계·정책 표·기대값은 `REVIEW.ko.md`에서 직접 대조했다. 단독 Arc 이동만의 개선이라는 초기 설명은 최종 cached SavePoint의 공유 복사와 구별했다. 모든 기능/비용 실행 종료 후 `state.rs`의 해당 주석3줄만 최종 설명에 맞췄으며 실행 코드·시험·콘텐츠·설정은 바꾸지 않았다. 이 주석과 runner의 결과 경로/metadata 정리는 기능 재실행 사유가 아니다. 7개 Python runner syntax 검사와 변경 파일의 diff whitespace 검사도 통과했다.

최종 고정은 `test_parts/finalize.py`가 실제 preimage `BASELINE*.tar.gz/json`, 이전 U17의 변경 없는 source, 후속 source/증거를 결속한다. 격리 tree에서 정방향·역방향의 전체 member/bytes 일치를 확인하며 작업 tree에 patch를 재적용하지 않는다. 고정 결과·판정은 `target/u17_storage_001/{AUDIT_TARGET_RECORD,SOURCE_MANIFEST,EVIDENCE_MANIFEST,RESTORE}.json`으로 확인한다. 감사와 U18은 수행하지 않는다.
