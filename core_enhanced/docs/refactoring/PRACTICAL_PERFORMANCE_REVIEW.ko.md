# 저사양 PC 단일 플레이 관점의 성능 영향 판단

작성·갱신일: 2026-10-18

**현재 근거로 긴급하거나 필수적인 성능 최적화를 선정하지 않는다.** 기존 기준 초과는 사실이지만,
저사양 PC의 단일 플레이에 큰 영향을 준다는 증거는 없다. 영향이 없다는 보장도 아니다.
S10-P2 추가 계측, S01-P3 서버 표현 변환 개선과 S01-C1 응답 축소는 후순위로 둔다.
동작·구조 검토를 완료한 S01-P2는 유지하고, 지연 개선·비퇴행이 입증되지 않았다는 기록도 보존한다.

## 1 범위와 근거의 유효성

사용자는 기준 초과만으로 결함을 판정하지 말고 실제 영향을 판단하도록 지시했으며,
이번 확인에서 **저사양 PC 단일 플레이 우선**을 선택했다. 정확한 최소 CPU/RAM, 목표 프레임,
서버를 같은 PC에서 실행하는지 여부는 지정되지 않았다. 이를 임의의 사양이나 배포 구성으로 확정하지 않는다.
[기준 §7.4](../refactoring_criteria.ko.md#74-성능과-자원)와 [실행 지침 §5.3–5.4](../system_refactoring_execution_guide.ko.md#53-검증-범위와-근거-재사용)를 적용한다.

[재사용 대조](evidence/practical_impact_001/reuse.json)에서 다음을 확인했다.

- X01-P2의 제품·도구·build 입력 411개와 S01-P2 구현/검증 입력 4개가 현재 파일과 일치한다. 중복을 포함한 목록별 개수다.
- S01-P1 소비 경계 입력 13개 중 11개는 동일하고, 변경된 world 파일 2개는 S01-P2의 검증된 새 hash와 일치한다.
  서버 발행 경로와 Unity reader 파일도 동일하다. 이전 전체 JSON 동등성과 영향 테스트 7개 기록을 재사용한다.
- 직전 X01-P2 이후 달라진 기준·실행 지침은 사용자 지시에 따른 판단 변경이다. 이전 기준으로 새 결론을 자동 결정하지 않는다.
- snapshot 101개와 전투 두 부하의 각 3회×21개 원자료에서 p95·출력 identity를 다시 계산했다.
  S01-P2의 CPU 고정 6쌍 비교도 기존 원자료와 일치한다. **새 벤치마크·컴파일·기능 테스트는 실행하지 않았다.**

기존 환경은 Linux/WSL2 release이며, 전투 측정은 32 logical CPU 환경에서 8 lane affinity를 사용했다.
현재 호스트는 AMD Ryzen 9 7950X로 확인했다. 과거 receipt에 CPU 모델명이 없으므로 이를 과거 실행의 추가 보증으로 쓰지 않는다.
현재 호스트를 제한된 코어 수로 실행하거나 수치를 임의 배수로 늘려 저사양 PC 실측을 대신하지 않는다.

## 2 실제 사용 경로와 수치의 의미

| 항목 | 확인한 사실 | 저사양 단일 플레이 관점의 판단 |
| --- | --- | --- |
| Run snapshot 생성+직렬화 | S01-P2 이후 일반 guard의 median 0.746382ms, p95 1.021362ms. 6쌍 비교의 after p95는 1.141882–1.500759ms | 현재 측정 환경에서 한 번의 비용은 작다. 대상 PC의 시간·실제 명령 빈도·host 및 client 비용은 미측정이므로 체감 영향이 없다고 보장하지 않음 |
| compact JSON 크기 | 117,922 bytes = 약 115.16KiB. 기존 65,536-byte guard 초과 | 단일 응답 크기만으로 PC 메모리 결함을 판단할 근거 부족. Value·DTO heap/RSS, 전송 중 사본, client 보관량과 다름 |
| snapshot 발행 | 접속/재접속, 일반 변경 명령, 전투 종료에서 전체 발행. 전투 명령은 별도 update/resync, ReadOnly는 전체 발행하지 않음 | 전체 응답이 매 화면 프레임·매 전투 tick 만들어진다는 가정을 기각. 실제 사용자의 초당 명령 수를 측정한 것은 아님 |
| host 전투 tick | 기본 interval 100ms이며 설정 가능. 일반 tick은 battle_update를 내고 종료 시 전체 snapshot을 추가 | 10Hz는 기본 host 설정이지 Unity 렌더 주기나 목표 성능 보장이 아님. tick의 전체 지연은 현재 Core guard에 포함되지 않음 |
| 64-session 부하 | 8 lane에서 세션마다 100×10ms, 즉 시뮬레이션 1초 진행. 전체 p95 11.789399–13.473684ms. 초기화·step·최종 로그 직렬화 포함 | 단일 플레이의 1 frame 또는 1 tick 지연이 아님. 64·100·8로 나눈 값을 단일 플레이 p95로 쓰지 않음. 이 초과만으로 AttackStart 최적화를 강제하지 않음 |
| 단일 event-heavy 부하 | 시뮬레이션 5초를 한 호출로 진행하는 커널 fixture의 p95 17.705238–20.852338ms | 단일 전투 근거로는 더 가깝지만 현재 콘텐츠의 live host tick·버스트·렌더링·저사양 입력 지연을 측정하지 않음. 5초에 균등 분산된 프레임 비용으로 환산하지 않음 |

서버 근거는 [actor 기본 설정·read_snapshot](../../../game_server/src/game/player_game_actor/mod.rs),
[명령 발행 매핑](../../../game_server/src/game/player_game_actor/state.rs)의 `behavior_result_to_command_result`,
[handlers](../../../game_server/src/game/player_game_actor/handlers.rs)의 `build_state_snapshot`·`ensure_live_battle_tick`·`tick_live_battle`다.
커널 측정 범위는 [기존 도구](../../tools/runtime_architecture_final_validation/src/final_performance.rs)의
`concurrent_sample`·`event_heavy_sample`와 대조했다. event-heavy의 log 직렬화는 timer 밖이고 concurrent는 안이다.
서버가 원격이면 Core 비용을 client PC의 CPU 비용으로 직접 귀속할 수 없으며, 같은 PC에서 실행해도 합산 end-to-end 비용은 미측정이다.

[기존 allocator 결과](PERFORMANCE_RESOURCE_MEASUREMENT.ko.md#고정-전투-부하)의 concurrent peak 약 9.7MB는
64세션·직렬화 결과를 포함한 요청 heap 관측이다. 단일 플레이 RSS나 누수·최대 콘텐츠 보장으로 사용하지 않는다.
서버 소스의 다른 64KiB 제한은 측정 record·입력 문서 등의 별도 범위다. snapshot engineering guard를 wire 제한으로 간주하지 않는다.

## 3 동작과 설계 품질 판단

| 대상 | 판정과 이유 |
| --- | --- |
| 전체 snapshot의 책임 | 유지. 상태 변경 시 파생하고 전체 교체하는 현재 계약과 전투 증분 메시지가 분리돼 있다. 무효화 책임이 없는 전체 cache를 추가할 이익이 입증되지 않음 |
| 시설·지도·스킬 데이터 | 유지. 시설 공개 조건, 현재/방문방 원본, 지도별 preview 차이와 전체 skill catalog 소비 의미가 있다. 크기 기준만 맞추기 위한 필드 삭제·겉보기 중복 통합은 하지 않음 |
| S01-P2 borrowed resolver | 유지. immutable publication의 검증 원본은 하나이며 읽기 호출의 소유 사본을 제거했다. 동작/설계 검토 완료와 성능 개선 미입증을 구분. 최초 지연 증가·혼합된 6쌍 결과를 지우거나 추가 반복의 자동 사유로 삼지 않음 |
| S01-P3 서버 Value→DTO→Value | 실제 중간 변환 후보지만 현재 Core 1ms 측정 밖이다. typed 경계 개선은 의미가 있을 수 있으나 host overlay·fault·selected_event 보존 비용과 함께 판단해야 함. 현재는 필수 수정으로 선정하지 않음 |
| X01/S10-P2 사건 처리 | AttackStart 34.1%는 계측한 마지막 lane step의 공통 전후 처리 포함 비중이다. 절감 가능한 시간이나 구조 결함 자체를 증명하지 않음. 원자성·source/ticket·순서·실패 prefix를 유지하고 세부 계측/수정은 후순위 |

이번 확인에서 새 동작 오류를 재현하지 않았다. 이는 새 게임플레이 실행을 했거나 모든 소비자 정상 동작을 증명했다는 뜻이 아니다.
기존 Unity SkillCatalog의 v2 reader / Core v3 생산 불일치는 별도 계약 문제로 남는다. 성능 판단 변경으로 이를 면제하지 않는다.

## 4 후순위 항목의 재개 조건

다음 중 실제 근거가 생기면 해당 경계만 재개한다. 조건 확인 전 큰 fixture 확대나 전체 재측정을 수행하지 않는다.

- 대상 저사양 PC에서 단일 플레이의 화면 진입/명령 응답/전투가 지연되거나 host tick 처리가 누적된다.
  같은 사용자 행동의 Core·host 변환/저장·전송·client 파싱/렌더 중 시간을 먼저 구분하고, 유의미한 부분의 최소 후보를 고른다.
- 실제 전체 snapshot의 발행 빈도·보관 수·크기가 커져 통신 또는 heap/RSS 부담이 확인된다. 이때 S01-C1/P3를 함께 판단한다.
- 공식 콘텐츠의 유닛·사건·자원 규모나 운영 환경이 크게 바뀐다. 바뀐 대표/최대 경로에 필요한 증거만 추가한다.

저사양 지원을 최종 보장할 때에는 목표 장치·실행 구성과 대표 플레이의 확인이 필요하다. **이 미확인은 현재 구조 정리의
완료를 자동으로 막거나, 이미 확보한 원자료를 반복 생성할 이유가 아니다.** 과거 P01 출력 비교 불가·guard 실패는 그대로 남긴다.

## 5 완료 검토와 다음 행동

이번 완료 범위는 유효한 기존 근거와 실제 발행 경로에 기반한 **성능 영향의 해석·개선 우선순위 판단**이다.
저사양 성능 인증, 기존 guard 통과, 성능 개선 또는 모든 경로의 비퇴행 판정이 아니다.
[읽기 전용 분리 검토](evidence/practical_impact_001/review.json)는 같은 작성자의 별도 pass이며 독립 감사가 아니다.
제품·도구·cap·DTO는 변경하지 않았고 문서와 재사용 대조 기록만 추가했다.

**다음은 S01-C2: Unity SkillCatalog reader의 v2/v3 불일치가 현재 사용 경로에 도달하는지와 유효 계약을 확인하는 읽기 전용 조사**다.
기존 세 파일·Core 생산자 근거부터 재사용하며 외부 프로젝트 수정이나 schema 변경을 자동으로 시작하지 않는다.
현재 실행 경로가 아니면 근거와 함께 후순위로 종료한다. 실제 불일치면 영향받는 최소 계약·수정 후보·검증 범위를 정한다.
성능 세부 계측은 위 재개 조건이 확인될 때 선택한다.
