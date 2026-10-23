import { conditionalEffectCatalog } from "./conditional-effect-catalog";
import {
  weaponStateCatalog,
  weaponStatePortTypes,
} from "./weapon-state-catalog";
import {
  equipmentModifierCatalog,
  equipmentModifierPortTypes,
} from "./equipment-modifier-catalog";
import {
  mobileEffectCatalog,
  mobileEffectPortTypes,
} from "./mobile-effect-catalog";
import {
  reactiveStatusCatalog,
  reactiveStatusPortTypes,
} from "./reactive-status-catalog";
import { offensiveCatalog } from "./offensive-catalog";
import { guardCatalog, guardPortTypes } from "./guard-catalog";
import { protectionCatalog } from "./protection-catalog";
import { markCatalog, markPortTypes } from "./mark-catalog";
import { burstCatalog, burstPortTypes } from "./burst-catalog";
import { portalCatalog, portalPortTypes } from "./portal-catalog";
import { inputPlanCatalog, inputPlanPortTypes } from "./input-plan-catalog";
import { spawnCatalog, spawnPortTypes } from "./spawn-catalog";
import { projectileCatalog, projectilePortTypes } from "./projectile-catalog";
import { actionPlanCatalog, actionPlanPortTypes } from "./action-plan-catalog";
import { relationCatalog, relationPortTypes } from "./relation-catalog";
import { inventoryCatalog, inventoryPortTypes } from "./inventory-catalog";
import type { Definition } from "../types";
import {
  input,
  output,
  number,
  choice,
  unresolved,
  define,
  state,
  activation,
  incoming,
  next,
  cue,
} from "./skill-definition";
import { resonanceCatalog, resonancePortTypes } from "./resonance-catalog";
import { spatialCatalog, spatialPortTypes } from "./spatial-catalog";
import { groundCatalog, groundPortTypes } from "./ground-catalog";
import { channelCatalog, channelPortTypes } from "./channel-catalog";
import { targetCatalog, targetPortTypes } from "./target-catalog";

import { resourceCatalog, resourcePortTypes } from "./resource-catalog";

import { phaseCatalog, phasePortTypes } from "./phase-catalog";

import { pairCatalog, pairPortTypes } from "./pair-catalog";

// Authoring semantics only. No battle execution or Core format is implied.
export const skillPortTypes = [
  ...weaponStatePortTypes,
  ["projectile-wound", "출처별 탄환 약화 선언"],
  ...equipmentModifierPortTypes,
  ...mobileEffectPortTypes,
  ...reactiveStatusPortTypes,
  ...guardPortTypes,
  ...markPortTypes,
  ...burstPortTypes,
  ...portalPortTypes,
  ...inputPlanPortTypes,
  ...spawnPortTypes,
  ...projectilePortTypes,
  ...actionPlanPortTypes,
  ...relationPortTypes,
  ...inventoryPortTypes,
  ...resonancePortTypes,
  ...spatialPortTypes,
  ...groundPortTypes,
  ...channelPortTypes,
  ...targetPortTypes,
  ...pairPortTypes,
  ...phasePortTypes,
  ...resourcePortTypes,
  ["skill-request", "사용 요청"],
  ["skill-vacancy", "비활성 조회 결과"],
  ["skill-start-permit", "첫 사용 허가"],
  ["skill-followup-permit", "후속 사용 허가"],
  ["skill-state", "상태 참조"],
  ["skill-activation", "이번 활성"],
  ["seconds", "시간 (초)"],
  ["attack-profile", "기본 공격 설정"],
  ["area-profile", "전방 범위"],
  ["damage-power", "피해 위력"],
  ["presentation-event", "연출 사건"],
] as const;

export const skillCatalog: Definition[] = [
  ...weaponStateCatalog,
  ...conditionalEffectCatalog,
  ...equipmentModifierCatalog,
  ...markCatalog,
  ...burstCatalog,
  define(
    "state",
    "상태 선언",
    "스킬 상태",
    "적용 동작과 사건들이 함께 참조할 상태를 선언합니다. 유지 시간과 공격 설정은 연결된 노드에서 편집합니다. 상태 참조 연결은 실행 순서가 아닙니다.",
    [],
    [output("state", "상태 참조", "skill-state")],
  ),
  define(
    "seconds",
    "시간 값",
    "스킬 값",
    "전투 시간 기준 초 단위 값입니다. 0.001초(1밀리초) 단위까지 허용하며 더 세밀한 값은 반올림 없이 거절합니다. 빈 값은 미정이며 0초로 대신하지 않습니다.",
    [],
    [output("value", "시간 (초)", "seconds")],
    [{ ...number("value", "시간 (초)"), unit: "seconds" }],
  ),
  define(
    "attack-profile",
    "관통 기본 공격 설정",
    "스킬 값",
    "직선 관통 기본 공격의 설정입니다. 장착 무기의 직접 기본 공격 출처와 공격 주기를 보존합니다. 관통이 곧 물리 투사체를 뜻하지는 않습니다. 무기 효과는 실제 HP 피해를 공격별로 합산하며, 관통 유물 이력도 공격마다 새로 시작합니다.",
    [],
    [output("profile", "공격 설정", "attack-profile")],
    [
      number("length", "길이 (타일)"),
      number("width", "폭 (타일)"),
      number("power", "공격 위력"),
      {
        id: "weaponEffects",
        label: "장착 무기 효과",
        valueType: "boolean",
        default: true,
      },
      {
        id: "piercingRelics",
        label: "호환 관통 유물",
        valueType: "boolean",
        default: true,
      },
    ],
  ),
  define(
    "request",
    "스킬 입력 사건",
    "스킬 사건",
    "스킬 입력 하나를 받습니다. 연결한 대상 선택 설정과 입력 ID·직원 ID·장착된 스킬 출처 ID·입력 시각을 보존합니다. 대상 선택은 접수 전이며 자격·범위 재검증과 비용 지불은 접수에서 함께 확정합니다. 설정이 없는 이전 그래프의 대상 방식은 외부 정의에 남아 있으며 자기 자신으로 추정하지 않습니다.",
    [
      {
        ...input("selection", "대상 선택 설정", "target-selection"),
        required: false,
      },
    ],
    [
      output("out", "입력 도착", "flow"),
      output("request", "사용 요청", "skill-request"),
    ],
  ),
  define(
    "lookup",
    "이번 상태 활성 조회",
    "스킬 조건",
    "요청의 직원·장착 스킬 출처와 상태 선언이 모두 같은 활성을 조회합니다. 활성 있음과 없음 중 한 경로만 진행합니다. 이번 활성은 있음 경로에서만, 비활성 결과는 없음 경로에서만 유효합니다. 조회는 비용·수명·마무리 권리를 바꾸지 않습니다.",
    [incoming(), input("request", "사용 요청", "skill-request"), state()],
    [
      output("present", "활성 있음", "flow"),
      output("absent", "활성 없음", "flow"),
      output("activation", "이번 활성", "skill-activation"),
      output("vacancy", "비활성 결과", "skill-vacancy"),
    ],
  ),
  define(
    "qualify",
    "첫 사용 자격 확인",
    "스킬 조건",
    "비활성 결과의 요청에 공명 만충과 공통 스킬 사용 자격을 확인합니다. 통과 시 해당 요청·상태에만 유효한 첫 사용 허가를 만듭니다. 여기서는 비용을 지불하지 않습니다. 실패하면 후속 흐름과 허가가 없습니다.",
    [incoming(), input("vacancy", "비활성 결과", "skill-vacancy")],
    [
      output("out", "자격 통과", "flow"),
      output("permit", "첫 사용 허가", "skill-start-permit"),
    ],
  ),
  define(
    "begin",
    "비용 지불과 상태 시작",
    "스킬 동작",
    "첫 사용 허가의 요청·상태·자격·공명을 다시 확인한 뒤, 비용 지불·새 활성 생성·타이머와 공격 교체 설치를 한 번에 확정합니다. 같은 요청의 중복 확정이나 기존 활성 갱신은 거부합니다. 실패 시 비용과 상태 모두 바뀌지 않습니다. 타이머는 정지 중 흐르지 않는 전투 시간입니다. 즉시 추가 공격하거나 공격 주기를 초기화하지 않습니다. 종료 시 이번 활성의 교체만 해제합니다.",
    [
      incoming(),
      input("permit", "첫 사용 허가", "skill-start-permit"),
      state(),
      input("duration", "유지 시간", "seconds"),
      input("profile", "공격 설정", "attack-profile"),
    ],
    [output("activation", "생성한 활성", "skill-activation"), cue("변형 시작")],
  ),
  define(
    "permission",
    "재입력 허용 확인",
    "스킬 조건",
    "요청과 이번 활성의 직원·스킬 출처를 대조합니다. 공통 수동 재입력은 공격 태세에서 행동 가능할 때만 허용하고 Hard CC·Silence 중에는 거절합니다. 해당 스킬 자신의 사용 잠금만 예외입니다. 공명 만충·추가 소비 없이 해당 요청·활성의 후속 허가를 만듭니다. 거부하거나 정책이 미정이면 진행하지 않습니다.",
    [incoming(), input("request", "사용 요청", "skill-request"), activation()],
    [
      output("out", "허용", "flow"),
      output("permit", "후속 사용 허가", "skill-followup-permit"),
    ],
    [
      choice("permission", "재입력 허용 정책", [
        unresolved,
        ["manual-followup", "공통 수동 재입력"],
        ["skill-gate", "공통 스킬 제한 적용"],
        ["active-use", "활성 사용의 후속 입력"],
      ]),
    ],
  ),
  define(
    "claim",
    "마무리 권리 확보",
    "스킬 동작",
    "후속 허가와 이번 활성의 미사용 권리를 재검증해 원자적으로 확보합니다. 만료 우선은 처리 시각이 만료 시각과 같거나 늦으면 거절합니다. 같은 활성에서는 한 요청만 성공하며 실패·중복에는 다음 흐름이 없습니다. 공명·유지 시간을 갱신하지 않습니다. 성공 시 수락 시각과 잔여 시간을 보존하고 값 고정과 수동 종료로 진행합니다.",
    [incoming(), input("permit", "후속 사용 허가", "skill-followup-permit")],
    [
      output("out", "권리 확보", "flow"),
      output("activation", "이번 활성", "skill-activation"),
    ],
    [
      choice("expiryOrder", "만료와 동시 입력", [
        unresolved,
        ["expiry-first", "만료 우선"],
        ["input-first", "입력 우선"],
      ]),
    ],
  ),
  define(
    "use",
    "첫 스킬 사용",
    "스킬 사건",
    "이전 그래프의 복합 진입점입니다. 새 그래프에서는 입력·활성 조회·자격 확인·상태 시작을 연결합니다. 공명 만충과 공통 사용 자격을 확인하고 유효한 입력에서 정상 공명 비용을 지불하는 진입점입니다. 실패·취소에는 후속 실행이 없습니다.",
    [],
    [output("out", "사용 수락", "flow")],
  ),
  define(
    "overlay",
    "기본 공격 교체",
    "스킬 동작",
    "실행 입력을 받으면 참조한 상태에 유지 시간과 공격 설정을 적용하는 동작입니다. 적용 뒤 변형 시작 사건을 연출에 전달합니다. 즉시 추가 공격하거나 공격 주기를 초기화하지 않습니다. 직원 위치·저지 수·무기 종류를 바꾸지 않습니다. 상태 종료 시 이 출처의 교체를 해제합니다.",
    [
      incoming(),
      state(),
      input("duration", "유지 시간", "seconds"),
      input("profile", "공격 설정", "attack-profile"),
    ],
    [cue("변형 시작")],
  ),
  define(
    "basic-event",
    "변형 중 기본 공격",
    "스킬 사건",
    "참조한 상태가 적용된 동안 공통 행동 규칙에 따라 발생하는 개별 기본 공격을 표시합니다. 실행은 공격 교체 설정을 따르며 이 노드가 추가 공격을 생성하지 않습니다.",
    [state()],
    [cue("공격 표시")],
  ),
  define(
    "reinput",
    "활성 상태 재입력",
    "스킬 사건",
    "이전 그래프의 복합 진입점입니다. 새 그래프에서는 입력·활성 조회·허용 확인·권리 확보를 나누어 연결합니다. 현재 활성에서 아직 사용하지 않은 마무리를 한 번 수락합니다. 별도 Hello 시전·지속 갱신·추가 공명 소비가 아닙니다. 군중 제어와 침묵의 허용 정책은 별도 지정합니다.",
    [state()],
    [
      output("out", "재입력 수락", "flow"),
      output("activation", "이번 활성", "skill-activation"),
    ],
    [
      choice("permission", "재입력 허용 정책", [
        unresolved,
        ["skill-gate", "공통 스킬 제한 적용"],
        ["active-use", "활성 사용의 후속 입력"],
      ]),
    ],
  ),
  define(
    "capture-time",
    "남은 시간 고정",
    "스킬 동작",
    "마무리 권리를 확보한 이번 활성에서 선택한 시점의 잔여 시간을 한 번 고정합니다. 수락 시점은 권리 확보 시 보존한 값이며, 방출 시점은 방출 직전의 유효성을 다시 확인해야 합니다. 이후 연출이나 대상별 처리에서 다시 계산하지 않습니다. 만료된 상태에 0초 마무리를 허용하는 연산은 아닙니다.",
    [incoming(), activation()],
    [next(), output("seconds", "남은 시간", "seconds")],
    [
      choice("moment", "시간 고정 시점", [
        unresolved,
        ["accepted", "재입력 수락 시점"],
        ["release", "마무리 방출 시점"],
      ]),
    ],
  ),
  define(
    "finish",
    "상태 종료",
    "스킬 동작",
    "참조한 이번 활성과 그 출처의 타이머·공격 교체를 종료합니다. 수동 종료는 확보한 마무리 권리를 요구합니다. 이미 종료된 활성에는 후속 실행과 종료 표시를 중복 발생시키지 않습니다. 자연 만료와 수동 마무리는 서로 다른 경로입니다. 종료 자체에 추가 피해·부활을 넣지 않습니다.",
    [incoming(), activation()],
    [next(), cue("종료 표시")],
    [
      choice(
        "reason",
        "종료 원인",
        [
          ["manual", "수동 마무리"],
          ["expiry", "자연 만료"],
          ["owner-exit", "소유자 이탈 / 전투 종료"],
        ],
        "manual",
      ),
    ],
  ),
  define(
    "expiry",
    "상태 시간 만료",
    "스킬 사건",
    "참조한 변형의 전투 시간이 다 되었을 때 발생합니다. 자동 마무리를 보상하지 않습니다. 만료 시각과 같은 재입력은 거절하며 자동 마무리를 만들지 않습니다.",
    [state()],
    [
      output("out", "만료", "flow"),
      output("activation", "이번 활성", "skill-activation"),
    ],
  ),
  define(
    "owner-exit",
    "상태 소유자 이탈",
    "스킬 사건",
    "철수·전투불능·전투 종료에 연결할 상태 정리 경로입니다. 이번 활성을 종료하고 출처의 교체만 해제합니다. 무료 마무리나 부활을 만들지 않습니다.",
    [state()],
    [
      output("out", "정리", "flow"),
      output("activation", "이번 활성", "skill-activation"),
    ],
    [
      choice("policy", "이탈 처리 정책", [
        unresolved,
        ["end-active", "이번 활성 종료"],
      ]),
    ],
  ),
  define(
    "time-power",
    "시간에 따른 위력",
    "스킬 값",
    "고정한 잔여 시간으로 위력을 정합니다. 기본 위력·초당 증가량·상한은 0 이상의 정수입니다. 밀리초 정밀도의 시간을 그대로 계산에 사용하고, 상한 적용을 포함한 최종 위력을 한 번 올림합니다. 식과 수치는 작성값이며 예제에서 새 밸런스를 확정하지 않습니다.",
    [input("seconds", "남은 시간", "seconds")],
    [output("power", "위력", "damage-power")],
    [
      choice("curve", "위력 곡선", [
        unresolved,
        ["linear", "기본 위력 + 초당 증가"],
        ["capped-linear", "선형 증가 후 상한"],
      ]),
      { ...number("base", "기본 위력"), integer: true },
      { ...number("rate", "초당 위력 증가"), integer: true },
      { ...number("cap", "위력 상한"), integer: true },
    ],
  ),
  define(
    "area",
    "전방 범위 설정",
    "스킬 값",
    "선택한 타일이 실제 효과 범위입니다. 장판을 포함한 스킬 범위는 타일 패턴을 사용합니다. 위쪽이 전방이며 ◎는 기준 타일입니다. 연출 크기는 판정을 바꾸지 않습니다.",
    [],
    [output("area", "전방 범위", "area-profile")],
    [
      {
        id: "pattern",
        label: "효과 타일 패턴",
        valueType: "tile-pattern",
        default: "",
      },
      {
        id: "includeAnchor",
        label: "기준 타일 포함",
        valueType: "boolean",
        default: false,
      },
      choice("facing", "방향 고정", [
        unresolved,
        ["accepted", "재입력 수락 방향"],
        ["release", "방출 시 바라보는 방향"],
      ]),
    ],
  ),
  define(
    "area-damage",
    "전방 광역 일격",
    "스킬 동작",
    "흐름이 보존한 직원·장착 스킬 출처를 사용해 연결한 위력과 범위로 파편 직접 스킬 피해를 한 번 적용합니다. 무기 기본 공격 흡혈이나 Hello의 관통 순서를 상속하지 않습니다. 효과 처리 완료 뒤 다음 흐름으로 진행합니다.",
    [
      incoming(),
      input("power", "위력", "damage-power"),
      input("area", "전방 범위", "area-profile"),
    ],
    [
      next(),
      cue("일격 표시"),
      output("completed", "일격 종료", "presentation-event"),
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
    "cue",
    "연출 표시",
    "연출",
    "연결된 게임 사건을 보여줄 외형·동작 설명입니다. 게임의 시간·위치·타격 수·피해를 변경하거나 다음 게임 동작을 실행하지 않습니다. 실제 애니메이션 재생기는 연결하지 않았습니다.",
    [{ ...input("event", "연출 사건", "presentation-event"), multiple: true }],
    [],
    [
      {
        id: "description",
        label: "연출 내용",
        valueType: "text",
        default: "",
        multiline: true,
      },
    ],
  ),
  ...resonanceCatalog,
  ...spatialCatalog,
  ...groundCatalog,
  ...channelCatalog,
  ...targetCatalog,
  ...protectionCatalog,
  ...offensiveCatalog,
  ...guardCatalog,
  ...mobileEffectCatalog,
  ...reactiveStatusCatalog,
  ...pairCatalog,
  ...phaseCatalog,
  ...resourceCatalog,
  ...portalCatalog,
  ...inputPlanCatalog,
  ...spawnCatalog,
  ...projectileCatalog,
  ...actionPlanCatalog,
  ...relationCatalog,
  ...inventoryCatalog,
  ...skillPortTypes.map(([type, label]) => ({
    ...define(
      `reroute-${type}`,
      `${label} 경유`,
      "스킬 연결",
      "같은 의미와 단위를 유지하는 연결 경유점입니다.",
      [input("in", "입력", type)],
      [output("out", "출력", type)],
    ),
    kind: `reroute-${type}`,
  })),
];
