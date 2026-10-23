import type { Definition, Field, FieldValue } from "../types";
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

export const groundPortTypes = [
  ["ground-target-filter", "장판 대상 조건"],
  ["ground-member", "이번 장판 대상 관계"],
  ["healing-power", "회복 요청량"],
  ["ground-status-profile", "장판 상태 정의"],
] as const;

const area = () => input("area", "이번 장판", "skill-area-instance");
const filter = () => input("filter", "대상 조건", "ground-target-filter");
const member = () => input("member", "이번 대상 관계", "ground-member");
const status = () => input("status", "상태 정의", "ground-status-profile");
const target = () => output("member", "이번 대상 관계", "ground-member");

// Authoring contracts. These nodes do not run a timer, query battle units, or apply effects.
export const groundCatalog: Definition[] = [
  define(
    "ground-ramping-slow",
    "체류 점진 둔화",
    "스킬 사건",
    "이번 장판의 대상별 진입 관계에 둔화를 부여합니다. 생성 당시 대상도 0부터 시작하며 필요 체류 시간 동안 0%에서 최대 둔화율까지 선형 증가합니다. 이탈하면 둔화·진행을 버리고 재진입은 0부터 시작합니다. 도달 흐름은 관계당 한 번 발생하며 둔화는 이탈까지 최대값을 유지합니다. 둔화 면역도 체류 진행은 유지하며 후속 기절의 면역은 별도로 검사합니다. 같은 정의의 여러 장판 둔화는 가장 강한 값만 적용합니다. 장판 종료가 같은 시각의 도달보다 우선합니다. 정지 중 시간은 흐르지 않습니다.",
    [area(), filter(), input("duration", "필요 체류 시간", "seconds")],
    [output("out", "체류 시간 도달", "flow"), target(), cue("체류 도달")],
    [{ ...number("maxSlow", "최대 둔화율 (%)"), max: 100 }],
  ),
  define(
    "ground-filter",
    "장판 대상 조건",
    "스킬 값",
    "장판 또는 일회 조회 사용의 출처 진영을 기준으로 살아 있는 전장 유닛을 고릅니다. 아군·적군·전체와 시전자 포함 여부를 각각 정합니다. 포함 설정도 범위 밖의 시전자를 끌어오지 않습니다. 비행·플랫폼 여부만으로 제외하지 않으며 효과 면역은 각 효과에서 별도로 판정합니다.",
    [],
    [output("filter", "대상 조건", "ground-target-filter")],
    [
      choice("side", "대상 진영", [
        unresolved,
        ["enemies", "적군"],
        ["allies", "아군"],
        ["all", "전체"],
      ]),
      choice("caster", "시전자 포함", [
        unresolved,
        ["include", "범위 안이면 포함"],
        ["exclude", "제외"],
      ]),
    ],
  ),
  define(
    "ground-enter",
    "장판 진입 사건",
    "스킬 사건",
    "이번 장판과 대상 조건에 구독합니다. 생성 당시 이미 안에 있던 대상도 진입으로 처리합니다. 장판·대상·진입 회차별 관계를 만들고 한 번 발생합니다. 계속 머무르는 동안 다시 진입시키지 않으며 재진입은 새 관계입니다.",
    [area(), filter()],
    [output("out", "대상 진입", "flow"), target(), cue("대상 진입")],
  ),
  define(
    "ground-exit",
    "장판 이탈 사건",
    "스킬 사건",
    "살아 있는 장판에서 대상이 범위·대상 조건을 벗어나거나 전장에서 사라져 관계가 닫힐 때 한 번 발생합니다. 닫힌 관계의 출처·대상·진입 회차를 보존하며 재진입 관계로 바꾸지 않습니다. 장판 자체의 종료는 별도 장판 종료 사건에서 다룹니다.",
    [area(), filter()],
    [output("out", "대상 이탈", "flow"), target(), cue("대상 이탈")],
  ),
  define(
    "ground-dwell",
    "장판 체류 시간 도달",
    "스킬 사건",
    "같은 장판 대상 관계가 지정한 양수 전투 시간 동안 이어지면 관계당 한 번 발생합니다. 생성 당시 대상은 생성 시점부터 셉니다. 이탈하면 진행을 버리고 재진입은 0부터 셉니다. 장판 종료 시각과 겹치면 발생하지 않으며 다른 장판의 체류 시간과 합치지 않습니다.",
    [area(), filter(), input("duration", "필요 체류 시간", "seconds")],
    [output("out", "체류 시간 도달", "flow"), target(), cue("체류 도달")],
  ),
  define(
    "ground-periodic",
    "장판 주기 사건",
    "스킬 사건",
    "장판 생성 시점에 결속한 양수 전투 시간 간격입니다. 첫 발동은 생성 즉시 또는 한 주기 뒤로 정합니다. 각 주기는 새 사건이며 대상 조회·피해는 연결한 동작이 수행합니다. 장판 종료 시각의 주기는 발생하지 않습니다. 일시정지 중 진행하거나 재개 시 밀린 주기를 보충하지 않습니다.",
    [area(), input("interval", "발동 주기", "seconds")],
    [output("out", "이번 주기", "flow")],
    [
      choice("first", "첫 발동", [
        unresolved,
        ["immediate", "생성 즉시"],
        ["after-interval", "한 주기 뒤"],
      ]),
    ],
  ),
  define(
    "ground-targets",
    "장판 안 대상별 실행",
    "스킬 동작",
    "장판 생성 완료 또는 같은 장판 사건의 흐름에서 현재 범위·대상 조건에 맞는 유한한 대상을 조회합니다. 각 대상의 관계와 같은 사건 문맥으로 후속 흐름을 한 번씩 실행합니다. 빈 범위에서는 대상 흐름이 없습니다. 대상별 흐름은 지연·재진입 없이 동기 처리합니다. 전체 완료는 빈 범위에서도 한 번 발생하며 모든 대상 처리가 끝난 뒤 새 위치 조회에 사용할 수 있습니다. 전체 완료에는 개별 대상 관계를 넘기지 않습니다.",
    [incoming(), area(), filter()],
    [
      output("out", "대상마다", "flow"),
      output("completed", "모든 대상 처리 완료", "flow"),
      target(),
    ],
  ),
  define(
    "ground-ended",
    "장판 종료 사건",
    "스킬 사건",
    "장판의 수명 만료·소유 집중 종료 또는 전투 종료에서 한 번 발생합니다. 먼저 이 장판의 주기·체류 구독과 대상 관계, 이 장판에 묶인 상태 기여분을 정리합니다. 다른 장판의 상태는 유지합니다. 같은 시각의 새 주기보다 종료가 우선하며 종료 흐름으로 장판을 다시 활성화하지 않습니다.",
    [area()],
    [output("out", "정리 후 종료", "flow"), cue("장판 종료")],
  ),
  define(
    "damage-amount",
    "피해 위력 값",
    "스킬 값",
    "피해 판정에 전달할 0 이상 정수 위력입니다. 실제 HP 손실이 아니며 피해 유형·방어·면역 판정은 피해 적용에서 처리합니다. 빈 값은 미정입니다.",
    [],
    [output("power", "피해 위력", "damage-power")],
    [{ ...number("value", "피해 위력"), integer: true }],
  ),
  define(
    "healing-amount",
    "회복 요청량 값",
    "스킬 값",
    "회복 판정에 전달할 0 이상 정수 요청량입니다. 실제 회복량과 구분하며 빈 값은 미정입니다. 초과 회복이나 사망자 부활을 뜻하지 않습니다.",
    [],
    [output("power", "회복 요청량", "healing-power")],
    [{ ...number("value", "회복 요청량"), integer: true }],
  ),
  define(
    "ground-damage",
    "장판 대상 피해",
    "스킬 동작",
    "같은 사건 흐름의 대상 관계에 명시한 위력·피해 유형으로 파편 스킬 피해를 한 번 요청합니다. 장판 출처·효과 지점·이번 사건을 보존합니다. 이탈 사건도 기존 대상에게 적용할 수 있지만 소멸·사망 대상을 바꾸거나 되살리지 않습니다. 처리 완료는 유효 피해나 처치 성공을 뜻하지 않습니다. 결과에는 이번 사용·단계·사건·대상·출처 ID와 요청 위력, 실제 HP 손실, 보호막 손실, 거절을 구분해 보존합니다.",
    [incoming(), member(), input("power", "피해 위력", "damage-power")],
    [
      next(),
      output("result", "확정 HP 피해 결과", "damage-result"),
      cue("피해 처리"),
    ],
    [
      choice("damageType", "피해 유형", [
        unresolved,
        ["physical", "물리"],
        ["magic", "마법"],
        ["true", "고정"],
      ]),
    ],
  ),
  define(
    "ground-heal",
    "장판 대상 회복",
    "스킬 동작",
    "같은 사건 흐름의 대상 관계에 회복을 한 번 요청합니다. 효과 적용 시 생존·회복 가능 여부와 현재 HP 상한을 검사합니다. 초과분을 비축하거나 사망자를 부활시키지 않습니다. 장판 출처와 사건을 유지하며 처리 완료를 실제 회복량으로 사용하지 않습니다. 결과에는 사용·단계·사건·대상 ID와 요청량, 실제 HP 회복, HP 상한 초과분, 거절을 따로 보존합니다. 거절량은 상한 초과분이 아닙니다.",
    [incoming(), member(), input("power", "회복 요청량", "healing-power")],
    [
      next(),
      output("result", "확정 HP 회복 결과", "healing-result"),
      cue("회복 처리"),
    ],
  ),
  define(
    "ground-status",
    "장판 상태 정의",
    "스킬 상태",
    "대상에게 부여할 상태의 종류·수치·수명·중첩을 정의합니다. 같은 정의를 쓰는 여러 장판의 수치 효과는 합산 또는 가장 강한 값으로 결합합니다. 기절은 유효 기여분이 하나라도 남으면 유지합니다. 기여분은 장판·대상·이 정의별로 구분하며 반복 적용만으로 같은 기여분을 중첩하지 않습니다.",
    [],
    [output("status", "상태 정의", "ground-status-profile")],
    [
      choice("effect", "상태 효과", [
        unresolved,
        ["slow", "이동 속도 감소"],
        ["stun", "기절"],
        ["damage-taken", "받는 피해 증가"],
      ]),
      number("percent", "효과 크기 (%)"),
      choice("overlap", "같은 상태 중첩", [
        unresolved,
        ["strongest", "가장 강한 값"],
        ["sum", "합산"],
      ]),
      choice("lifetime", "이탈 후 상태", [
        unresolved,
        ["membership", "이탈 시 해제"],
        ["area", "장판 종료까지 유지"],
      ]),
    ],
  ),
  define(
    "ground-apply-status",
    "장판 대상 상태 적용",
    "스킬 동작",
    "같은 사건의 유효한 대상 관계에 연결한 상태를 적용합니다. 상태별 면역을 검사하며 다른 효과의 성공을 요구하지 않습니다. 이탈 시 해제는 해당 진입 관계에, 장판 종료까지 유지는 해당 장판·대상에 수명을 묶습니다. 장판 종료와 대상 소멸에서는 모두 정리합니다. 이미 이탈한 관계에는 새 상태를 부여하지 않습니다.",
    [incoming(), member(), status()],
    [next(), cue("상태 처리")],
  ),
  define(
    "ground-clear-status",
    "이번 장판 상태 해제",
    "스킬 동작",
    "대상 관계와 상태 정의로 지정한 이번 장판의 기여분만 해제합니다. 다른 장판이나 재진입 뒤의 새 관계 상태를 지우지 않습니다. 이미 해제된 기여분은 무변경으로 완료합니다. 상태 정의의 자동 종료 조건은 이 동작을 연결하지 않아도 유지됩니다.",
    [incoming(), member(), status()],
    [next(), cue("상태 해제")],
  ),
];

export function groundFieldActive(
  node: { kind: string; values: Record<string, FieldValue> },
  field: Pick<Field, "id">,
) {
  return (
    node.kind !== "skill-ground-status" ||
    !["percent", "overlap"].includes(field.id) ||
    ["slow", "damage-taken"].includes(String(node.values.effect))
  );
}
