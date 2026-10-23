import type { Definition } from "../types";
import {
  define,
  input,
  output,
  incoming,
  next,
  cue,
  number,
  choice,
  unresolved,
  optionalInput,
  multipleInput,
  integer,
} from "./skill-definition";
import { bankInput } from "./inventory-catalog";

export const actionPlanPortTypes = [
  ["hit-status", "타격 뒤 상태 정의"],
  ["hit-ramp", "시전 대상별 후속 피해 증가"],
  ["action-plan", "조합한 행동 계획"],
  ["skill-program", "스킬 시작과 행동 설정"],
  ["action-run", "이번 행동 실행"],
] as const;
const plan = (id = "plan", label = "행동 계획") =>
  input(id, label, "action-plan");
const planOut = () => output("plan", "행동 계획", "action-plan");
const programOut = () => output("program", "시작과 행동 설정", "skill-program");
const count = () => input("count", "개수 계산", "count-rule");
export const actionPlanCatalog: Definition[] = [
  define(
    "damage-bonus",
    "기본보다 강한 피해 위력",
    "스킬 값",
    "같은 기본 피해 위력에 양수 정수 추가 위력을 더합니다. 실제 피해 결과가 아니라 피해 판정에 요청할 위력이며 방어·면역 판정은 소비하는 공격이 수행합니다. 미정 추가 위력을 0으로 대체하지 않습니다.",
    [input("base", "기본 피해 위력", "damage-power")],
    [output("power", "증가한 피해 위력", "damage-power")],
    [integer("bonus", "추가 위력", 1)],
  ),
  define(
    "hit-status",
    "타격 뒤 상태",
    "스킬 값",
    "직접 타격 처리 뒤 살아 있는 해당 대상에게 별도 효과로 부여합니다. 기절·속박·기본 공격 불가는 같은 출처 재적용 때 남은 시간을 max(기존, 새 지속 시간)로 갱신하고 제어 면역/저항을 개별 검사합니다. 기본 공격 불가는 새 기본 공격을 막고 미방출 기본 공격을 취소하되 스킬을 막지 않습니다. 받는 피해 증가 스택은 사본·정의·대상별로 더하고 전체 지속 시간을 새 시간으로 갱신합니다. 공격 자체가 HP 피해 0이어도 유효 적중이면 별도 상태 면역 판정을 할 수 있습니다. 타격 거절·빗나감·죽은 대상에는 붙이지 않습니다. 상태는 자기 지속 시간을 따르며 행동 종료로 지우지 않습니다.",
    [
      input("duration", "상태 지속 시간", "seconds"),
      optionalInput("count", "부여 스택 계산", "count-rule"),
    ],
    [output("status", "상태 정의", "hit-status")],
    [
      choice("effect", "타격 뒤 효과", [
        unresolved,
        ["stun", "기절"],
        ["root", "속박"],
        ["basic-lock", "기본 공격 불가"],
        ["vulnerability", "받는 모든 피해 증가 스택"],
      ]),
      number("percent", "스택당 받는 피해 증가 (%)"),
    ],
  ),
  define(
    "hit-ramp",
    "시전 내 후속 타격 증가",
    "스킬 값",
    "같은 행동 실행·같은 대상에 대한 이 설정을 가진 직접 타격의 유효 적중 뒤 증가율을 더합니다. 첫 타격은 증가 전 피해이고 다음 타격부터 적용합니다. 다른 공격·다른 시전·다른 대상과 공유하지 않고 행동 종료 시 폐기합니다. 대상의 일반 받는 피해 증가 상태가 아닙니다. 연타를 총피해 한 번으로 합치지 않습니다.",
    [],
    [output("ramp", "후속 타격 계산", "hit-ramp")],
    [number("percent", "적중 뒤 증가 (%p)")],
  ),
  define(
    "strike-plan",
    "한 번의 직접 타격 계획",
    "스킬 값",
    "주 대상 1회 또는 타일 패턴 안 현재 적 각각에게 피해를 한 번 처리하고 타격 뒤 상태를 적용합니다. 주 대상은 이번 행동이 고른 대상이며 사망·무효화 시 다른 적으로 자동 이전하지 않습니다. 지정 타일 범위는 접수 타일과 작성 방향을 고정하고 매 타격 현재 대상을 다시 조회합니다. 전방 범위는 해당 타격 방출 때 시전자 위치·방향에 패턴을 놓습니다. 접수 방향 범위는 선택한 방향을 보존하고 방출 때 시전자 위치에 패턴을 놓습니다. 같은 타격의 한 적은 한 번만 맞습니다. 피해와 상태 처리를 마친 뒤 이 계획이 완료되며 연출의 검·입자는 추가 전투 개체가 아닙니다.",
    [
      input("power", "피해 위력", "damage-power"),
      optionalInput("range", "효과 타일 패턴", "tile-pattern"),
      multipleInput("after", "타격 뒤 상태", "hit-status"),
      optionalInput("ramp", "시전 내 후속 증가", "hit-ramp"),
    ],
    [planOut()],
    [
      choice("target", "타격 대상", [
        unresolved,
        ["primary", "고정 주 대상"],
        ["selected-area", "접수 타일 범위"],
        ["forward-area", "방출 시 전방 범위"],
        ["selected-direction-area", "접수 방향·방출 위치 범위"],
      ]),
      choice("damageType", "피해 유형", [
        unresolved,
        ["physical", "물리"],
        ["magic", "마법"],
        ["true", "고정"],
      ]),
    ],
  ),
  define(
    "sequence-plan",
    "두 행동의 순차 계획",
    "스킬 값",
    "첫 행동 전체의 완료 뒤에만 다음 행동을 시작합니다. 첫 행동의 대기·반복·타격 뒤 상태까지 끝나야 하며 취소·비용 부족·필수 대상 소실은 뒤 행동을 시작하지 않습니다. 공유 계획 정의는 실행 회차마다 독립된 상태를 만들고 순환 계획은 허용하지 않습니다.",
    [plan("first", "먼저 할 행동"), plan("next", "다음 행동")],
    [planOut()],
  ),
  define(
    "repeat-plan",
    "유한 반복 타격 계획",
    "스킬 값",
    "개수를 이 반복에 들어올 때 고정합니다. 간격 방식은 첫 타격을 즉시/한 간격 뒤 중 명시하고 타격 완료 뒤 지정 간격을 기다리며 마지막 타격 완료에 끝납니다. 고정 총시간 방식은 첫 타격부터 순서 i의 시각을 floor(i × 총 밀리초 / 개수)로 배치하여 같은 총시간 안에 순서대로 처리하고 총시간 끝에 완료합니다. 같은 시각의 타격도 순번대로 별도 판정합니다. 0회는 첫 대기도 없이 즉시 완료합니다. 반복 본문은 한 번의 즉시 직접 타격이며 중첩 대기·소비를 숨기지 않습니다. 대상 소실·외부 취소는 남은 타격을 버립니다.",
    [
      plan("body", "반복할 타격"),
      count(),
      input("time", "간격 또는 총시간", "seconds"),
    ],
    [planOut()],
    [
      choice("timing", "타격 시간 배치", [
        unresolved,
        ["interval", "타격 사이 간격"],
        ["fixed-total", "고정 총시간 안에 분배"],
      ]),
      choice("first", "첫 반복 타격", [
        unresolved,
        ["immediate", "즉시"],
        ["after-interval", "한 간격 뒤"],
      ]),
    ],
  ),
  define(
    "wait-plan",
    "행동 안의 준비 대기",
    "스킬 값",
    "이번 행동의 일부로 양수 전투 시간을 기다립니다. Pause에는 멈추며 중단·필수 대상 소실은 뒤 행동을 실행하지 않습니다. 대기 자체는 공격·자원 지불·별도 시전이 아닙니다. 소비 단계 안과 밖 중 어디에 연결했는지로 대기 전후 비용 경계가 정해집니다.",
    [input("duration", "준비 시간", "seconds")],
    [planOut()],
  ),
  define(
    "spend-plan",
    "자원 소비 단계",
    "스킬 값",
    "이 단계 시작에 필수 대상·행동 자격과 재고를 검사한 뒤 지정 개수 또는 현재 전량을 한 번 소비하고 본문을 시작합니다. 전량은 0개도 성공하며 실제 소비량을 단계 문맥에 고정합니다. 지정 개수가 부족하면 남은 연속기를 종료하고 아직 시작하지 않은 단계는 소비하지 않습니다. 이미 소비한 재고는 중단·대상 사망에 환불하지 않습니다. 본문 전체의 공격 처리 직후 같은 주 대상에 단계 후 상태를 적용한 다음 바깥 순차 계획으로 돌아갑니다. 그 상태의 count-paid는 이 단계 실제 소비량입니다.",
    [
      bankInput(),
      optionalInput("count", "단계 소비량 계산", "count-rule"),
      plan("body", "소비 뒤 본문"),
      optionalInput("after", "단계 완료 뒤 주 대상 상태", "hit-status"),
    ],
    [planOut()],
    [
      choice("spend", "재고 소비", [
        unresolved,
        ["amount", "지정 개수"],
        ["all", "단계 시작 전량"],
      ]),
    ],
  ),
  define(
    "stance-plan",
    "시간제 방어 자세 계획",
    "스킬 값",
    "양수 전투 시간 동안 시전자에게 받는 모든 피해 감소 기여분을 붙이고 대기합니다. 자연 종료·중단·사망·철수에서 해당 자세의 기여분만 정리합니다. 종료한 뒤 다음 순차 행동으로 진행하며 취소는 다음 행동을 실행하지 않습니다.",
    [input("duration", "자세 유지 시간", "seconds")],
    [planOut()],
    [{ ...number("percent", "자세 피해 감소 (%)"), max: 100 }],
  ),
  define(
    "count-duration",
    "개수에 따른 유지 시간",
    "스킬 값",
    "같은 행동 시작에 고정한 개수로 기본 시간 + 개수 × 추가 시간을 계산하는 유한 유지 시간입니다. 기본 시간은 양수, 개수당 추가 시간은 양수이며 계산 결과도 안전한 정수 밀리초여야 합니다. 미기입 값은 임의 지속 시간으로 대체하지 않습니다. 행동 계획 안에서만 계산합니다.",
    [
      count(),
      input("base", "기본 유지 시간", "seconds"),
      input("perCount", "개수당 추가 시간", "seconds"),
    ],
    [output("value", "유지 시간", "seconds")],
  ),
  define(
    "timed-plan",
    "유한 시간의 주기 타격 계획",
    "스킬 값",
    "시작에 최대 유지 시간을 고정하고 지정 타일의 즉시 타격 본문을 주기마다 실행합니다. 본문은 매번 현재 범위 대상을 다시 조회합니다. 첫 발동을 명시하며 종료 시각과 주기가 겹치면 종료가 우선합니다. 직접 중단·외부 중단·소유자 이탈은 남은 주기를 취소합니다. 이미 붙인 독립 시간제 상태는 원래 수명을 따릅니다.",
    [
      plan("body", "주기마다 할 타격"),
      input("duration", "최대 유지 시간", "seconds"),
      input("interval", "타격 주기", "seconds"),
    ],
    [planOut()],
    [
      choice("first", "첫 타격", [
        unresolved,
        ["immediate", "생성 즉시"],
        ["after-interval", "한 주기 뒤"],
      ]),
    ],
  ),
  define(
    "action-entry",
    "행동 시작 설정",
    "스킬 값",
    "즉시 시작 또는 다음 기본 공격 한 번을 교체할 행동 계획입니다. 다음 기본 공격은 원래 공격 주 대상·무기 출처를 보존하고 준비 중인 공격이 있으면 그 다음 새 공격을 교체합니다. 즉시 주 대상 공격은 명시한 선택 설정을 접수 때 검증·고정하며 타일 공격은 접수 타일을 사용합니다. 직접 연속 행동은 진행 중 이동·기본 공격을 멈추고 하나의 공명 사용 잠금을 유지합니다. 독립 적 상태·공격 준비의 부여 동작은 설치 뒤 완료합니다. 시간제 아군 버프는 기본 공격을 허용하되 남은 버프가 끝날 때까지 공명 사용 잠금을 유지하며, 유한 기회·패링은 각 계획이 정한 기본 행동 허용과 수명을 따릅니다. 고정 주 대상 공격은 주 대상 사망 시 남은 단계/타격을 종료하며 전방/지정 타일 공격은 빈 범위여도 계속합니다. 외부 중단 정책과 자발적 중단은 별개입니다.",
    [plan(), optionalInput("selector", "즉시 주 대상 선택", "enemy-selector")],
    [programOut()],
    [
      choice("start", "행동 시작", [
        unresolved,
        ["next-basic", "다음 기본 공격 교체"],
        ["immediate", "접수 후 즉시"],
      ]),
      choice("interruption", "외부 중단", [
        unresolved,
        ["standard", "Hard CC 취소 · 침묵 중 유지"],
        ["silence", "Hard CC·침묵 취소"],
        ["protected", "진행 중 외부 중단 보호"],
      ]),
    ],
  ),
  define(
    "mode-program",
    "전투 상태별 행동 선택",
    "스킬 값",
    "이번 접수 직전 상태 전환을 먼저 처리하고 비활성/활성에 맞는 행동 시작 설정을 한 번 고정합니다. 접수된 사용 전체가 끝나기 전에는 뒤늦은 전환 예약으로 행동 구성을 교체하지 않습니다. 같은 입력의 비용·공명 설정은 한 번만 접수하며 상태 전환 자체가 무료 시전이 되지 않습니다.",
    [
      input("mode", "전투 상태", "battle-mode"),
      input("normal", "일반 상태 행동", "skill-program"),
      input("active", "활성 상태 행동", "skill-program"),
    ],
    [programOut()],
  ),
  define(
    "run-action",
    "접수한 행동 계획 실행",
    "스킬 동작",
    "같은 공명 접수 성공의 행동 설정을 실행합니다. 사용 자격 검사 때 연결한 프로그램과 같아야 하며 상태 분기·목표를 접수 문맥에서 고정합니다. 다음 기본 공격 대기부터 전체 연속기 종료까지 하나의 사용입니다. 실제 첫 행동 시작에서 재고 값을 고정합니다. 순차·반복·단계 비용·피해·상태는 연결한 계획으로 결정하며 스킬 이름을 해석하지 않습니다. 모든 행동의 자연 완료와 중단을 구분하고 공명 잠금/자세/주기 구독을 정리한 뒤 이번 사용을 한 번 종료합니다. 이미 확정한 피해·소비를 되돌리지 않습니다.",
    [
      incoming(),
      input("use", "이번 사용", "skill-use"),
      input("program", "시작과 행동 설정", "skill-program"),
    ],
    [
      next(),
      output("run", "이번 행동 실행", "action-run"),
      output("completed", "자연 완료", "flow"),
      output("cancelled", "중단", "flow"),
      cue("행동 시작"),
    ],
  ),
  define(
    "action-periodic-self-damage",
    "행동 중 주기 자기 피해",
    "스킬 사건",
    "이번 실행에서 지정한 단일 행동 시작 설정이 선택되고 그 유한 유지형 계획이 실제 시작한 동안만 동작합니다. 첫 온전한 주기 뒤부터 매번 당시 최대 HP 비율의 피해를 자신에게 줍니다. 보호막·해당 피해 유형의 피해 감소가 적용되며 HP 비용으로 바꾸지 않습니다. 행동 종료·사망·철수와 같은 시각이면 종료가 우선합니다. 이 자기 피해로 사망하면 본문과 모든 후속 주기도 즉시 중단합니다.",
    [
      input("run", "이번 행동 실행", "action-run"),
      input("entry", "자기 피해를 받는 행동 설정", "skill-program"),
      input("interval", "자기 피해 주기", "seconds"),
    ],
    [cue("유지 자기 피해")],
    [
      number("percent", "최대 HP 피해 (%)"),
      choice("damageType", "유지 피해 유형", [
        unresolved,
        ["physical", "물리"],
        ["magic", "마법"],
      ]),
    ],
  ),
  define(
    "action-stop-input",
    "유지 행동 직접 중단 입력",
    "스킬 사건",
    "이번 실행에서 유한 유지형 행동이 선택되어 진행 중일 때만 직접 중단 메뉴를 제공합니다. 이 입력은 새 시전이 아니며 공명 조건을 요구하지 않습니다. 자연 완료를 발생시키지 않고 해당 실행만 한 번 중단합니다. 순차 낙검·다음 기본 공격 대기 등의 다른 행동을 이 메뉴로 취소하지 않습니다.",
    [input("run", "이번 행동 실행", "action-run")],
    [cue("직접 중단")],
  ),
];
