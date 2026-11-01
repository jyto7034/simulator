import type { Definition } from "../types";
import {
  define,
  input,
  output,
  number,
  integer,
  choice,
  unresolved,
  optionalInput,
} from "./skill-definition";
import { marks } from "./mark-catalog";

export const burstPortTypes = [
  ["explosion-rule", "폭발 피해·중복 규칙"],
  ["stack-grade", "스택별 폭발 등급"],
  ["burst-conversion", "총 스택에 따른 대체 폭발"],
] as const;
const plan = () => output("plan", "행동 계획", "action-plan");
const rule = () => input("explosion", "폭발 규칙", "explosion-rule");
const area = () => input("range", "각 폭발의 타일 범위", "tile-pattern");
export const burstCatalog: Definition[] = [
  define(
    "explosion-rule",
    "폭발 피해·중복 규칙",
    "스킬 값",
    "한 폭발은 적의 점유 칸 수와 무관하게 대상당 한 번입니다. 차례당 한 번은 같은 차례의 여러 폭발을 합쳐 대상당 한 번, 폭발별 한 번은 각 중심의 폭발을 별도로 판정합니다. 같은 차례 겹침은 피해 위력이 큰 후보를 우선하고 동률은 중심 타일·안정 ID 순입니다. 자기 숙주의 배율 후보는 기본 겹침 모드에서 우선합니다. 피해 증가율은 가하는 폭발 피해에만 적용하며 대상 약화나 무기 proc를 만들지 않습니다.",
    [],
    [output("explosion", "폭발 규칙", "explosion-rule")],
    [
      choice("dedup", "동일 대상 피해", [
        unresolved,
        ["wave", "차례당 한 번"],
        ["blast", "폭발별 한 번"],
      ]),
      choice("damageType", "피해 유형", [
        unresolved,
        ["physical", "물리"],
        ["magic", "마법"],
        ["true", "고정"],
      ]),
      number("bonus", "폭발 피해 증가 (%)"),
    ],
  ),
  define(
    "mark-burst-plan",
    "살아 있는 숙주의 일제 소비 폭발",
    "스킬 값",
    "접수의 최종 자격 검사에서 자기 출처의 살아 있는 숙주가 하나 이상인지 검사합니다. 없으면 공명·표식·위력·전투 revision 모두 불변입니다. 성공 시 숙주·중심 위치·공유 위력을 한 번 고정하고 표식과 위력을 전량 소비한 뒤 안정 ID 순으로 폭발합니다. E=고정 위력×폭발 계수. 자기 숙주는 E×숙주 배율 한 번이며 자기 기본 E를 더하지 않습니다. 차례당 한 번 모드는 자기 숙주 피해를 우선하고 다른 폭발로 추가 적중하지 않습니다. 폭발별 모드는 다른 숙주 폭발의 E도 받을 수 있습니다. 이 폭발은 최초 감염·사망 전파를 억제하며 다른 출처 표식은 소비하지 않습니다.",
    [
      marks(),
      input("power", "소비할 연속 위력", "population-power"),
      area(),
      rule(),
    ],
    [plan()],
    [
      number("coefficient", "폭발 위력 계수"),
      { ...number("hostMultiplier", "자기 숙주 피해 배율", 2), max: 4 },
    ],
  ),
  define(
    "path-burst-plan",
    "경로 전체의 지연 폭발",
    "스킬 값",
    "행동 방출 시 유효한 모든 walkable path 타일을 고정합니다. 한 방향이나 선택한 경로 하나로 줄이지 않습니다. 한 지연 뒤 각 중심의 타일 범위에서 현재 적을 조회하여 같은 차례의 폭발로 판정합니다. 각 폭발 범위를 넓혀도 중심 목록은 늘리지 않습니다. 방출 이후 독립 예약이며 시전자 이탈·사망으로 취소하지 않고 전투 종료에는 폐기합니다. 예약에 피해와 소유자를 고정합니다.",
    [
      input("delay", "방출 뒤 지연", "seconds"),
      input("power", "폭발 기준 위력", "damage-power"),
      area(),
      rule(),
    ],
    [plan()],
  ),
  define(
    "stack-grade",
    "기초 스택의 계단식 폭발",
    "스킬 값",
    "타일별 기초 스택 S에 대해 q=floor(S/나눌 수)입니다. q=0이면 S회 기본 위력, q≥1이면 q회 (기본 위력×(1+q×등급당 증가율))로 폭발합니다. S=0은 0회입니다. 등급 상한이나 몰래 버리는 초과 스택은 없습니다. 수치 오버플로·실행 한도 초과는 비용 확정 전에 거절합니다. 색은 q=0 하양, q=1 빨강, q≥2 파랑에 등급 숫자를 표시하는 연출 계약이며 피해 유형이 아닙니다.",
    [input("power", "등급 0 기준 위력", "damage-power")],
    [output("grade", "계단식 등급 계산", "stack-grade")],
    [
      integer("divisor", "등급을 나눌 수", 1),
      number("rankBonus", "등급당 위력 증가 (%)"),
    ],
  ),
  define(
    "stack-burst-plan",
    "타일·생존 표식 합산 폭발",
    "스킬 값",
    "접수 때 자기 저장소의 타일 스택과 생존 숙주의 중심 타일 스택을 합산·고정하고 참여한 스택을 전량 소비합니다. 사망 이전 스택을 다시 합산하거나 획득 배율을 재적용하지 않습니다. 양수 타일이 없으면 비용 확정 전에 거절합니다. 준비 지연 뒤 타일별 등급과 횟수로 폭발합니다. 회차 i의 모든 타일을 같은 폭발 차례로 묶고 다음 차례는 지정 간격 뒤입니다. 첫 방출 뒤에는 독립 예약으로 남습니다. 총 스택 대체 조건이 맞으면 일반 폭발 전체를 하나의 대체 폭발로 바꿉니다.",
    [
      marks(),
      input("grade", "타일별 등급 계산", "stack-grade"),
      input("delay", "첫 폭발 지연", "seconds"),
      input("interval", "폭발 차례 간격", "seconds"),
      area(),
      rule(),
      optionalInput("conversion", "총 스택 대체 조건", "burst-conversion"),
    ],
    [plan()],
  ),
  define(
    "burst-conversion",
    "총 스택 임계치의 대체 폭발",
    "스킬 값",
    "지정한 일반 유물을 장착하고 총 스택이 임계치 이상이면, 모든 스택을 소비해 일반 폭발 대신 지정 범위에 한 번 폭발합니다. 위력은 기준 위력+총 스택×스택당 추가 위력입니다. 연출 시간 동안 전투가 정지합니다.",
    [
      input("condition", "대체 폭발의 장착 조건", "equipment-match"),
      input("power", "기준 위력", "damage-power"),
      input("range", "전장 중심의 명시 범위", "tile-pattern"),
      input("presentation", "전투 정지 연출 시간", "seconds"),
    ],
    [output("conversion", "대체 폭발 조건", "burst-conversion")],
    [
      integer("threshold", "총 스택 임계치", 1),
      number("perStack", "스택당 추가 위력"),
    ],
  ),
];
