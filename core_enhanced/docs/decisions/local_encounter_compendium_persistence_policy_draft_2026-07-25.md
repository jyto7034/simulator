# 로컬 영구 조우 도감 저장·복구 정책 초안 — 2026-07-25

- 논의·작성일: 2026-07-25
- 분리일: 2026-07-25
- 마지막 정책 검토: 2026-07-27
- 상태: **Policy closed / 사용자 합의 완료 / P02 구현 완료 (2026-07-27)**
- 문서 등급: 구현 이력·정책 근거 문서
- 분리 출처:
  [save_point_trauma_recovery_policy_draft_2026-07-25.md](save_point_trauma_recovery_policy_draft_2026-07-25.md)
- 주의: 현재 runtime, host persistence와 wire의 source of truth는 `archive/game_rulebook_legacy_2026-10-04.md`,
  `core_runtime_contract.md`, `data_loading_contract.md`, game_server bridge 문서와 외부 Unity canonical이다.
  이 문서는 P02가 구현한 승인 정책의 역사적 입력으로 유지한다.

## 1. 목적과 범위

환상체·침식 도감을 run save가 아니라 해당 컴퓨터에 귀속되는 local persistent profile file로
저장한다. 플레이어가 여러 run에서 실제로 겪은 조우 지식이 설치 환경에 계속 누적되는 구조다.

현재 profile scope는 OS application persistent-data 경로를 사용하는 로컬 사용자·설치 단위 하나다.
Standard/Endless와 모든 run이 같은 도감을 공유하며 server transport의 `player_id`나 active-run save별로
도감을 나누지 않는다. 향후 게임 내부 다중 프로필을 제품 기능으로 추가할 때만 profile namespace와
이관 정책을 별도로 결정한다.

이 문서는 다음을 결정한다.

- 도감과 run/checkpoint의 persistence 경계
- canonical 발견/격파 ID set, authored placeholder projection, dormant ID와 migration 원칙
- host/game server와 Core, Unity 사이의 authority
- atomic current file 교체와 단일 rolling backup
- 시작 시 복구와 두 파일 동시 손상 처리
- 저장 실패 후 in-memory 유지, dirty retry와 사용자 알림
- 명시적 전체 초기화와 cloud/account 비목표

실제 조우 ID의 발견·격파 eligibility는 `archive/game_rulebook_legacy_2026-10-04.md`와
[elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md](elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md)가
소유한다. 이 문서는 두 상태가 commit된 뒤 local profile에 보존하고 복구하는 방법을 소유한다.

## 2. 데이터와 authority

- canonical abnormality/corrosion profile ID의 `discovered`와 `defeated` set을 저장하고 이름·위험
  등급, authored 도감 순서와 placeholder metadata 같은 표시 정보는 현재 live catalog에서 해결한다.
- `defeated`는 항상 `discovered`의 부분집합이다. profile load, merge와 저장 전 validation이 이를
  보장하며 불일치 상태를 조용히 보정하지 않는다.
- profile format과 ID 형식의 무결성 검증은 현재 catalog membership과 구분한다.
- 콘텐츠 업데이트로 현재 catalog에서 사라진 과거 ID는 파일 손상으로 취급하거나 삭제하지 않고
  dormant ID로 그대로 보존한다.
- Unity projection은 현재 catalog가 저작한 전체 도감 순서를 사용한다. 미발견 현재 항목은 이름,
  이미지, 설명, 위험 등급과 기타 semantic metadata를 숨긴 placeholder로 표시하고, 발견 항목은
  현재 metadata를 표시하며, 격파 항목은 별도 완료 표식을 가진다. Placeholder의 authored 순서와
  빈 자리 자체는 표시하므로 현재 보스 도감의 숨겨진 네 번째 항목처럼 존재를 암시하는 콘텐츠를
  지원한다. 과거 dormant ID를 다시 지원하면 기존 발견/격파 상태가 자동으로 나타난다.
- ID rename이 필요하면 명시적인 profile migration table을 사용한다.
- 커밋된 발견과 격파는 각 set union으로만 누적한다. checkpoint load, 새 run, Run Failed와 일반 run
  rollback은 이미 기록된 상태를 삭제하지 않는다.
- 실패를 일으킨 전투 결과에서 발견만 commit된 뒤 checkpoint를 불러와도 해당 발견은 local
  profile에 남으며 격파 표식은 추가하지 않는다.

Core는 발견/격파 ID의 검증·union과 snapshot projection authority를 유지한다. host/game server가
시작 시 local profile file을 읽어 Core에 주입하고, 변경된 profile을 파일에 영속화한다. Unity가
battle event나 local catalog를 읽어 별도의 발견·격파 상태를 만들거나 직접 병합하지 않는다. 새로
commit되는 ID는 현재 runtime catalog에 존재해야 하지만 host가 주입한 과거 dormant ID는 삭제하지
않는다.

## 3. Run과 파일 경계

- local profile file은 `RunState`, `RunCheckpointPayload`와 `BattleEntrySnapshot`에 포함하지 않는다.
- 파일은 OS별 application persistent-data 경로에 둔다.
- project/work directory나 StreamingAssets를 사용하지 않는다.
- versioned envelope 안에 compendium payload를 저장해 지원하지 않는 format을 조용히 다른 schema로
  읽지 않는다.
- [save_point_trauma_recovery_policy_draft_2026-07-25.md](save_point_trauma_recovery_policy_draft_2026-07-25.md)의
  active-run file과 이 profile file은 별도 canonical이다. 이어하기는 시작 시 최신 local profile을
  Core에 주입한 뒤 active-run checkpoint를 복원하며, active-run payload 안의 stale 도감 copy로
  profile을 덮어쓰거나 병합하지 않는다.
- 두 파일의 write는 서로 다른 transaction이다. Compendium write 실패는 active-run save를
  rollback하지 않고, active-run save 실패도 이미 정상적으로 저장된 compendium을 되돌리지 않는다.

## 4. Atomic 저장과 rolling backup

저장은 canonical current file과 같은 디렉터리의 단일 rolling backup을 사용한다.

1. 메모리의 최신 전체 발견/격파 set을 별도 temporary file에 serialize하고 다시 읽어 envelope/schema,
   checksum과 ID 형식을 검증한다. 현재 catalog에 없는 dormant ID도 유효한 set 구성원이다.
2. 기존 canonical file이 정상적으로 읽히고 검증된 경우에만 그 파일을 이전 세대 backup으로
   원자적으로 교체한다. 손상되거나 부분 기록된 current file을 정상 backup 위에 덮지 않는다.
3. 검증된 temporary file을 canonical file로 원자적으로 교체한다. platform별 filesystem API를
   사용해 중간 바이트가 canonical 경로에 노출되지 않게 한다.
4. canonical 교체가 성공한 뒤에만 저장 성공으로 보고 `dirty`를 해제한다. backup은 직전의 마지막
   정상 canonical 세대를 유지한다.

Profile file은 single-writer resource다. Host는 한 프로세스 안에서 쓰기를 직렬화하고, 동시에 실행한
다른 프로세스가 같은 canonical을 경쟁적으로 갱신하지 못하게 platform lock 또는 동등한 배타적
ownership을 사용한다. Lock을 얻지 못하면 last-writer-wins 병합을 시도하지 않고 profile bootstrap을
명시적인 system error로 중단한다.

## 5. 시작 시 복구

시작 시 복구 순서는 다음과 같다.

- canonical file이 정상이라면 이를 사용한다.
- canonical file이 없거나 손상되었지만 backup이 정상이면 backup을 자동으로 불러오고 플레이어에게
  복구 사실을 비차단 알림으로 정확히 한 번 보고한다.
- backup 복구 뒤 profile을 `dirty`로 두어 다음 저장 때 canonical을 재생성하며, 손상된 canonical을
  backup 위에 회전시키지 않는다.
- 재생성 성공은 추가 성공 알림을 만들지 않고 상태만 정상으로 되돌린다. 재생성도 실패하면 영속
  저장 실패를 한 번 보고한다.
- 상세 파일 경로와 parsing 진단은 사용자 메시지가 아니라 로그에 기록한다.
- canonical과 backup이 모두 없는 첫 실행 또는 명시적 초기화 뒤에는 빈 도감으로 시작한다.
- 하나 이상의 파일이 존재하지만 canonical과 backup이 모두 읽기·schema·무결성 validation에
  실패하면 빈 도감으로 조용히 덮어쓰거나 profile-disabled gameplay mode로 진행하지 않는다.
- 두 파일이 모두 손상된 경우 원본들을 보존하고 단일 오류 화면에서 다시 시도, 명시적인 전체 도감
  초기화와 종료를 제공한 뒤 하나가 선택될 때까지 게임 시작을 중단한다.
- 오류 화면은 진단 reference와 버그 리포트 안내를 제공한다.
- 현재 catalog에 없는 dormant ID만으로 이 경로에 진입하지 않는다.
- 지원 범위보다 새로운 envelope version도 손상과 구분되는 `unsupported_version` 오류로 원본을
  보존하고 시작을 중단한다. 구 reader가 새 파일을 빈 profile이나 구 schema로 덮어쓰지 않는다.

## 6. 저장 실패와 dirty retry

- local profile file write 실패는 이미 커밋된 전투·노드 결과나 현재 run을 실패시키거나 rollback하지
  않는다.
- host는 현재 세션의 전체 compendium 발견/격파 set을 메모리에 유지하고 profile을 `dirty`로
  표시한다.
- Unity는 저장 실패와 아직 파일에 반영되지 않은 상태임을 비차단 방식으로 한 번 알린다.
- 같은 오류를 매 frame/modal로 반복하지 않고 clean/dirty 또는 오류 종류가 실제로 바뀔 때만 상태를
  갱신한다.
- 다음 정상 local profile save 기회에는 개별 누락 항목이 아니라 메모리의 최신 전체 발견/격파 set을
  다시 durable write한다. union은 idempotent하므로 재시도로 중복 항목이 생기지 않는다.
- 재시도가 성공하면 그 성공 시점까지의 두 set이 파일에 반영된 뒤에만 `dirty`를 해제한다.
- 다시 실패하면 gameplay를 계속하고 `dirty`와 사용자-visible 저장 경고를 유지한다.
- 애플리케이션 종료 전에 durable write가 끝내 성공하지 않으면 이번 세션의 미저장 발견이 유실될
  수 있으므로 저장 실패 알림은 이 가능성을 숨기지 않는다.

Host는 발견 또는 격파 set의 성공한 union 직후 최신 전체 profile의 durable write를 한 번 시도한다.
실패 뒤에는 다음 profile mutation, 명시적 안전 retry 지점과 정상 종료 전에 다시 시도하되 매 frame
busy retry하지 않는다. 정상 종료는 dirty profile의 bounded flush를 시도하지만 gameplay commit을
되돌리거나 애플리케이션을 무기한 붙잡지 않는다.

## 7. 초기화와 동기화 경계

- 명시적인 전체 local profile/도감 초기화는 확인 절차 뒤 canonical, backup과 메모리 발견/격파
  set을 함께 비운다.
- 초기화는 active run이 없는 title/profile 관리 흐름에서만 제공한다. 초기화 성공은 검증된 빈
  canonical envelope publication과 backup 제거를 하나의 관리 작업으로 완료하며, 중간 실패 시 기존
  정상 profile 또는 복구 가능한 원본을 보존한다.
- canonical 하나만 수동으로 지우면 남은 backup을 복구 대상으로 취급한다.
- 계정 동기화, cloud save와 여러 컴퓨터 사이의 자동 병합은 이 정책 범위에 포함하지 않는다.

## 8. 구현 전 동기화 대상

독립 goal에서 최소 다음 표면을 함께 감사하고 갱신한다.

- `archive/game_rulebook_legacy_2026-10-04.md`: 영구 환상체·침식 도감의 persistence 의미와 checkpoint 경계
- `core_runtime_contract.md`: 발견/격파 commit, Core injection/projection과 run rollback 경계
- Core compendium discovered/defeated state, result commit, authored placeholder와 snapshot projection
- host/game server의 versioned local profile bootstrap, atomic write와 rolling backup
- dormant ID 보존, migration table과 current catalog resolution
- backup 복구 알림, dirty retry와 fatal dual-corruption reporting
- 외부 Unity `unity_core_contract.md`: profile status와 사용자-visible notification projection
- 외부 Unity UI 계약: 복구 알림, 저장 실패 경고와 fatal recovery 화면
- host persistence test, corruption/recovery test, server mapping/probe와 Unity test

## 9. 비목표

- 실제 전투에서 발견/격파 ID를 commit하는 gameplay eligibility 변경
- account 또는 cloud save
- 여러 컴퓨터 사이의 자동 병합과 conflict resolution
- 현재 catalog에서 제거된 dormant ID의 자동 삭제
- 도감 데이터를 run checkpoint에 포함하는 기능
- write 실패 때문에 이미 commit된 gameplay 결과를 rollback하는 기능
- 파일 위·변조를 막는 anti-cheat, 암호화 또는 계정 서명. Checksum은 우발적 손상과 부분 기록을
  검출하기 위한 무결성 수단이며 경쟁 플레이의 보안 경계가 아니다.

## 10. 승인 후 완료 조건

1. 로컬 도감 persistence 전용 영향 감사와 독립 구현 goal을 작성한다.
2. durable canonical 문서와 외부 Unity 계약을 같은 authority 구조로 갱신한다.
3. versioned envelope, checksum, migration과 OS별 storage path를 구현한다.
4. atomic current/backup 교체와 시작 복구를 구현한다.
5. Core injection/projection, dirty retry와 Unity 알림·복구 UI를 구현한다.
6. 정상 저장, current 손상, backup 복구, dual corruption, write retry와 dormant ID를 검증한다.
7. run/checkpoint에 local profile data가 포함되거나 Unity가 발견/격파 set을 재구성하는 경로가 없는지
   감사한다.
8. 완료 review 뒤 이 문서를 historical decision input으로 전환한다.

## 11. 다른 작업자에게 넘길 때의 기준

이 문서의 2~7절은 합의된 target policy다. 현재 runtime, host persistence와 기존 canonical이 다르게
동작하는 부분은 이 결정을 무효화하는 근거가 아니라 구현 시 함께 전환해야 할 current-state 차이다.

- 도감 local profile의 발견/격파 set은 run/checkpoint와 분리하고 격파 set은 발견 set의 부분집합이다.
- atomic current/backup 교체, backup 복구와 dirty retry를 host persistence 경계에서 구현한다.
- 현재 catalog에 없는 dormant ID를 손상으로 취급하거나 자동 삭제하지 않는다.
- write 실패는 gameplay 결과를 rollback하지 않으며 최신 전체 in-memory set을 재시도한다.
- current와 backup이 모두 손상되면 조용한 초기화 대신 원본을 보존하고 명시적 사용자 결정을 받는다.
- Unity가 battle event나 local catalog로 발견/격파 상태를 별도 재구성하거나 병합하지 않는다.
- 현재 catalog의 authored 전체 순서와 placeholder 표시를 Core가 투영하며, 보스 도감에는 숨겨진
  흉탄의 사수를 포함한 미발견 자리도 남긴다.

현재 지원 항목은 모두 authored 순서의 placeholder 자리를 표시한다. Placeholder 자리 자체도 숨겨야
하는 future entry가 생기면 그 entry의 gameplay reveal 정책으로 별도 확장한다. 계정 동기화와 cloud
save는 별도 제품 정책이 필요한 후속 범위다.

정책 검토 결과 이 문서가 소유하는 local persistence·복구 경계에는 추가 사용자 결정이 없다. 실제
전투에서 어떤 ID를 `discovered`/`defeated`로 commit할지는 이 문서가 임의로 정하지 않고
`archive/game_rulebook_legacy_2026-10-04.md`와 모노리스·도감 gameplay 정책의 검토 결과를 따른다. 구현 감사에서 이 문서의
persistence 의미를 바꿔야만 해결할 수 있는 충돌이 확인될 때만 사용자 결정을 다시 요청한다.
