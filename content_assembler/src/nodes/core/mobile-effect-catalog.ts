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
export const mobileEffectPortTypes = [
  ["mobile-effect-profile", "비저지 이동 효과 정의"],
  ["fusion-tier", "합체 수별 효과 항목"],
  ["fusion-family", "같은 타일의 합체 계열"],
] as const;
const plan = () => output("plan", "행동 계획", "action-plan");
export const mobileEffectCatalog: Definition[] = [
  define(
    "mobile-variant",
    "이동 효과의 위력·범위 변형",
    "스킬 값",
    "기본 효과의 감지·이동·지연·수명을 재사용하고 폭발 위력과 범위만 명시한 값으로 교체합니다. 기본을 변형하거나 별도 이동 효과를 하나 더 생성하지 않습니다. 다른 변형을 참조할 수 있으나 순환은 허용하지 않습니다.",
    [
      input("base", "재사용할 이동 효과", "mobile-effect-profile"),
      input("power", "교체할 위력", "damage-power"),
      input("range", "교체할 폭발 범위", "tile-pattern"),
    ],
    [output("profile", "변형한 이동 효과", "mobile-effect-profile")],
  ),
  define(
    "scaled-range",
    "타일 범위 배율",
    "스킬 값",
    "기준 타일 중심에 대해 원래 포함 셀의 연속 정사각형을 배율로 확대하고 중심점이 확대된 영역 안에 있는 정수 타일을 포함합니다. 경계는 포함하며 결과를 집합으로 중복 제거합니다. 재귀 확대가 아니라 항상 연결한 원본 패턴에서 한 번 계산합니다. 감지·시전 선택 범위를 암묵적으로 확대하지 않습니다.",
    [input("base", "원본 타일 패턴", "tile-pattern")],
    [output("range", "확대한 타일 패턴", "tile-pattern")],
    [integer("factor", "범위 길이 배율", 1)],
  ),
  define(
    "fusion-count-power",
    "보존한 합체 수 비례 위력",
    "스킬 값",
    "이동 효과가 방출할 때 보존한 실제 누적 단위 수를 사용해 ceil(기본 위력 × (1 + max(0, 단위 수 - 기준 수) × 단위당 증가율 / 100))로 계산합니다. 등급이 바뀌어도 누적 수를 3으로 잘라내지 않습니다. 일반 스택 자원이나 스택 획득 유물의 대상이 아닙니다.",
    [input("base", "기본 위력", "damage-power")],
    [output("power", "합체 수 비례 위력", "damage-power")],
    [
      integer("baseline", "증가 전 기준 수", 1),
      number("percent", "추가 단위당 증가율 (%)"),
    ],
  ),
  define(
    "mobile-effect",
    "비저지 이동·폭발 효과",
    "스킬 값",
    "HP·피격·저지·직접 명령이 없는 소유자 귀속 효과입니다. 대기 도약은 설치 위치의 감지 범위에서 적을 찾은 뒤 목표의 당시 타일을 고정해 이동 속도로 도착하고 도착 후 지연 뒤 폭발합니다. 대기 수명은 도약 전에만 적용합니다. 추격 지연은 현재 위치 기준 가장 가까운 적을 계속 추격하고 생성 시점부터 정해진 지연 뒤 현재 위치에서 폭발합니다. 목표 소실은 재탐색하며 후보가 없으면 대기는 유지, 추격은 제자리에서 원래 폭발 지연을 유지합니다. 동률은 유닛 ID 순입니다. 폭발은 당시 범위 적당 한 번이며 결과에 이 효과 계보를 남깁니다. 다른 효과의 폭발과는 별개입니다. 독립 전달은 소유자 이탈 뒤 유지하지만 전투 종료에는 정리합니다. 공격력은 생성 때 고정하고 합체는 더 최근 생성자의 값을 덮어쓰지 않습니다.",
    [
      input("detection", "감지 타일", "tile-pattern"),
      input("range", "폭발 타일", "tile-pattern"),
      input("power", "폭발 위력", "damage-power"),
      input("delay", "도착/생성 후 폭발 지연", "seconds"),
      optionalInput("lifetime", "최대 대기 수명", "seconds"),
    ],
    [output("profile", "이동 효과 정의", "mobile-effect-profile")],
    [
      choice("motion", "이동과 폭발 시점", [
        unresolved,
        ["wait-leap", "대기 → 감지 → 고정 타일 도약"],
        ["chase-delay", "추격 → 지연 후 현재 위치 폭발"],
      ]),
      choice("damageType", "폭발 피해 유형", [
        unresolved,
        ["physical", "물리"],
        ["magic", "마법"],
      ]),
      number("speed", "이동 속도 (타일/초)"),
    ],
  ),
  define(
    "fusion-tier",
    "합체 누적 수별 외형·공격",
    "스킬 값",
    "누적 단위 수가 이 항목의 최소 수 이상인 항목 중 가장 큰 최소 수의 정의를 선택합니다. 같은 계열에서 최소 수를 중복하지 않습니다. 등급 변경은 외형·폭발 정의만 바꾸며 실제 누적 수·최초 생성 출처·위치를 보존합니다.",
    [input("profile", "해당 수의 이동 효과", "mobile-effect-profile")],
    [output("tier", "합체 항목", "fusion-tier")],
    [integer("minimum", "최소 누적 단위", 1)],
  ),
  define(
    "fusion-family",
    "같은 타일 합체 계열",
    "스킬 상태",
    "소유 직원·파편 사본·이 선언·설치 타일이 같은 대기 효과의 단위 수를 합산해 하나로 유지합니다. 서로 다른 직원·사본은 합치지 않습니다. 이미 도약한 효과는 합체하지 않습니다. 사망 생성 등 같은 사건 묶음은 모든 생성·합체를 먼저 확정한 뒤 탐색합니다. 계열당 동시 타일 상한을 넘는 새 타일은 생성하지 않고 기존 타일 합체는 허용합니다. 누적 상한 초과분은 버리며 저장/환불하지 않습니다. 철수·사망 뒤 이미 생성된 독립 효과는 유지하고 전투 종료에 정리합니다.",
    [multipleInput("tiers", "누적 수별 항목", "fusion-tier")],
    [output("family", "합체 계열", "fusion-family")],
    [
      integer("maxUnits", "타일당 최대 누적 단위", 3),
      integer("maxTiles", "동시 설치 타일 상한", 1),
    ],
  ),
  define(
    "place-fusion-plan",
    "선택 타일과 인접 한 곳에 배치",
    "스킬 값",
    "접수한 유효 타일에 주 단위를, 동서남북의 유효 인접 타일 한 곳에 인접 단위를 생성합니다. 북→동→남→서의 안정 순서로 고르며 유효 지상 타일과 다른 전투 유닛의 점유를 검사합니다. 같은 계열 대기 효과는 점유 장애물이 아닙니다. 인접 공간이 없으면 주 타일에 두 수를 합산하며 이전 효과도 누적 합체합니다. 큰 개체를 만든 뒤 별도 작은 개체를 더 남기지 않습니다. 전체 배치 가능 여부·상한은 비용 확정 전에 검사하고 생성 묶음 뒤 탐색합니다.",
    [input("family", "합체할 계열", "fusion-family")],
    [plan()],
    [
      integer("primary", "선택 타일 단위 수", 1),
      integer("adjacent", "인접 타일 단위 수", 1),
    ],
  ),
  define(
    "spawn-mobile-plan",
    "마지막 유효 적중 위치에 효과 생성",
    "스킬 값",
    "같은 직접 본문 결과에 유효 적중한 적 중 루트 진행도·유닛 ID 순으로 고른 한 적의 적중 당시 위치에 이동 효과를 하나 생성합니다. 처치를 요구하지 않습니다. 시전당 한 번이며 해당 이동 효과와 그 파생 피해의 계보에서 다시 생성하지 않습니다. 상한을 넘으면 생성만 생략하고 앞선 피해를 취소하지 않습니다. 이 계획은 유효 적중 후속 문맥에서만 실행합니다.",
    [
      input("profile", "생성할 효과", "mobile-effect-profile"),
      input("limit", "동시 효과 상한", "count-rule"),
    ],
    [plan()],
  ),
  define(
    "fusion-death-seed",
    "합체 효과 적중의 사망 씨앗",
    "스킬 상태",
    "명시한 유물을 장착한 동안 연결한 계열의 폭발 적중에 독립 시간제 표식을 부여합니다. 같은 출처 재부여는 시간을 갱신하고 중첩하지 않습니다. 치명 타격에도 사망 확정 전에 표식을 부여합니다. 표식이 유효한 적이 사망 확정되면 처치자와 무관하게 사망 위치에 단위 1을 한 번 생성합니다. 이 생성에는 시전의 인접 배치를 붙이지 않습니다. 생성된 효과도 표식을 부여하며 동시 사망은 생성→합체→탐색 순서로 처리합니다. 사망 ID·출처별 중복은 제외하고 타 직원의 계열로 이전하지 않습니다. 전투 종료에 정리합니다.",
    [
      input("family", "적중·사망 생성 계열", "fusion-family"),
      input("duration", "적중 표식 시간", "seconds"),
    ],
    [],
    [
      {
        id: "relic",
        label: "필요 유물 정의 ID",
        valueType: "text",
        default: "",
      },
    ],
  ),
];
