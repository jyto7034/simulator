import type { Definition } from "../types";
import {
  define,
  input,
  output,
  number,
  choice,
  unresolved,
} from "./skill-definition";
export const guardPortTypes = [
  ["damage-guard", "출처별 상시 피해 감면"],
] as const;
export const guardCatalog: Definition[] = [
  define(
    "guard-charge",
    "감면 피해 축적 · 성공 받아치기 방출",
    "스킬 상태",
    "장착 조건을 만족할 때 연결한 상시 감면이 실제 막은 적 직접 피해만 착용자별로 축적합니다. 다른 감면·패링 무효화·면역·반사와 그 파생 피해는 제외합니다. 실패는 보존하고 성공은 전량 소비합니다. 주 대상에는 기본 받아치기와 축적 배율 피해를 합치고, 주변에는 축적 파급 배율만 적용합니다. 범위는 원래 공격자의 타일에 고정되며 주 대상을 다시 타격하지 않습니다. 철수·사망·침식·전투 종료에는 축적분을 정리합니다.",
    [
      input("guard", "축적할 감면 출처", "damage-guard"),
      input("parry", "성공 시 방출할 받아치기", "action-plan"),
      input("condition", "실제 장착 유물", "equipment-match"),
      input("range", "주 대상 주변의 파급 타일", "tile-pattern"),
    ],
    [],
    [
      { ...number("capacity", "축적 상한"), min: 1, integer: true },
      { ...number("factor", "주 대상 축적 피해 배율"), min: 0 },
      { ...number("splash", "주변 축적 피해 배율"), min: 0 },
    ],
  ),
  define(
    "passive-guard",
    "배치 중 출처별 피해 감소",
    "스킬 상태",
    "이 파편 사본을 장착한 소유자가 살아서 배치된 동안 받는 피해 감소 기여를 설치합니다. 공명·시전 상태와 무관하며 기본 행동·저지·배치를 바꾸지 않습니다. HP 비용·최대 HP 희생·직접 처형에는 적용하지 않습니다. 중단/복원은 이 선언의 기여만 다루고 방어구·다른 출처 감면을 제거하지 않습니다. 사망·철수·장착 해제·전투 종료에 정리합니다.",
    [],
    [output("guard", "이 출처의 감면", "damage-guard")],
    [{ ...number("percent", "받는 피해 감소 (%)"), max: 100 }],
  ),
  define(
    "direct-reflection",
    "적 직접 공격의 확률 반사",
    "스킬 상태",
    "연결한 감면 출처가 배치 중일 때 적대적 직접 공격의 대상별 첫 양수 HP 손실 구간에서 결정론적 확률을 한 번 판정합니다. 방어·감면·보호막 적용 뒤 실제 HP 손실에 비율을 적용해 살아 있는 원래 공격자에게 반사 피해를 한 번 요청합니다. 원래 피해를 취소하지 않습니다. 자기 피해·비용·지속피해·반사 및 그 파생 피해는 반사 출처가 아니며 재귀 반사를 금지합니다. 연결한 감면을 참조하는 패링 성공의 첫 타격과 면역 중에는 발동하지 않습니다. 감면 실패 빈틈 자체는 반사를 없애지 않습니다.",
    [input("guard", "같은 출처의 감면", "damage-guard")],
    [],
    [
      { ...number("chance", "발동 확률 (%)"), max: 100 },
      number("percent", "원래 피해의 반사 비율 (%)"),
      choice("damageType", "반사 피해 유형", [
        unresolved,
        ["original", "원래 공격 유형"],
        ["physical", "물리"],
        ["magic", "마법"],
      ]),
    ],
  ),
  define(
    "parry-plan",
    "첫 직접 공격 받아내기",
    "스킬 값",
    "유한 판정 창에서 자신에게 들어오는 첫 적 직접 공격을 피해·행동제어 적용 전에 한 번 확보합니다. 성공 타격부터 지정 시간 동안 모든 피해와 행동제어를 차단하고 원래 공격자에게 받아치기를 한 번 적용합니다. HP 비용과 다른 버프·디버프 부착/해제는 차단하지 않습니다. 추가 피격은 면역을 연장하거나 받아치기/반사를 반복하지 않습니다. 판정 창 만료는 실패이며 연결한 감면 기여만 실패 시간 동안 제거한 뒤 복원합니다. 피해를 새로 만들거나 공명을 환불하지 않습니다. 창 중 기본 공격·저지는 유지하고 철수·사망·전투 종료에 창과 면역/빈틈을 정리합니다. 성공 후 효과는 Hard CC·Silence로 소급 취소하지 않습니다.",
    [
      input("guard", "실패 때 비활성화할 감면", "damage-guard"),
      input("window", "받아내기 판정 창", "seconds"),
      input("immunity", "성공 뒤 피해·행동제어 면역", "seconds"),
      input("failure", "실패 감면 공백", "seconds"),
      input("power", "받아치기 위력", "damage-power"),
    ],
    [output("plan", "행동 계획", "action-plan")],
    [
      choice("damageType", "받아치기 피해 유형", [
        unresolved,
        ["physical", "물리"],
        ["magic", "마법"],
      ]),
    ],
  ),
];
