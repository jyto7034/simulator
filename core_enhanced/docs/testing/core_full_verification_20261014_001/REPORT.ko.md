# Core 전체 기본 검사 재검증

2026-10-14 사용자의 전체 실패 유무 확인 요청에 따라 `game_core` 기본 feature의 전체 target을 다시 실행했다. **단위2,097개 + 통합72개 = 총2,169개 통과, 실패0개, 기존 ignore6개**다. 과거 여러 실행의 합집합 대신 이번 실행을 근거로 한다.

## 실행 조건과 결과

| 실행 | 결과 | 조건 |
|---|---|---|
| [full001](full001.result.json) | exit0, 단위2,096 + 통합72 통과, 실패0, 기존ignore6 | 기본 feature `--all-targets --no-fail-fast`, G11 환경변수2개 제거, Unix socket1만 환경상 분리 |
| [socket002](socket002.result.json) | exit0, 1통과 | 같은 현재 빌드의 Unix socket 검사를 허용 환경에서 실행 |
| [doc003](doc003.result.json) | exit0, 문서 테스트0 | `--doc` 명령 정상 종료; 기능 PASS 수에 더하지 않음 |

전체 실행에서 제외한 socket1은 성공으로 간주한 skip이 아니다. 샌드박스의 Unix 소켓 생성 차단이 이미 확인돼 별도 허용 환경에서 실제 통과시켰다. 두 실행의 대상은 겹치지 않는다. 기존 ignore6은 실제 검토된 승인 캐시·별도 관리 실행을 필요로 하는 U11 검사이며 이번에 새로 제외하지 않았다. 정확한 목록과 결과는 [실행 색인](EXECUTION_INDEX.json)을 따른다. 바이너리3개의 시험0도 성공 사례 수에 더하지 않는다.

범위는 기본 feature의 `game_core` 전체 target과 별도 doc 명령이다. 모든 선택 feature의 전체 조합, 서버/Unity, 승인 캐시 관리 행위, 성능SLO 전체를 실행했다는 뜻은 아니다. 이전 계측/CLI 및 U14 범위 이력은 각 당시 기록을 유지한다.

## 이번에 닫은 기본 실행 조건

이전 G11 4개는 최신 U14 manifest를 `U12_G11_INPUTS`로 지정하면 통과했지만, 테스트의 기본 locator는 구 U12 입력을 가리켰다. 따라서 이전 단계에서는 환경 지정 없는 전체 실행의 실패0을 주장할 수 없었다.

이번에는 [G11 테스트](../../../src/game/world/tests/u12_lifecycle/cost.rs)의 `DEFAULT_INPUTS` 한 줄을 현재 U14 `g11_inputs001/manifest.json`으로 바꿨다. 환경변수 `U12_G11_INPUTS`·`U12_G11_INPUTS_SHA256`을 제거한 실제 전체 실행에서 4개 모두 통과했다. 과거 fixture·manifest·receipt·원시 결과 및 기존 oracle는 수정하지 않았다. 제품 실행 로직도 변경하지 않았다.

[정확한 변경](full001.diff), [입력 해시](full001.inputs.json), [실행 환경](full001.environment.json), [원시 전체 stdout](full001.stdout.log), [소켓 stdout](socket002.stdout.log), [문서 stdout](doc003.stdout.log)을 보존한다.

## 재현

저장소 루트에서 소켓 생성이 허용되는 환경이면 전체 target을 분리 없이 실행할 수 있다. 아래 경로는 이번 환경 기준이다.

```bash
env -u U12_G11_INPUTS -u U12_G11_INPUTS_SHA256 \
  PATH=/home/blast/.cargo/bin:$PATH CARGO_HOME=/tmp/wp02-cargo-home \
  cargo test -p game_core --all-targets --offline --locked \
  --target-dir /mnt/f/work/simulator/.cache/u12-vault-cost-target --no-fail-fast
```

이번 실제 명령은 소켓만 분리한 형태이며 각 result JSON에 정확한 argv와 종료 코드를 남겼다. 입력은 전체 실행부터 문서 검사 종료까지 고정했다. 이번 확인으로 이전138 및 추가17 교정 이후 기본 Core 전체의 실패0을 확인했으며, U15는 착수하지 않았다.
