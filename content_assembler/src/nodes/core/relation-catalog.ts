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
} from "./skill-definition";
import { bankInput, grantOutput } from "./inventory-catalog";

export const relationPortTypes = [
  ["ally-relation", "사본별 아군 관계"],
  ["relation-death", "이번 관계의 HP 0 퇴각"],
  ["battle-mode", "전투 상태 전환"],
] as const;
const relation = () => input("relation", "아군 관계", "ally-relation");
const mode = () => input("mode", "전투 상태", "battle-mode");
export const relationCatalog: Definition[] = [
  define(
    "battle-mode",
    "전투 중 단방향 상태",
    "스킬 상태",
    "파편 사본의 전투 로컬 비활성/전환 예약/활성 상태입니다. 비활성으로 시작하고 한 번 활성화되면 전투 종료까지 유지합니다. 관계 대상의 부활·해제·교체, 소유자 사망·재배치로 전투 기록을 초기화하지 않습니다. 예약은 죽은 소유자의 행동·부활을 만들지 않습니다. 새 전투는 비활성입니다.",
    [],
    [output("mode", "상태 정의", "battle-mode")],
  ),
  define(
    "ally-relation",
    "메뉴의 단일 아군 관계",
    "스킬 상태",
    "살아 있는 배치 아군 한 명을 메뉴로 지정하며 자신도 선택할 수 있습니다. 같은 대상을 다시 고르면 해제하고 다른 대상은 기존 관계를 정리한 뒤 교체합니다. 대상별 관계 회차를 구분하고 새 부착의 주기는 0부터 시작합니다. 대상/소유자 이탈과 전투 종료에 관계·종속 효과를 정리합니다. 활성 차단 상태에서는 새 지정을 거절합니다. 메뉴는 공명 스킬이 아니며 비용·새 행동·기본 공격 교체를 만들지 않습니다. 소유자가 생존·배치·행동 가능해야 하며 진행 중 자기 스킬의 잠금만 메뉴 조작에서 제외합니다.",
    [optionalInput("blocked", "활성 시 지정 금지", "battle-mode")],
    [output("relation", "관계 정의", "ally-relation"), cue("관계 변경")],
    [{ id: "entry", label: "메뉴 항목 ID", valueType: "text", default: "" }],
  ),
  define(
    "relation-reduction",
    "관계 대상 피해 감소",
    "스킬 상태",
    "현재 관계 대상에게 받는 모든 피해 감소 기여분을 제공합니다. 관계 해제·교체·종료 시 해당 출처의 기여분만 제거합니다. 같은 종류의 다른 출처와는 공통 피해 감소 합산/상한 규칙을 따르며 대상의 기존 능력치를 덮어쓰지 않습니다.",
    [relation()],
    [cue("피해 감소")],
    [{ ...number("percent", "받는 피해 감소 (%)"), max: 100 }],
  ),
  define(
    "relation-emergency-shield",
    "전투당 한 번 긴급 보호막",
    "스킬 상태",
    "살아 있는 관계 대상의 HP가 최대 HP의 지정 비율 이하이면 최대 HP 비율의 보호막을 부여합니다. 이미 조건을 만족한 대상에게 새로 부착해도 즉시 검사합니다. HP 0 사망을 취소하거나 부활시키지 않습니다. 사용 기록은 관계 회차가 아닌 파편 사본의 전투에 남으며 해제·대상 교체·소유자 재배치로 초기화하지 않습니다. 보호막은 부여 시 최대 HP로 고정하고 독립된 시간제 효과로 남아 관계 교체로 옮기지 않습니다. 대상 이탈·전투 종료에는 정리합니다.",
    [relation(), input("duration", "보호막 지속 시간", "seconds")],
    [cue("긴급 보호막")],
    [
      { ...number("threshold", "발동 HP 상한 (%)"), max: 100 },
      { ...number("shield", "최대 HP 대비 보호막 (%)"), max: 100 },
    ],
  ),
  define(
    "relation-periodic",
    "관계 부착 주기",
    "스킬 사건",
    "현재 살아 있는 관계의 부착 시점부터 양수 전투 시간 주기를 셉니다. 완료마다 같은 파편 저장소의 획득 권리를 내고 다음 주기를 시작합니다. 교체·해제·대상 소멸은 진행을 버리고 재부착은 새 주기입니다. Pause·소멸 뒤 콜백은 권리를 만들지 않습니다. 한 관계·저장소에 중복 주기를 만들지 않습니다.",
    [relation(), bankInput(), input("interval", "관계 주기", "seconds")],
    [next(), grantOutput()],
  ),
  define(
    "relation-hp-zero",
    "관계 대상 HP 0 퇴각",
    "스킬 사건",
    "현재 관계 대상이 HP 0으로 전투불능 퇴각할 때 이전 관계 회차를 보존해 한 번 알립니다. 수동 철수·해제·교체·전투 종료는 이 사건이 아닙니다. 자기 자신이 대상인 경우에도 소유자 사망 처리가 우선이며 죽은 소유자에게 새 행동을 시작하지 않습니다.",
    [relation()],
    [next(), output("event", "HP 0 퇴각 사건", "relation-death")],
  ),
  define(
    "defer-mode",
    "다음 행동 직전 전환 예약",
    "스킬 동작",
    "같은 관계 HP 0 사건의 전투 상태를 예약합니다. 현재 기본 공격 또는 접수된 스킬 사용 전체가 끝난 뒤 다음 행동의 자격 검사 전에 한 번 활성화합니다. 연속기의 중간 타격·단계 경계에서 끼어들지 않습니다. 이미 예약/활성이면 무변경입니다. 생존 소유자가 없으면 실행하지 않고, 활성 전환 흐름을 마친 뒤에도 살아 있어야 다음 행동을 시작합니다.",
    [incoming(), input("event", "HP 0 퇴각 사건", "relation-death"), mode()],
    [next()],
  ),
  define(
    "mode-entered",
    "전투 상태 진입 사건",
    "스킬 사건",
    "예약한 전환이 행동 사이에서 활성화된 직후 한 번 발생합니다. 이 동기 흐름을 모두 처리한 뒤 다음 행동의 자격을 검사합니다. 전환 자체는 스킬 시전·무료 공격·공명 지불이 아니며 종료한 전투의 예약을 실행하지 않습니다.",
    [mode()],
    [next(), cue("상태 진입")],
  ),
  define(
    "relation-clear",
    "현재 아군 관계 해제",
    "스킬 동작",
    "해당 사본의 현재 관계와 그 종속 주기·피해 감소를 정리합니다. 이미 해제됐으면 무변경입니다. 전투당 긴급 보호막 사용 기록과 이미 독립 부여한 보호막을 지우거나 새 대상에게 이전하지 않습니다.",
    [incoming(), relation()],
    [next()],
  ),
  define(
    "self-stock-damage",
    "개수에 따른 자기 HP 피해",
    "스킬 동작",
    "같은 상태 진입 사건에서 현재 재고를 한 번 고정하고 최대 HP × 개수 × 비율만큼 자기 피해를 처리합니다. 고정·보호막 무시는 방어/보호막을 거치지 않으며 HP 0 퇴각이 가능합니다. 체력 지불·최대 HP 감소가 아니며 재고를 소비하지 않습니다. 이 흐름의 count-stock은 행동 계획과 달리 이 진입 사건에서 고정합니다. 중복 사건은 피해를 반복하지 않습니다.",
    [incoming(), input("count", "재고 개수 계산", "count-rule")],
    [next(), cue("자기 피해")],
    [
      number("percent", "개수당 최대 HP 피해 (%)"),
      choice("damage", "자기 피해 판정", [
        unresolved,
        ["fixed-bypass", "고정 피해 · 보호막 무시"],
      ]),
    ],
  ),
];
