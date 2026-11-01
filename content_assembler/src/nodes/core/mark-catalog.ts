import type { Definition, Field } from "../types";
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

export const markPortTypes = [
  ["target-marks", "출처별 대상·타일 표식"],
  ["mark-filter", "표식 대상 자격"],
  ["mark-hit-source", "표식의 무기 적중 구독"],
  ["population-power", "생존 표식 수에 따른 연속 위력"],
] as const;
export const marks = () => input("marks", "표식 저장소", "target-marks");
const flag = (id: string, label: string, value = false): Field => ({
  id,
  label,
  valueType: "boolean",
  default: value,
});
const damageType = () =>
  choice("damageType", "피해 유형", [
    unresolved,
    ["physical", "물리"],
    ["magic", "마법"],
    ["true", "고정"],
  ]);
export const markCatalog: Definition[] = [
  define(
    "target-marks",
    "출처별 대상·타일 표식",
    "스킬 상태",
    "소유 직원·장착 사본·이 선언을 키로 분리한 표식입니다. 존재형은 숙주당 하나, 스택형은 비음수 정수입니다. 사망 확정 때 제거하거나 보유량을 사망 중심 타일로 한 번 옮깁니다. 이전은 새 획득이 아니며 증폭하지 않습니다. 큰 유닛도 중심 타일 하나로 적립·합산합니다. 배치 수명은 철수·전투불능 때, 전투 수명은 전투 종료 때 폐기합니다. Retreat는 진입 전 상태로 복원합니다. 별도 만료 없이 수명 종료까지 유지하며 새 적중은 타이머를 만들지 않습니다.",
    [],
    [output("marks", "표식 정의", "target-marks")],
    [
      choice("mode", "저장 방식", [
        unresolved,
        ["presence", "숙주 존재"],
        ["stacks", "정수 스택"],
      ]),
      choice("lifetime", "표식 수명", [
        unresolved,
        ["deployment", "배치 동안"],
        ["battle", "이번 전투"],
      ]),
      choice("death", "사망 시 처리", [
        unresolved,
        ["remove", "숙주와 제거"],
        ["tile", "사망 타일로 이전"],
      ]),
    ],
  ),
  define(
    "mark-filter",
    "표식 대상 자격",
    "스킬 값",
    "살아 있는 적 중 각 항목에 해당하는 대상만 허용합니다. 보스·환상체·공중·소환·임시 유닛은 독립 조건이며 하나라도 제외이면 거절합니다. 임시는 보상·Wave authority가 없는 유닛입니다. 필터는 최초 부여와 전파에 동일하게 적용하며 피해 대상 필터로 암묵 재사용하지 않습니다.",
    [],
    [output("filter", "대상 자격", "mark-filter")],
    [
      flag("air", "공중 포함"),
      flag("boss", "보스 포함", true),
      flag("abnormality", "환상체 포함", true),
      flag("summon", "소환물 포함"),
      flag("temporary", "임시 유닛 포함"),
    ],
  ),
  define(
    "mark-weapon-hit",
    "무기 적중의 표식 부여",
    "스킬 상태",
    "설정한 장착 출처가 활성인 동안 직접 적대적 기본 공격의 유효 적중을 구독합니다. 행동 ID·대상 ID별 한 번만 판정하며 지속피해·반사·장비 proc·환경·스킬 폭발은 제외합니다. 출처·행동·대상의 결정론적 난수로 최초 존재 표식을 판정합니다. 이미 존재하면 새 확률을 굴리지 않습니다. 스택은 각 유효 적중의 기본 부여량에 획득 배율을 한 번 적용합니다. 존재형은 치명타격으로 새 숙주를 만들지 않고, 스택형 치명타격은 사망 확정 전에 부여하여 타일 이전에 포함합니다. 무기 승급 출처는 지정한 실제 무기 사본의 영구 승급을 검사하고 다른 파편 장착 중에도 유지합니다.",
    [
      marks(),
      input("filter", "부여 대상 자격", "mark-filter"),
      optionalInput("evolution", "무기 영구 승급 정의", "weapon-evolution"),
    ],
    [output("source", "적중 구독", "mark-hit-source")],
    [
      choice("binding", "장착 출처", [
        unresolved,
        ["fragment", "이 파편 장착"],
        ["weapon-base", "승급 전 무기 장착"],
        ["weapon-upgrade", "승급 무기 장착"],
      ]),
      {
        id: "weapon",
        label: "무기 정의 ID",
        valueType: "text",
        default: "",
      },
      { ...number("chance", "최초 부여 확률 (%)"), max: 100 },
      integer("amount", "적중당 스택", 1),
      integer("grantMultiplier", "스택 부여 배율", 1),
    ],
  ),
  define(
    "mark-spread",
    "피격·사망의 표식 전파",
    "스킬 상태",
    "같은 적중 구독 출처의 기존 숙주가 비치명 기본 공격에 맞거나, 누구에게든 죽어 사망 확정할 때 한 번 전파합니다. 치명 공격은 사망 전파만 수행합니다. 후보를 먼저 고정하고 거리 오름차순·경로 진행 내림차순·안정 유닛 ID 오름차순으로 상한까지 선택하여 확률 없이 부여합니다. 이 트랜잭션의 새 숙주는 다시 전파하지 않습니다. 일제 소비 폭발의 사망은 전파하지 않습니다.",
    [
      marks(),
      input("source", "같은 출처 적중 구독", "mark-hit-source"),
      input("filter", "전파 대상 자격", "mark-filter"),
    ],
    [],
    [
      number("radius", "전파 반경 (타일)"),
      integer("limit", "전파 대상 상한", 1),
    ],
  ),
  define(
    "population-power",
    "생존 숙주 수의 연속 위력",
    "스킬 상태",
    "같은 출처 존재형 표식의 살아 있는 숙주 수 N에 대해 dP/dt=N×초당 성장량을 적분하고 상한을 적용합니다. 피해 주기와 독립된 전투 시간입니다. N=0이면 유예 뒤 초당 감소량만큼 0까지 감소하며, 재감염 때 남은 값에서 성장을 재개합니다. 자연 감소 방지는 유예 뒤 감소만 막고 성장·소비·수명 종료를 바꾸지 않습니다. 배치 이탈·전투 종료에는 위력을 폐기합니다.",
    [marks(), input("grace", "숙주 소멸 유예", "seconds")],
    [output("power", "공유 연속 위력", "population-power")],
    [
      number("growth", "숙주당 초당 성장"),
      number("cap", "위력 상한"),
      number("decay", "초당 자연 감소"),
      flag("preventDecay", "자연 감소 방지"),
    ],
  ),
  define(
    "mark-periodic-damage",
    "숙주의 주기 피해",
    "스킬 상태",
    "각 숙주의 최초 감염 뒤 한 온전한 주기부터 현재 공유 위력×계수의 피해를 처리합니다. 새 감염·기존 숙주 재피격은 다른 숙주의 시계를 바꾸지 않습니다. 간격 변경은 피해 cadence만 바꾸며 위력 성장량을 바꾸지 않습니다. 적중·무기 proc를 생성하지 않고 실제 사망은 별도의 사망 전파 구독이 처리합니다.",
    [
      marks(),
      input("power", "같은 출처 연속 위력", "population-power"),
      input("interval", "피해 주기", "seconds"),
    ],
    [],
    [number("coefficient", "현재 위력 계수"), damageType()],
  ),
];
