import type { Definition } from "../types";
import {
  define,
  input,
  output,
  number,
  integer,
  choice,
  unresolved,
  multipleInput,
  optionalInput,
} from "./skill-definition";

const effect = () => output("effect", "대상 효과", "target-effect-profile");
const plan = () => output("plan", "행동 계획", "action-plan");
export const protectionCatalog: Definition[] = [
  define(
    "attribute-effect",
    "출처별 능력치 기여",
    "스킬 값",
    "상태 수명 동안 지정한 능력치에 독립 기여분을 설치합니다. Attack 증가는 이를 참조하는 이후 공격에 적용하고 이미 방출한 공격을 소급 수정하지 않습니다. 기본 공격 피해/속도 전용 값은 스킬·장비 proc·소환물·공명·배치 회복을 가속하지 않습니다. 종료는 이 출처의 기여분만 제거합니다. 같은 정의의 복수 출처는 가장 강한 값만 사용합니다.",
    [],
    [effect()],
    [
      choice("attribute", "변경할 능력치", [
        unresolved,
        ["attack", "공격력"],
        ["defense", "물리 방어"],
        ["attack-speed", "공격 속도"],
        ["basic-power", "기본 공격 피해만"],
        ["basic-speed", "기본 공격 속도만"],
      ]),
      choice("operation", "계산 방식", [
        unresolved,
        ["percent", "기준 능력치의 백분율 증가"],
        ["points", "포인트 가산"],
      ]),
      number("amount", "증가량"),
    ],
  ),
  define(
    "hp-floor-effect",
    "전투 피해의 HP 하한",
    "스킬 값",
    "피해 계산·개인 보호막 뒤 실제 HP 감소를 지정 하한에서 멈춥니다. HP가 이미 하한 아래라면 현재 HP를 늘리지 않습니다. 피해·기절 면역이나 회복이 아닙니다. HP 비용·최대 HP 희생·직접 처형에는 적용하지 않습니다. 종료는 하한만 제거하며 누적 피해 청구·회복·사망을 만들지 않습니다. 사망 확정 대상에는 설치하지 않습니다.",
    [],
    [effect()],
    [integer("minimum", "최소 HP", 1)],
  ),
  define(
    "personal-shield-effect",
    "대상별 개인 보호막",
    "스킬 값",
    "부여 시 시전자의 최대 HP를 고정해 대상마다 동일한 독립 보호량을 만듭니다. 대상 방어·저항·감면 뒤 피해를 흡수하고 초과분을 방어 재계산 없이 HP에 적용합니다. 비용은 흡수하지 않습니다. 범위 이탈·시전자 이동·HP 회복으로 잔량을 바꾸지 않으며 잔량 0 또는 상태 만료에 종료합니다. 대상끼리 공유하거나 대상 수로 나누지 않습니다. 같은 출처·정의·대상에 재부여하면 새 양으로 교체합니다.",
    [],
    [effect()],
    [number("percent", "시전자 최대 HP 비율 (%)")],
  ),
  define(
    "recipient-buff-plan",
    "대상 집합에 시간제 상태 부여",
    "스킬 값",
    "방출 때 정한 살아 있는 active player 직원 집합에 연결한 상태를 대상마다 한 번 적용합니다. self는 자신 한 명, selected는 접수한 아군, allies는 현재 범위의 자신 포함 직원입니다. Guest·소환물·방어 오브젝트는 제외하고 Ground/Platform은 포함합니다. 새 진입자에게 재부여하지 않습니다. 상태는 대상별 수명으로 남고 부여 동작은 즉시 완료하여 기본 공격을 계속합니다. 공명 사용 잠금은 마지막으로 남은 시간제 버프가 끝날 때까지 유지합니다. 범위 밖 기존 대상의 상태는 자기 만료까지 남습니다.",
    [
      input("state", "시간제 상태", "target-state-profile"),
      optionalInput("range", "아군 조회 범위", "tile-pattern"),
    ],
    [plan()],
    [
      choice("recipient", "부여 대상", [
        unresolved,
        ["self", "자기 자신"],
        ["selected", "선택한 아군 직원"],
        ["allies", "범위 내 아군 직원"],
      ]),
    ],
  ),
  define(
    "next-basic-plan",
    "다음 기본 공격 한 번 강화",
    "스킬 값",
    "다음 새로 시작하는 정상 무기 기본 공격에 준비 출처를 결속하고 release에서 한 번 소비합니다. 기존 windup·주기·사거리·타입·저지·표적 선택을 바꾸거나 즉발 공격을 만들지 않습니다. release 전 취소는 준비를 보존하며 방출 뒤 빗나감은 환불하지 않습니다. 무기의 다단/범위 전체는 같은 행동 ID로 묶고 강화된 직접 피해의 확정 처치가 있으면 보상 상태를 소유자에게 공격당 한 번 적용합니다. proc·지속·연쇄·다른 아군의 처치는 제외합니다. 준비는 철수·사망·전투 종료에 폐기합니다. 보상은 이미 처치한 공격을 소급 강화하지 않습니다.",
    [optionalInput("reward", "강화 공격 처치 보상", "target-state-profile")],
    [plan()],
    [number("multiplier", "기본 공격 피해 배율")],
  ),
  define(
    "reactive-heal-plan",
    "한정 횟수 반응 회복 부여",
    "스킬 값",
    "접수한 살아 있는 아군 직원에게 출처별 횟수 저장소를 부여합니다. 대상의 실제 HP 감소가 확정된 적대적 공격 action마다 한 번, 남은 횟수를 먼저 1 소비하고 당시 대상 최대 HP 비율로 회복합니다. 같은 다단 action의 중복 결과는 제외합니다. 보호막만 감소·HP 비용·사망 확정에는 반응하지 않습니다. 회복은 최대 HP를 넘지 않고 부활하지 않습니다. 회복 표시 개체는 유닛/소환물이 아닙니다. 횟수 소진·시간 만료·대상 이탈·전투 종료에 정리하며 시전자 이탈 뒤에는 남은 수명을 유지합니다. 같은 출처 재부여는 교체합니다.",
    [input("duration", "반응 상태 지속 시간", "seconds")],
    [plan()],
    [
      integer("charges", "최초 회복 횟수", 1),
      { ...number("percent", "대상 최대 HP 회복 (%)"), min: 10, max: 20 },
    ],
  ),
  define(
    "aura-buff-plan",
    "집중 범위의 능력치 기여",
    "스킬 값",
    "유한 집중 동안 시전자 중심 범위 안의 자신 이외 active player 직원에게만 효과를 적용합니다. 새 진입/배치도 포함하며 이탈·철수·집중 종료와 같은 전이에서 기여분을 제거합니다. Guest·소환물·방어 오브젝트를 제외합니다. 유효한 기본 공격 가능 직원이 없으면 접수할 수 없습니다. 집중 중 기본 공격·다른 시전·태세 전환·전술 이동·자리 교대·Field Work를 막고 배치/피격과 기존 무기 저지는 유지합니다. 직접 중지·Hard CC·강제 이동·철수·사망·전투 종료에 취소하며 보통 피해·시작 후 Silence에는 유지합니다. Pause에 동결합니다. 종료 뒤 새 windup을 시작하고 공격 보충이나 환불을 하지 않습니다.",
    [
      input("duration", "집중 지속 시간", "seconds"),
      input("range", "시전자 중심 범위", "tile-pattern"),
      multipleInput("effects", "범위 안 능력치 기여", "target-effect-profile"),
    ],
    [plan()],
  ),
];
