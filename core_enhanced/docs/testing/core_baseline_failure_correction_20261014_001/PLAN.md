# 기존 Core 실패 138개 교정

2026-10-14 사용자 승인: 앞선 138개 진단의 유지·통합·부분제거 판단에 따라 모두 교정한다. 단독 실행. 기존 dirty 작업과 동결 U14 원문을 보존한다. 새로운 정식 Goal이나 U15 착수는 아니다.

- 현재 source 713개와 직전 진단의 일치 확인을 재사용한다. 수정 파일의 최초 bytes를 preimages에 보존하며 작업 delta만 검토한다.
- 정상 current Expedition/지원 authored content로 시작·역할·노드 난이도/활동·저장 자격을 준비한다. 공식 경로를 구형 mode로 치환하지 않는다. 내부 단위 fixture와 공개 외부 RON/Trial 경로는 구분한다.
- 135개 유지 보정, 시작 장비 검사1개는 고유 assertion 통합 후 중복 제거, Continue preview1개는 정상 경로 유지/저장 위조 분기 제거, socket1개는 환경 분리 후 지원 환경에서 실행한다.
- 초기 기계적 계약 보정과 공통 fixture 교정 후 기존138 선택으로 후속 실패를 수집한다. 실패 지점이 바뀌면 원래 목표·진입 조건을 확인하며 제품결함과 test 결함을 다시 구분한다.
- Trial은 실제 공개 시작/시설 진행을 기록하고 명시 명령과 binding/receipt 경계를 유지한다. 실제 자원 부족/fault/clock/새프로세스 결과는 의미를 완화하지 않는다.
- 선택 검사 빌드 약1–3분, 실행 수십초 예상. 첫60초에 기존 runner가 로그/자식 상태를 남긴다. 5분 초과 또는 새실패/무진행 증거에만 진단한다. 실행 중 입력은 고정한다.
- 영향 선택 통과 후 전체 Core lib/all-targets와 필요한 feature/외부CLI 소비를 검증한다. 원138 대응표, 고유 assertion 보존, 원본 대비 diff 및 실제 결과를 기록한다. 환경 실패·skip·제거를 PASS로 계산하지 않는다.

## 교정 중 확인한 준비 계약

- `correction138_001`: 60 통과/78 실패. `correction_remaining_004`: 그 78개 중 48 통과/30 실패. 중간 002/003은 컴파일 실패이며 실행 통과 증거가 아니다.
- 외부 Trial은 현재 Expedition의 작성된 FacilityLayout으로 시작한다. 기존 시나리오의 전투·보상·이벤트 슬롯 이름을 유지하는 작은 테스트용 시설을 정상 RON 검증으로 적재한다. 현재 역할별 시작 장비/파편, 시설 난이도, 고정 조우 은폐 규칙을 유지한다.
- U13은 첫 방문의 고정 조우를 입장 전 미리보기에서 숨긴다. `preview_probe_006`의 공개 결과로 `combat_preview:null`을 확인했다. 배치 좌표는 실제 입장 명령의 `battle_setup_snapshot`에서 읽고, 바인딩도 해당 기록을 참조한다. 입장 전 공개 정보를 노출하도록 제품을 바꾸지 않는다.
- 내부 단위 시험의 강제 노드 helper는 현재 activity 지원 여부도 확인한다. 공식 미지원 activity metadata가 남은 노드를 이름만 바꾸어 진입하지 않는다. 기존 내부 authored graph 설치 분기를 사용하며, 공개 외부 Trial 검증을 이 내부 준비 경로로 대신하지 않는다.
- 현재 시설의 Gate 확인은 방 이동과 별개다. 층 전이 관측은 Gate → 필요한 Continue 선택 → MoveToFacilityRoom을 명시적으로 실행한다. 층수만 바꿔 방 그래프와 모순되는 저장 상태를 만들지 않는다.
- 역할별 active 파편이 무기 변경을 제한하므로 사본 이전/새 사본 장착 전에 해당 active를 공개 해제 명령으로 해제한다. 같은 정의의 실제 두 사본 시험은 두 번째 사본을 지원 AdminGrantEquipment로 획득하고 장착한다. 같은 사본과 서로 다른 사본의 resource key oracle을 유지한다.

## 최종 검증 해석

- 전체 lib `full_core_016`: 2,092 통과·4 실패·기존 ignore6·socket 별도1. 실패4는 G11의 구형 기본 manifest를 읽은 결과다. U14에서 사용하던 현재 `test_parts/g11_inputs001/manifest.json`을 명시한 `final_focus_019`에서4개 전부 통과했다. 원 manifest나 동결 근거는 수정하지 않았다.
- `final_focus_019`는 Continue 재계산 사례도 함께 통과했다. 실행 전 fixture 정책의 정상 배율을110으로 작성하고, 저장에서 생략된 배율이 기본값으로 읽혔다가 실제 입장 시110으로 재계산되는 것을 명시적으로 검사한다. 이전 전체 결과는 다른 함수에 비영향 재사용한다.
- `socket_supported_017`은 샌드박스 UnixListener bind EPERM 때문에 허용된 실행 환경에서1/1통과했다. ignore/환경거절을 PASS로 대신하지 않았다.
- 기존138은 현재139개 대응 사례가 모두 통과했다. 중복 시작 장비 본문1개를 U06 검사에 통합했고, socket 복합 사례1개를2개로 분리했으므로 lib 총 개수는2,103으로 유지된다.
- 외부CLI R07/R12 6개는 기본프로필, 확장 U09 6개는 기존 `config/runtime_safety_authoring_dev.ron`을 명시해 모든 step과 amount/trace 소비를 확인했다. U09 기본프로필 작업량 거절도 원문 보존한다. 이를 기본프로필 성공 또는 운영한도 승격으로 해석하지 않는다.
