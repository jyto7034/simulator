import type { Definition } from "../types";
import {
  choice,
  cue,
  define,
  incoming,
  input,
  next,
  number,
  output,
  unresolved,
} from "./skill-definition";

export const resourcePortTypes = [
  ["battle-counter", "사본별 전투 자원"],
  ["counter-terms", "자원 접수 조건"],
  ["counter-receipt", "확정 자원 소비"],
  ["aim-memory", "사본별 조준 기억"],
] as const;
const counter = () => input("counter", "사본별 자원", "battle-counter");
const integer = (id: string, label: string, min = 0) => ({
  ...number(id, label, min),
  integer: true,
});

// Definitions bind to the equipped fragment copy; no authoring node stores live battle state.
export const resourceCatalog: Definition[] = [
  define(
    "battle-counter",
    "사본별 전투 자원",
    "스킬 상태",
    "장착 파편 사본별 0 이상 정수 자원입니다. 배치 시 0, 사망·철수·전투 종료에 값과 예약·주기 진행·조준 기억을 정리합니다. 전술 이동은 보존합니다. 획득은 상한에서 자르고 초과분을 버립니다. 같은 사본의 선언을 참조하며 다른 직원·사본과 값을 합치지 않습니다. 선언 자체는 자원 획득 사건이 아닙니다.",
    [],
    [output("counter", "자원 정의", "battle-counter")],
    [integer("capacity", "자원 상한", 1)],
  ),
  define(
    "counter-periodic",
    "배치 중 자원 주기",
    "스킬 사건",
    "같은 사본이 살아서 배치된 동안 첫 주기 뒤부터 전투 시간으로 발생합니다. 일반 시전·기절·침묵 중에도 진행하며 공명 충전과 다릅니다. 상한 자동 시전의 예약/실행 중에는 멈추고 정상 복귀 뒤 0부터 셉니다. 전량 소비 접수는 0개 소비도 타이머를 초기화합니다. Pause·미배치 시간은 비축하지 않고 종료한 사본의 콜백은 무효입니다.",
    [counter(), input("interval", "획득 주기", "seconds")],
    [next()],
  ),
  define(
    "counter-gain",
    "사본 자원 획득",
    "스킬 동작",
    "같은 사본의 주기 또는 자연 완료 보상 흐름에서 정수 획득량을 한 번 적용합니다. 획득 증폭 뒤 상한을 적용하고 초과분을 버립니다. 보상 전체를 확정한 뒤 상한 도달을 알리므로 기본/추가 보상 사이에 자동 시전이 끼어들지 않습니다. 사건·사용 회차 ID로 중복 획득을 막습니다.",
    [incoming(), counter(), input("amount", "획득 개수", "number")],
    [next(), cue("자원 획득")],
  ),
  define(
    "counter-terms",
    "추가 자원 접수 조건",
    "스킬 값",
    "공명 접수와 같은 원자 경계에서 이번 요청 사본의 최소/최대 자원과 자동 시전 예약을 검사합니다. 전량 소비는 실제 소비량을 영수증에 고정하고 주기를 초기화합니다. 종료 시 소진은 상한 자동 시전의 정상/중단 종료에 적용합니다. 거절 시 공명·자원·사용 잠금 모두 무변경이며 취소 환불은 없습니다.",
    [counter()],
    [output("terms", "자원 접수 조건", "counter-terms")],
    [
      integer("minimum", "최소 자원"),
      integer("maximum", "최대 자원"),
      choice("spend", "자원 소비 시점", [
        unresolved,
        ["none", "소비 없음"],
        ["admission-all", "접수 시 전량"],
        ["end-all", "자동 시전 종료 시 전량"],
      ]),
    ],
  ),
  define(
    "counter-receipt",
    "확정 소비 자원 읽기",
    "스킬 동작",
    "같은 접수 성공 흐름에서 전량 소비가 확정된 개수를 읽습니다. 이후 현재 자원이나 UI 예상량으로 대신하지 않습니다. 이 결과를 같은 사용의 범위·위력에만 연결하며 다른 요청의 효과에 재사용하지 않습니다.",
    [incoming(), input("receipt", "확정 소비", "counter-receipt")],
    [next(), output("count", "실제 소비 개수", "number")],
  ),
  define(
    "aim-memory",
    "사본별 조준 타일 기억",
    "스킬 상태",
    "같은 전투 자원 사본의 마지막 접수 조준 타일을 보관합니다. 기록 동작을 연결한 사용만 갱신합니다. 새 전투·사망·철수 시 비우고 전술 이동은 보존합니다. 자동 요청에서 기억이 없거나 타일이 무효면 당시 시전자 타일을 사용합니다. 선택한 유닛을 추적하지 않습니다.",
    [counter()],
    [output("memory", "조준 기억", "aim-memory")],
  ),
  define(
    "remember-aim",
    "접수 조준 타일 기록",
    "스킬 동작",
    "같은 사본의 성공한 지정 타일 사용을 기억합니다. 접수 거절·미리보기는 기록하지 않습니다. 이후 시전 중단은 이미 접수한 기억을 되돌리지 않습니다. 다른 메뉴 항목은 이 동작을 연결하지 않으면 기억을 바꾸지 않습니다.",
    [
      incoming(),
      input("use", "이번 사용", "skill-use"),
      input("memory", "조준 기억", "aim-memory"),
    ],
    [next()],
  ),
  define(
    "menu-request",
    "스킬 메뉴 입력 사건",
    "스킬 사건",
    "메뉴 항목 ID로 구분한 수동 입력입니다. 플레이어는 이 항목과 연결된 선택 설정으로 조준합니다. 최종 요청 ID·직원·장착 사본·타일을 보존하며 공명/추가 자원 검사와 지불은 접수에서 함께 확정합니다. 항목 이름만으로 실행 동작을 추론하지 않습니다.",
    [input("selection", "대상 선택 설정", "target-selection")],
    [next(), output("request", "사용 요청", "skill-request")],
    [{ id: "entry", label: "메뉴 항목 ID", valueType: "text", default: "" }],
  ),
  define(
    "counter-auto-request",
    "자원 상한 자동 요청",
    "스킬 사건",
    "같은 사본 자원이 상한에 도달하면 단일 자동 요청을 예약합니다. 현재 사용과 그 완료 보상을 정리한 뒤 새 수동 요청보다 먼저 시작하며 추가 공명은 소비하지 않습니다. Hard CC·침묵·미배치에는 시작하지 않고 자격 회복을 기다립니다. 예약 동안 새 수동 입력을 막습니다. 조준 기억의 고정 타일을 요청에 넣고 같은 타일 방향은 현재 바라보는 방향입니다. 시작한 자동 집중은 수동 취소할 수 없습니다. 정상/외부 중단 종료 시 자원을 비우고 정상 복귀하며 중단한 시전을 자동 재개하지 않습니다.",
    [counter(), input("memory", "조준 기억", "aim-memory")],
    [
      next(),
      output("request", "자동 사용 요청", "skill-request"),
      cue("예약·위험 위치"),
    ],
  ),
];

export const requestKinds = [
  "skill-plan-prepare",
  "skill-request",
  "skill-menu-request",
  "skill-counter-auto-request",
];
