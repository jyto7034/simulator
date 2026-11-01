import type { Definition } from "../types";
import {
  define,
  input,
  output,
  number,
  choice,
  unresolved,
} from "./skill-definition";
const plan = () => output("plan", "행동 계획", "action-plan");
export const offensiveCatalog: Definition[] = [
  define(
    "limited-range",
    "타일 범위 거리 상한",
    "스킬 값",
    "연결한 실제 타일 패턴을 그대로 보존하되 기준 타일에서 각 포함 타일까지 Chebyshev 거리 max(|x|, |y|)가 상한 이하여야 합니다. 범위를 자동으로 채우거나 잘라내지 않고 벗어난 타일을 진단합니다. 기본 콘텐츠의 범위 경계이며 유물 변형 뒤 범위의 상한으로 암묵 재적용하지 않습니다.",
    [input("base", "실제 효과 타일", "tile-pattern")],
    [output("range", "경계 검증한 타일 패턴", "tile-pattern")],
    [number("tiles", "최대 타일 거리")],
  ),
  define(
    "weapon-power",
    "무기 공격력 비례 위력",
    "스킬 값",
    "해당 타격 방출 때 소유자의 장착 무기 Attack과 능력치 기여를 읽어 ceil(Attack × 계수) 위력을 만듭니다. 독립 전달 효과는 최초 방출 때 값을 고정합니다. 피해 유형·방어·실제 HP 손실은 후속 피해 판정이 소유합니다. 기본 공격 피해 전용 modifier는 이 스킬 위력에 적용하지 않습니다.",
    [],
    [output("power", "피해 위력", "damage-power")],
    [number("coefficient", "무기 공격력 계수")],
  ),
  define(
    "missing-hp-power",
    "대상별 잃은 HP 위력 증가",
    "스킬 값",
    "각 대상의 해당 타격 피해 직전 (최대 HP - 현재 HP) / 최대 HP 비율을 읽습니다. ceil(기본 위력 × (1 + min(상한, 부족 HP 비율 × 최대 증가율) / 100))로 계산합니다. 이전 타격의 피해·회복을 반영하고 대상별로 독립 계산합니다. 처형·추가 타격·대상 약화나 전투 누적 스택이 아닙니다. 실제 타격 대상이 있는 피해 위력 문맥에만 사용할 수 있습니다.",
    [input("base", "기본 위력", "damage-power")],
    [output("power", "대상별 위력", "damage-power")],
    [
      number("percent", "HP 0% 기준 증가율 (%)"),
      number("cap", "최대 증가율 상한 (%)"),
    ],
  ),
  define(
    "hit-pull",
    "타격 뒤 중심으로 당기기",
    "스킬 값",
    "유효 타격 뒤 살아 있는 대상의 실제 위치를 이번 타격 중심으로 지정 거리만큼 당깁니다. 중심을 넘지 않고 강제 이동 면역·지형·충돌을 따릅니다. 이동이 거절되어도 앞선 피해를 취소하지 않습니다. 끌어당김 자체는 기절이 아니며 기절 성공 사건을 만들지 않습니다. 대상 조회 범위와 실제 이동 거리는 별개입니다.",
    [],
    [output("status", "타격 뒤 이동", "hit-status")],
    [number("tiles", "최대 당기는 거리 (타일)")],
  ),
  define(
    "self-heal-plan",
    "자기 최대 HP 비례 회복",
    "스킬 값",
    "현재 살아 있는 시전자에게 당시 최대 HP의 비율만큼 회복을 요청하고 실제 HP 상한으로 제한합니다. 피해량·적 수와 비례하지 않고 이 계획 실행당 한 번입니다. 사망·이탈한 시전자를 복구하거나 초과 회복을 보호막으로 만들지 않습니다.",
    [],
    [plan()],
    [number("percent", "자기 최대 HP 회복 (%)")],
  ),
  define(
    "on-hit-plan",
    "유효 적중 때 후속 행동",
    "스킬 값",
    "본문 계획의 모든 판정이 끝난 뒤 적 한 명 이상에게 유효한 타격이 있었으면 후속 계획을 한 번 실행합니다. 직접 피해가 실제 HP 또는 보호막을 줄인 판정만 유효 적중으로 셉니다. 대상 수·처치 수·피해 콜백 수로 반복하지 않습니다. 빈 범위·빗나감·전부 무효인 타격에는 후속을 실행하지 않습니다. 이 실행의 직접 본문 결과만 집계하고 소환·장비·다른 시전의 결과를 섞지 않습니다. 후속의 결과로 본문 집계를 다시 열지 않습니다.",
    [
      input("body", "관찰할 본문", "action-plan"),
      input("after", "적중 성공 뒤 1회", "action-plan"),
    ],
    [plan()],
  ),
  define(
    "delayed-area-plan",
    "고정한 목표 위치의 지연 범위 타격",
    "스킬 값",
    "접수 또는 이번 기회 시작의 지정한 시점에 선택기로 적을 고르고 현재 목표 타일을 고정합니다. 접수 고정은 시작 설정의 동일 선택기를 사용하며 유효 목표가 없으면 비용을 소비하지 않습니다. 지연 뒤 그 위치의 현재 적 각각에게 피해를 한 번 적용하며 원래 목표의 이동·사망으로 중심을 옮기거나 취소하지 않습니다. 선택 대상이 없으면 무적중 완료합니다. 경로 중간에 관통 피해·저지·HP 개체를 만들지 않습니다. 지연은 방출 뒤 독립 전달 수명이며 Hard CC·Silence·소유자 이탈로 취소하지 않고 전투 종료에 정리합니다. 피해 위력은 시작 때 고정하되 대상별 위력 식은 착탄 때 판정합니다.",
    [
      input("selector", "목표 선택", "enemy-selector"),
      input("delay", "방출 후 도착 지연", "seconds"),
      input("range", "착탄 피해 범위", "tile-pattern"),
      input("power", "타격 위력", "damage-power"),
    ],
    [plan()],
    [
      choice("selectionTime", "목표 위치 고정 시점", [
        unresolved,
        ["accepted", "접수 시점"],
        ["start", "이번 기회 시작"],
      ]),
      choice("damageType", "피해 유형", [
        unresolved,
        ["physical", "물리"],
        ["magic", "마법"],
      ]),
    ],
  ),
  define(
    "scheduled-plan",
    "기본 공격과 병행하는 유한 기회",
    "스킬 값",
    "접수 시 시전자 위치를 탐색 기준으로 고정하고 유한한 횟수의 기회를 전투 시간 간격으로 제공합니다. 각 기회는 새로 본문을 시작하며 본문의 전달 지연을 기다리지 않습니다. 적이 없어 본문이 실패해도 기회를 소모하고 이월·몰아주기·종료 연장을 하지 않습니다. 공격 속도는 횟수·간격을 바꾸지 않습니다. 기본 행동을 허용하고 같은 출처 재시전은 기존의 미시작 기회를 교체하되 이미 방출한 본문은 자기 수명을 유지합니다. 철수·사망에는 새 기회를 중단하고 기존 방출은 유지하며 전투 종료에는 모두 정리합니다. Pause에 동결합니다.",
    [
      input("count", "기회 횟수", "count-rule"),
      input("interval", "기회 간격", "seconds"),
      input("body", "기회마다 실행", "action-plan"),
    ],
    [plan()],
    [
      choice("first", "첫 기회", [
        unresolved,
        ["immediate", "시작 즉시"],
        ["after-interval", "한 간격 뒤"],
      ]),
    ],
  ),
];
