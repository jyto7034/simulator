# Current Runtime Structural Integrity Audit Criteria

## 1. 문서 목적

이 문서는 `core_enhanced`의 현재 Rust 런타임을 대상으로 구조적 건전성 감사를 수행할 때 사용하는
공통 기준이다. 이후 생성되는 감사 goal은 이 문서를 기준 문서로 참조한다.

이번 감사의 질문은 다음과 같다.

> 현재 런타임은 정책이 변경되더라도 안전하게 이해하고 수정할 수 있으며, 동일한 입력에 안정적인
> 결과를 내고, 실패 시 권위 상태를 보존하며, 향후 Unity 어댑터를 붙일 수 있는 구조인가?

이번 감사는 현재 게임 정책의 구현 완료 여부를 판정하지 않는다. 구조 감사가 끝난 뒤 최신 정책이
동결되면 별도의 정책 정합성 감사를 수행한다.

## 2. 문서 지위

- 문서 종류: 구조 감사 기준
- 적용 대상: 현재 작업 트리의 Rust 런타임
- 주 용도: 감사 goal의 범위, 판정 방식, 증거 요건, 완료 조건 고정
- 비목표: 구체적인 게임 정책 확정, 기능 구현, 리팩터링 실행, 기존 Unity 호환성 인증

감사자는 개별 goal에서 이 기준을 완화하거나 새로운 합격 의미를 임의로 추가하지 않는다. 기준 변경이
필요하면 이 문서를 먼저 수정하고 변경 이유를 기록한다.

## 3. 감사 범위

### 3.1 기본 포함 범위

- `src/lib.rs`
- `src/game/**`
- `Cargo.toml`과 런타임 의존성·feature 구성
- 런타임을 검증하는 `tests/**` 및 `src/**` 내부 테스트
- RON 로더와 검증 경계를 판단하는 데 필요한 데이터 타입 및 로딩 코드
- 저장, snapshot, command, event, DTO 직렬화 경계
- 런타임에서 호출하는 파일 I/O, 시간, RNG, UUID, 물리 엔진 경계

테스트와 데이터는 런타임 동작을 증명하거나 반증하기 위한 보조 범위다. 테스트 수나 데이터 양 자체를
품질 증거로 사용하지 않는다.

### 3.2 기본 제외 범위

- `docs/goals/**`의 과거·현재 goal 문서
- 과거 감사 결과와 완료 선언
- 현재 `game_rulebook`과 코드의 정책 정합성
- 아직 갱신되지 않은 Unity 프로젝트의 구현 상태
- Unity와 기존 server가 현재 wire protocol을 정확히 소비하는지 여부
- 밸런스 수치, 콘텐츠 완성도, 플레이 재미
- 구조 감사 중 발견된 문제의 실제 수정

과거 goal이나 감사 문서는 후보 탐색에도 필수 자료로 취급하지 않는다. 우연히 같은 문제를 다시
발견하더라도 현재 코드에서 독립적으로 증명해야 한다.

### 3.3 제한적으로 허용되는 문서 사용

게임 개념을 이해하기 위해 rulebook이나 설계 문서를 읽을 수 있다. 단, 최신 정책과 코드가 불일치하는
상태이므로 문서와 다르다는 이유만으로 결함을 판정하지 않는다.

문서는 다음 용도로만 사용할 수 있다.

- 용어와 게임 흐름의 대략적인 이해
- 조사할 코드 진입점 탐색
- 타입이나 함수 이름의 의미 추정

문서만으로 `PASS` 또는 `FAIL`을 판정할 수 없다.

## 4. 감사 원칙

### 4.1 현재 실행 코드 우선

판정은 현재 작업 트리의 production 호출 경로를 기준으로 한다. 주석, 이름, 과거 계획, 완료 선언보다
실제 실행되는 코드가 우선한다.

### 4.2 정책과 구조를 분리

다음은 정책 불일치가 없어도 판정할 수 있는 구조 문제다.

- 하나의 상태를 여러 권위자가 소유함
- 실패한 명령이 일부 상태를 변경함
- 같은 입력이 실행 순서에 따라 다른 결과를 냄
- 손상된 입력이 정상 오류가 아니라 panic을 일으킴
- 저장과 복원 사이에 의미가 달라짐
- 내부 도메인 타입과 외부 wire 타입이 분리되지 않음

반대로 “이 스킬은 누구를 공격해야 한다”와 같은 의미는 최신 정책 없이는 결함으로 판정하지 않는다.

### 4.3 신호와 결함을 구분

다음 항목은 조사 신호이지 그 자체로 결함이 아니다.

- 큰 파일 또는 긴 함수
- 많은 `pub` 필드
- `clone`, `unwrap`, `expect`, `panic!`
- 많은 enum variant 또는 `match`
- 높은 테스트 수
- 낮은 주석 수
- 추상화 계층 수

결함으로 승격하려면 실제 호출 경로, 깨지는 불변조건, 실패 결과 또는 변경 증폭을 제시해야 한다.

### 4.4 증거 없는 합격 금지

코드를 읽고 문제가 눈에 띄지 않았다는 이유로 `PASS`를 부여하지 않는다. 조사한 표면과 합격을
뒷받침하는 정적·동적 증거를 함께 기록한다. 증거가 부족하면 `UNKNOWN` 또는 `PARTIAL`이다.

### 4.5 감사와 수정을 분리

구조 감사 goal은 기본적으로 read-only 조사다. 문제를 발견하는 즉시 production 코드를 고치면 기준선과
증거가 바뀌므로, 수정은 별도 remediation goal로 넘긴다.

재현용 probe가 필요하면 우선 `/tmp` 또는 감사 전용 임시 도구를 사용한다. 저장소에 테스트를 추가해야
한다면 그것이 감사 기준선을 변경한다는 사실과 변경 전 재현 증거를 명시한다.

## 5. 기준선 고정

감사 시작 시 다음을 기록한다.

- 감사 시작 시각과 timezone
- `git rev-parse HEAD`
- `git status --short`
- 포함 범위의 파일 목록 또는 content manifest
- 사용한 Rust toolchain과 Cargo 버전
- 활성화한 Cargo feature
- 실행한 build, test, lint 명령

작업 트리가 dirty이면 commit ID만으로 기준선을 식별할 수 없다. 이 경우 감사 대상 파일의 content hash
manifest나 이에 준하는 재현 가능한 식별자를 남긴다.

감사 도중 코드가 변경되면 다음 중 하나를 선택한다.

1. 기존 기준선에서 감사를 끝내고 변경분을 후속 감사로 분리한다.
2. 새 기준선을 선언하고 영향받는 판정과 재현 절차를 다시 실행한다.

서로 다른 기준선의 증거를 하나의 완료 판정에 섞지 않는다.

## 6. 증거 우선순위

증거는 대체로 다음 순서로 강하다.

1. 현재 트리의 production 진입점을 사용한 최소 재현
2. 현재 트리에서 통과하거나 실패하는 자동화 테스트
3. 완전한 호출 경로와 상태 변경 순서에 대한 코드 증명
4. 타입, 가시성, 의존 방향에 대한 정적 증명
5. 주석, 테스트 이름, 문서에 적힌 의도

하위 증거만으로도 조사 후보를 만들 수 있지만, 주석이나 문서만으로 확정 결함을 만들 수 없다.

## 7. 감사 기준 요약

| ID | 기준 | 핵심 질문 |
| --- | --- | --- |
| C00 | 실행 기준선과 기본 건전성 | 현재 트리가 무엇이며 기본 검사가 재현되는가? |
| C01 | 상태 소유권 | 하나의 사실에 하나의 권위자만 있는가? |
| C02 | 상태 전이와 불변조건 | 모든 전이가 유효 상태만 만들도록 보장되는가? |
| C03 | 원자성과 실패 안전성 | 실패한 명령이 권위 상태를 보존하는가? |
| C04 | 결정론과 시간 | 같은 입력이 같은 결과와 순서를 만드는가? |
| C05 | 의존 방향과 변경 용이성 | 작은 변경이 불필요하게 넓게 전파되지 않는가? |
| C06 | 외부 경계와 Unity 준비도 | 어댑터가 코어를 보호할 수 있는가? |
| C07 | 데이터 로딩과 검증 | 검증되지 않은 데이터가 런타임에 들어가지 않는가? |
| C08 | 저장과 복원 | durable state가 안전하게 왕복하고 진화할 수 있는가? |
| C09 | 실행 경로의 완결성 | 선언된 기능이 실제 진입점에서 도달 가능한가? |
| C10 | 오류와 관측 경계 | 오류·로그·I/O가 gameplay authority를 오염시키지 않는가? |
| C11 | 테스트 신뢰도 | 테스트가 실제 계약과 실패 경로를 검증하는가? |
| C12 | 성능과 자원 상한 | 비용과 상태 크기에 통제 가능한 상한이 있는가? |
| C13 | AI 생성 누적 부채 | 중복 의미와 방어 계층이 근본 원인을 숨기지 않는가? |

## 8. 세부 감사 기준

### C00. 실행 기준선과 기본 건전성

확인 항목:

- 현재 crate가 지정 toolchain과 feature 조합에서 build되는가?
- 전체 test 결과가 재현되는가?
- formatting과 lint 결과가 기록되는가?
- 경고를 suppress하는 설정이 문제를 숨기지 않는가?
- feature에 따라 컴파일되지 않거나 검증되지 않는 production 경로가 있는가?
- runtime dependency가 실제로 사용되고 책임 경계가 명확한가?

`cargo test` 통과는 감사 시작 조건에 가깝고 전체 품질 합격 증거가 아니다. 실패가 있으면 실패한 표면을
격리해 기록하고, 나머지 구조 감사를 가능한 범위에서 계속한다.

### C01. 상태 소유권

핵심 원칙:

> 하나의 gameplay 사실에는 하나의 권위자만 있어야 한다.

확인 항목:

- `GameCore`, `GameCoreState`, active run, active battle, `BattleCore`의 책임이 구분되는가?
- 같은 HP, cooldown, 배치, 자원, 진행 상태가 여러 곳에서 독립적으로 변경되는가?
- 캐시와 projection이 권위 상태처럼 사용되는가?
- 상태 변경이 명시적인 command 경계를 통과하는가?
- public field나 public mutable reference가 정상 경로를 우회하게 하는가?
- 파생 상태를 재계산할 수 있는 원본이 명확한가?
- 상태 소유권이 모듈 이름과 가시성에 반영되는가?

결함 예:

- 두 구조체가 같은 cooldown 값을 보유하고 각자 감소시킨다.
- projection DTO의 값이 다음 gameplay 판정 입력으로 다시 사용된다.
- caller가 `BattleCore` 내부 필드를 직접 수정해 event나 revision 갱신을 우회한다.

필수 산출물:

- 주요 권위 상태와 소유 모듈의 표
- 동일 의미가 중복 저장되는 후보 목록
- public mutation entry point 목록

### C02. 상태 전이와 불변조건

확인 항목:

- 유효한 상태와 금지된 상태 조합이 코드로 표현되는가?
- 명령의 사전조건을 mutation 전에 검증하는가?
- 생성자와 복원 경로가 동일한 핵심 불변조건을 보장하는가?
- enum과 `Option` 조합이 불가능한 상태를 표현하도록 허용하지 않는가?
- 수치 범위, ID 유일성, lifecycle, 점유 관계가 항상 보존되는가?
- terminal 상태가 다시 active 상태로 돌아갈 수 없는가?
- cleanup이 일부 인덱스나 역참조를 남기지 않는가?

정책과 무관하게 확인 가능한 대표 불변조건:

- 존재하지 않는 ID에 대한 dangling reference가 없다.
- 동일한 runtime identity가 동시에 두 객체를 지칭하지 않는다.
- dead, withdrawn, inactive 상태가 active occupancy와 충돌하지 않는다.
- current 값은 정의된 최소·최대 범위를 벗어나지 않는다.
- 양방향 인덱스와 점유 관계가 서로 일치한다.
- terminal outcome 또는 commit은 한 번만 확정된다.

### C03. 원자성과 실패 안전성

핵심 원칙:

> 권위 명령이 오류를 반환하면 명령 전과 후의 권위 상태는 의미상 동일해야 한다.

확인 항목:

- mutation 뒤에 실패 가능한 검증, 직렬화, projection, I/O가 남아 있는가?
- 자원 차감, UUID/sequence 소비, event 기록이 성공 commit과 함께 일어나는가?
- 여러 필드를 수동 rollback하면서 일부 필드를 누락하는가?
- clone 또는 staging이 완전한 transaction state를 포함하는가?
- nested command가 외부 transaction과 다른 commit 의미를 가지는가?
- 실패 후 재시도가 동일한 결과를 내는가?
- 중복 요청이 같은 효과를 두 번 적용할 수 있는가?
- overflow, future cursor, stale revision 같은 경계 실패가 부분 mutation을 남기는가?

권장 검증 형태:

```text
before = authoritative_snapshot(state)
result = execute(failing_command)
after = authoritative_snapshot(state)

assert result is error
assert before == after
```

단순 DTO snapshot이 권위 상태 전체를 포함하지 않으면 transaction 전용 debug snapshot 또는 의미상 동등성
검사를 사용한다.

### C04. 결정론과 시간

핵심 원칙:

> 동일한 초기 상태, seed, 명령열은 동일한 결과, event 순서와 최종 상태를 만들어야 한다.

확인 항목:

- `HashMap`·`HashSet` 순회 순서가 판정이나 출력 순서에 영향을 주는가?
- `BinaryHeap`의 서로 다른 event가 정렬상 동등해질 수 있는가?
- RNG identity가 log sequence, 관측 순서 또는 불안정한 카운터에 결합되는가?
- UUID 생성 순서가 gameplay 결과를 바꾸는가?
- Rapier world의 삽입·iteration 순서가 판정에 영향을 주는가?
- 부동소수점 값을 비교·정렬·직렬화하는 규칙이 안정적인가?
- 벽시계, thread scheduling 또는 filesystem 상태를 gameplay 판정에 사용하는가?
- 같은 timestamp의 event 처리 순서가 완전하게 정의되는가?
- pause, speed, elapsed-time 변환이 catch-up이나 중복 tick을 만들지 않는가?
- snapshot 복원 후 실행이 중단 없이 실행한 결과와 같은가?

필수 결정론 probe:

```text
run(initial_state, seed, commands) -> A
run(initial_state, seed, commands) -> B
canonicalize(A) == canonicalize(B)
```

가능하면 insertion order를 바꾼 동치 입력과 process를 다시 시작한 실행도 비교한다. 비교 전 정렬로 실제
순서 결함을 숨기지 않도록 canonicalization 대상은 명시한다.

### C05. 의존 방향과 변경 용이성

확인 항목:

- 상위 orchestration, domain, data, projection 책임이 구분되는가?
- battle leaf 로직이 world, DTO, persistence를 역으로 참조하는가?
- 하나의 개념이 여러 모듈에서 별도 타입으로 재정의되는가?
- enum variant 하나 추가할 때 무관한 계층까지 수정해야 하는가?
- 거대한 모듈이 여러 독립된 변경 이유를 동시에 가지는가?
- helper가 책임을 숨기거나 호출 순서를 암묵적으로 요구하는가?
- 내부 상세 구현이 과도하게 public으로 노출되는가?
- feature나 mode별 분기가 핵심 로직 전체에 흩어지는가?

변경 증폭을 판단할 때 단순 파일 수만 세지 않는다. 다음을 함께 기록한다.

- 수정해야 하는 표현의 종류
- 서로 다른 권위 계층의 수
- 순서를 맞춰야 하는 수동 동기화 지점
- 누락 시 compiler가 잡는지 여부
- 관련 없는 회귀 위험의 범위

### C06. 외부 경계와 Unity 준비도

이번 기준은 오래된 Unity 구현과의 호환 여부를 판정하지 않는다. 향후 Unity 어댑터가 domain을 보호할 수
있는 구조인지 평가한다.

확인 항목:

- command, query, event, snapshot, error가 구분되어 있는가?
- wire DTO와 내부 domain 타입이 분리되어 있는가?
- serde 속성이나 JSON shape가 핵심 판정 타입에 직접 결합되는가?
- 외부 입력이 내부 상태 객체를 직접 구성하거나 덮어쓸 수 있는가?
- protocol version과 호환성 변경 지점이 한 경계에 모이는가?
- full snapshot과 delta/event의 권위 의미가 구분되는가?
- command correlation, 중복 요청, stale revision을 처리할 수 있는가?
- 외부에 안정적인 identity와 ordering을 제공하는가?
- 내부 리팩터링이 불필요한 wire breaking change를 만들지 않는가?

목표 경계의 개념 형태:

```text
Unity
  <-> versioned DTO / protocol
Adapter
  <-> typed command / query / result
GameCore
  <-> domain operations
Run and Battle runtime
```

### C07. 데이터 로딩과 검증

확인 항목:

- 역직렬화 성공과 publication 가능 상태가 구분되는가?
- 검증 전 타입과 검증 완료 타입 또는 생성 경로가 구분되는가?
- 중복 ID, dangling reference, 빈 필수 목록, 잘못된 수치 범위를 거부하는가?
- 여러 파일 로딩 중 일부만 성공한 database가 공개되는가?
- unknown field와 legacy alias 처리 원칙이 일관적인가?
- default가 authoring 누락을 조용히 정상값으로 바꾸는가?
- 외부 데이터 오류가 정상적인 validation error가 아니라 panic을 만드는가?
- 검증 후 runtime이 같은 조건을 반복 검증하거나 다시 추측하는가?
- embedded fixture와 실제 live loader가 다른 경로를 사용하는가?
- validation 순서가 입력 순서에 따라 다른 첫 오류나 publication 결과를 만드는가?

핵심 합격 의미:

> 검증되지 않은 gameplay 데이터는 production runtime에 도달할 수 없으며, 잘못된 데이터는 경계에서
> 문맥이 있는 오류로 거부된다.

### C08. 저장과 복원

확인 항목:

- durable state와 process-local 또는 battle-temporary state가 구분되는가?
- 저장 포맷에 명시적인 version이 있는가?
- schema 변경과 migration 또는 명시적 거부 전략이 있는가?
- 저장 파일 생성이 atomic하거나 이전 정상본을 보존하는가?
- 손상, truncation, unknown field를 부분 성공으로 받아들이지 않는가?
- load 후 참조 무결성과 핵심 불변조건을 다시 검증하는가?
- cache, projection, derived state가 잘못 durable state로 저장되는가?
- 저장되지 않는 counter나 RNG 상태 때문에 복원 뒤 결과가 달라지는가?
- 같은 payload를 여러 번 load해도 중복 보상이나 중복 commit이 없는가?

필수 검증:

```text
state -> save -> load -> state'
continue(state, commands) == continue(state', commands)
```

직렬화 round-trip만으로 충분하지 않다. 복원 후 행동 동등성도 확인한다.

### C09. 실행 경로의 완결성

이 기준은 최신 정책 기능 목록과 대조하지 않는다. 현재 코드에 선언된 기능이 실제 production 경로에서
완결되어 있는지를 조사한다.

확인 항목:

- public command 또는 상태가 실제 root entry point에서 도달 가능한가?
- data definition이 runtime handler까지 연결되는가?
- handler가 실질적 mutation 없이 형식적인 성공만 반환하는가?
- 특정 state에 진입한 뒤 정상적인 종료·취소·복구 경로가 있는가?
- test helper에서만 가능한 동작이 production capability처럼 보이는가?
- 서로 다른 진입점이 같은 명령에 다른 검증·commit 의미를 제공하는가?
- 생성만 되고 소비되지 않는 event, signal, DTO field가 있는가?
- match의 wildcard나 fallback이 새 variant를 조용히 무시하는가?

산출물은 구현 기능의 inventory와 unreachable·partial 후보 목록이다. 최신 정책에 없는 기능이라는 이유만으로
제거 후보로 판정하지 않는다.

### C10. 오류와 관측 경계

확인 항목:

- 사용자·데이터·환경 오류가 typed error로 전달되는가?
- recoverable 오류가 `panic!`, `unwrap`, `expect`로 process를 종료하는가?
- 오류가 어느 command, entity, data path에서 발생했는지 보존하는가?
- debug log, trace, export, metric 실패가 gameplay commit을 실패시키는가?
- gameplay event와 진단용 log가 같은 권위 채널을 공유하는가?
- 오류 발생 후 caller가 재시도, 복구 또는 폐기 여부를 판단할 수 있는가?
- 민감하거나 지나치게 큰 상태가 오류·debug 출력에 포함되는가?
- non-authoritative I/O가 권위 mutation 앞에 배치되는가?

`panic!`이나 `expect`가 허용될 수 있는 경우에도 도달 불가능성이 타입 또는 이전 검증 경계로 증명되어야
한다. 단순한 주석은 증명이 아니다.

### C11. 테스트 신뢰도

확인 항목:

- 핵심 테스트가 가능한 한 production public entry point를 사용하는가?
- 성공 경로뿐 아니라 실패 후 상태 불변성을 검사하는가?
- 테스트가 implementation detail보다 관측 가능한 계약을 확인하는가?
- 테스트 안에서 production 계산을 동일하게 복제하지 않는가?
- fixture가 실제 loader, validator, compiler 경로를 사용하는가?
- 테스트 순서, wall clock, filesystem 잔여물, hash iteration에 의존하지 않는가?
- `is_err()`만 확인하지 않고 오류 종류와 불변 상태를 확인하는가?
- snapshot test가 거대한 출력 변화를 무비판적으로 승인하게 만들지 않는가?
- property, boundary, malformed-input, determinism 테스트가 위험도에 비례해 존재하는가?
- ignored 또는 feature-gated 테스트가 핵심 보장을 비활성화하지 않는가?

테스트 수와 coverage 수치는 조사 보조 지표일 뿐이다. 핵심 위험별로 어떤 증명이 존재하는지 traceability
표를 만든다.

### C12. 성능과 자원 상한

확인 항목:

- tick 또는 event마다 전체 상태를 clone하는가?
- event 하나가 전체 unit, item, buff 목록을 반복 순회하는가?
- targeting, collision, effect 전파가 중첩 순회로 폭증하는가?
- event log, queue, graveyard, dedupe set, cache가 무제한 성장하는가?
- 매 update마다 큰 full snapshot이나 JSON value를 반복 생성하는가?
- 물리 world 또는 인덱스를 필요 이상으로 재구성하는가?
- 외부 데이터가 반복 횟수, recursion, allocation을 무제한으로 늘릴 수 있는가?
- overflow 방지가 wraparound 대신 명시적인 상한과 오류를 제공하는가?
- 성능 최적화가 결정론이나 정확성을 깨뜨리지 않는가?

임의의 절대 성능 점수를 사용하지 않는다. 대표 규모와 최악 규모를 정의해 다음을 측정한다.

- tick/event 처리 시간
- allocation과 clone 양
- queue와 log 성장
- snapshot과 wire payload 크기
- 유닛·effect·projectile 수 증가에 따른 복잡도 변화

측정 환경과 입력 fixture를 함께 기록하지 않은 benchmark 수치는 증거로 인정하지 않는다.

### C13. AI 생성 누적 부채

확인 항목:

- 같은 개념이 이름만 바뀐 타입·helper·validator로 반복 구현되는가?
- 기존 추상화를 사용하지 않고 유사한 새 계층을 추가했는가?
- 실제 caller가 하나인데 speculative extension point가 복잡성을 만드는가?
- borrow 문제를 피하기 위한 큰 clone이 transaction 설계처럼 굳어졌는가?
- 테스트를 통과시키기 위한 entity 이름, ID, 특정 순서 기반 분기가 있는가?
- 주석이나 함수명이 실제 동작과 어긋나는가?
- 오래된 표현과 새 표현이 dual path로 남아 있는가?
- 방어 validator가 여러 층에 추가되어도 근본적으로 invalid state를 표현할 수 있는가?
- 비슷한 오류 변환과 DTO 변환이 여러 계층에 복제되는가?
- 작은 기능마다 새로운 정책 타입과 wrapper가 생겨 의미 추적이 어려운가?

중복 코드를 줄 수로만 판정하지 않는다. 같은 규칙이 여러 곳에서 독립적으로 진화할 가능성과 변경 누락을
compiler가 잡는지를 평가한다.

## 9. 판정 상태

각 세부 항목과 기준은 다음 중 하나로 판정한다.

| 상태 | 의미 |
| --- | --- |
| `PASS` | 조사 범위가 명시되었고 합격을 뒷받침하는 현재 코드·실행 증거가 있다. |
| `PARTIAL` | 일부 경로는 증명됐지만 하위 경로나 예외 경계가 미확인이다. |
| `FAIL` | 현재 코드에서 재현되거나 호출 경로로 증명된 위반이 있다. |
| `UNKNOWN` | 조사 또는 증거가 부족해 판정할 수 없다. |
| `N/A` | 현재 감사 범위에 실제로 존재하지 않는 기준이다. |

`UNKNOWN`을 억지로 `PASS`로 바꾸지 않는다. 감사 완료 시 남아 있는 `UNKNOWN`은 별도 gap register에 이유와
후속 조사 방법을 기록한다.

전체 품질을 단일 백분율이나 평균 점수로 환산하지 않는다. 치명적인 결함이 다수의 사소한 합격에 가려질
수 있기 때문이다.

## 10. 발견사항 심각도

| 등급 | 의미 |
| --- | --- |
| `S0 Blocker` | 정상 핵심 경로에서 상태 손상, 진행 불능, 비복구 crash, 권위 결과의 심각한 비결정성을 만든다. |
| `S1 High` | 부분 commit, 권위 중복, 저장·복원 실패, 핵심 경계 붕괴 또는 넓은 회귀 위험이 있다. |
| `S2 Medium` | 변경 증폭, 중복 책임, 중요한 검증 공백 또는 제한된 잘못된 동작이 있다. |
| `S3 Low` | 국소적인 복잡성, 가시성, 이름, 불필요한 추상화 등 장기 유지보수 비용이 있다. |

심각도는 다음 요소를 함께 보고 정한다.

- 영향: 무엇이 잘못되는가?
- 도달 가능성: production 경로에서 얼마나 쉽게 발생하는가?
- 범위: 단일 기능인가, run 또는 battle 전체인가?
- 복구 가능성: 재시도나 load로 복구할 수 있는가?
- 탐지 가능성: 즉시 오류가 나는가, 조용히 잘못된 상태를 만드는가?

코드 스타일 위반만으로 `S0` 또는 `S1`을 부여하지 않는다.

## 11. 발견사항 확신도

| 확신도 | 의미 |
| --- | --- |
| `Confirmed` | 현재 기준선에서 재현 절차나 자동화 테스트가 존재한다. |
| `Strong inference` | production 호출 경로와 mutation 순서로 결과가 사실상 증명되지만 동적 재현은 없다. |
| `Hypothesis` | 위험 신호는 있으나 결함이나 영향이 아직 증명되지 않았다. |

`Hypothesis`는 최종 확정 결함 수에 포함하지 않는다. 후보 catalog 또는 gap register에 둔다.

## 12. 발견사항 증거 계약

최종 발견사항은 최소한 다음 필드를 가진다.

```markdown
## <ID>. <제목>

- 기준선:
- 기준 ID:
- 심각도:
- 확신도:
- 영향받는 불변조건:
- 코드 위치:
- production 호출 경로:
- 재현 명령 또는 테스트:
- 재현 입력:
- 현재 동작:
- 구조적으로 기대되는 동작:
- 상태 변경 전후 증거:
- 영향 범위:
- 근본 원인:
- 중복·연관 발견:
- 권장 수정 방향:
- 필요한 회귀 테스트:
```

코드 위치는 가능한 한 파일과 줄 번호를 포함하되, 줄 번호만 제시하지 않는다. entry point부터 문제
지점까지의 호출 경로와 실제 상태 영향을 설명한다.

재현이 불가능한 정적 구조 문제는 다음을 대신 제시한다.

- 관련 타입과 필드
- 값을 쓰는 모든 경로
- 값을 읽는 모든 권위 경로
- 중복 또는 불일치가 생기는 조건
- compiler나 validation이 이를 막지 못한다는 근거

## 13. 근본 원인 통합 규칙

한 근본 원인이 여러 증상을 만들면 증상마다 독립된 대형 결함으로 부풀리지 않는다.

예:

- state ownership 중복 하나가 deploy, withdraw, reconnect에서 각각 오동작한다면 하나의 root finding과 여러
  affected path로 기록한다.
- 동일한 validator 복제 때문에 여러 data type에서 drift가 발생한다면 공통 원인과 개별 재현을 함께 묶는다.

반대로 수정 전략과 권위 경계가 서로 다르면 비슷한 증상이어도 별도 발견으로 유지한다.

## 14. 권장 감사 실행 순서

1. 현재 기준선과 도구 결과를 고정한다.
2. crate module, public entry point, command, query, event, DTO 표면을 목록화한다.
3. 주요 권위 상태와 mutation entry point를 매핑한다.
4. `GameCore`와 run/world 상태 전이를 감사한다.
5. `BattleCore`의 event loop, 결정론, 물리·시간, transaction 경계를 감사한다.
6. data deserialize, validate, compile, publication 경계를 감사한다.
7. save, checkpoint, restore, reconnect 경계를 감사한다.
8. 외부 DTO와 향후 Unity adapter seam을 감사한다.
9. 테스트가 위 위험을 실제로 증명하는지 역추적한다.
10. 성능 상한과 비정상 입력 robustness를 측정한다.
11. AI 생성 누적 패턴을 root cause 단위로 통합한다.
12. 확정 발견, 후보, `UNKNOWN`, remediation backlog를 분리한다.

파일 순서대로 읽는 감사보다 entry point에서 상태 변경과 출력까지 추적하는 vertical call-path 감사를
우선한다.

## 15. 필수 감사 산출물

감사 goal은 최소한 다음 산출물을 남긴다.

1. 기준선 기록
2. runtime surface inventory
3. state authority map
4. command와 mutation entry point catalog
5. determinism input·ordering inventory
6. fallible mutation과 transaction boundary catalog
7. data validation·publication map
8. persistence·restore boundary map
9. external DTO·adapter seam map
10. test evidence matrix
11. performance·unbounded growth 후보 목록
12. 확정 발견사항 catalog
13. hypothesis와 `UNKNOWN` gap register
14. root-cause 기준 remediation 우선순위
15. 감사 완료 보고서

도식은 관계가 복잡할 때만 사용하며, 표나 호출 경로 목록으로 충분하면 불필요한 대형 diagram을 만들지
않는다.

## 16. 감사 완료 조건

다음을 모두 만족해야 구조 감사가 완료된다.

- 감사 기준선이 재현 가능하게 식별되었다.
- production runtime의 주요 진입점과 상태 소유자가 목록화되었다.
- 모든 주요 mutation 경로에서 사전 검증, mutation, event, projection, 실패 순서가 조사되었다.
- RNG, 시간, ordering, UUID, collection iteration, physics 결정론 입력이 조사되었다.
- data가 deserialize되어 runtime에 publication되는 전체 경계가 조사되었다.
- durable state와 temporary state, 저장·복원 경계가 조사되었다.
- 외부에 노출되는 command, result, event, snapshot의 변환 경계가 조사되었다.
- 핵심 위험마다 현재 테스트 증거 또는 테스트 공백이 연결되었다.
- 모든 `FAIL`은 증거 계약을 만족한다.
- 모든 `UNKNOWN`은 남은 이유와 후속 조사 방법을 가진다.
- 중복 증상은 가능한 한 근본 원인 단위로 통합되었다.
- 감사 중 production 문제를 수정했다면 새 기준선에서 영향받는 판정을 다시 검증했다.

감사 완료는 런타임이 무결점이라는 뜻이 아니다. 현재 기준선의 구조적 위험이 재현 가능한 형태로
목록화되고, 무엇이 아직 미확인인지 명확해졌다는 뜻이다.

## 17. 후속 goal과의 관계

구조 감사 뒤에는 발견사항을 다음과 같이 분리한다.

1. `Runtime Structural Remediation`: 구조 감사에서 확정된 문제 수정
2. `Runtime Policy Alignment Audit`: 최신 정책 동결 후 정책과 구현 비교
3. `Unity Adapter Contract Design`: 새 Unity 연결을 위한 versioned boundary 설계
4. `Unity Integration Audit`: 실제 Unity 구현 후 end-to-end 호환성과 흐름 검증

정책 정합성 감사나 Unity 통합 감사의 결과를 현재 구조 감사에 소급하여 섞지 않는다.

## 18. Goal 참조 문구

이 기준을 사용하는 감사 goal은 PLAN 또는 목표 선언에 다음 의미를 포함한다.

```markdown
이 goal은 `docs/audit/current_runtime_structural_integrity_audit_criteria.ko.md`를
판정 기준과 증거 계약의 source of truth로 사용한다. 과거 `docs/goals/**`, 현재 rulebook의
정책 정합성, 오래된 Unity 구현 호환성은 감사 근거와 완료 조건에서 제외한다. 모든 판정은
고정된 현재 작업 트리와 production runtime 호출 경로에서 독립적으로 증명한다.
```

