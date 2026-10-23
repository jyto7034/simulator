import type { Definition, Field, FieldValue } from "../types";
import {
  choice,
  cue,
  define,
  incoming,
  input,
  next,
  output,
  unresolved,
} from "./skill-definition";

export const spatialPortTypes = [
  ["tile-pattern", "타일 패턴"],
  ["target-selection", "대상 선택 설정"],
  ["skill-area-instance", "이번 장판"],
] as const;

export const spatialCatalog: Definition[] = [
  define(
    "range",
    "스킬 범위",
    "스킬 값",
    "포함할 타일 패턴만 정의합니다. ◎는 기준 타일이며 선택 범위와 효과 범위에 각각 연결할 수 있습니다. 실제 위치와 방향은 이 범위를 사용하는 설정에서 정합니다.",
    [],
    [output("range", "타일 패턴", "tile-pattern")],
    [
      {
        id: "pattern",
        label: "타일 패턴",
        valueType: "tile-pattern",
        default: "",
      },
      {
        id: "includeAnchor",
        label: "기준 타일 포함",
        valueType: "boolean",
        default: false,
      },
    ],
  ),
  define(
    "range-placement",
    "효과 범위 배치",
    "스킬 값",
    "타일 패턴을 실제 효과 위치에 놓는 설정입니다. 지정 타일은 접수한 타일을 유지하고 패턴을 회전하지 않습니다. 시전자 기준일 때만 위치와 바라보는 방향의 고정 시점을 각각 정합니다. 지정 타일 방향은 접수한 시전자 위치에서 지정 타일을 향하며 동일 타일이면 접수 방향을 사용합니다. 사용 동작의 요청·출처 문맥을 사용하며 대상을 새로 선택하지 않습니다.",
    [input("range", "효과 범위", "tile-pattern")],
    [output("area", "배치 범위", "area-profile")],
    [
      choice("origin", "범위 기준", [
        unresolved,
        ["caster", "시전자"],
        ["selected-tile", "접수한 지정 타일"],
        ["selected-direction", "접수한 위치에서 지정 타일 방향"],
      ]),
      choice("positionTime", "시전자 위치 고정", [
        unresolved,
        ["accepted", "입력 접수 시"],
        ["release", "효과 방출 시"],
      ]),
      choice("facingTime", "시전자 방향 고정", [
        unresolved,
        ["accepted", "입력 접수 시"],
        ["release", "효과 방출 시"],
      ]),
    ],
  ),
  define(
    "target-selection",
    "스킬 대상 선택",
    "스킬 값",
    "스킬 입력에서 무엇을 선택할지 정의합니다. 지정 타일은 시전자 타일·방향 기준의 선택 범위 안에서 고릅니다. 최종 접수 때 최신 자격·유효 타일·범위를 비용과 함께 검증하고 타일을 보존합니다. 선택 범위는 효과 범위가 아니며 빈 타일도 선택할 수 있습니다. 아군 지정은 범위 내 살아 있는 active player 직원만 허용하며 접수와 효과 방출 때 대상 ID·생존·자격을 다시 검사합니다. Guest·소환물·방어 오브젝트는 제외합니다.",
    [{ ...input("range", "대상 선택 범위", "tile-pattern"), required: false }],
    [output("selection", "대상 선택 설정", "target-selection")],
    [
      choice("mode", "대상 선택 방식", [
        unresolved,
        ["self", "자기 자신"],
        ["player-tile", "플레이어 지정 타일"],
        ["player-ally", "플레이어 지정 아군 직원"],
        ["player-direction", "플레이어 지정 방향"],
      ]),
    ],
  ),
  define(
    "ground-area",
    "지정 타일 장판 생성",
    "스킬 동작",
    "접수한 사용의 지정 타일에 배치 범위를 고정해 장판 하나를 생성합니다. 작성한 패턴을 회전하지 않으며 생성 후 위치·방향도 유지합니다. 생성 완료 뒤 다음 흐름으로 진행하고 이번 장판 참조를 제공합니다. 생성 자체는 피해·회복·상태 효과를 발생시키지 않습니다. 이번 장판에 진입·이탈·체류·주기·종료 사건을 연결합니다.",
    [
      incoming(),
      input("use", "이번 사용", "skill-use"),
      input("area", "배치 범위", "area-profile"),
      input("duration", "장판 유지 시간", "seconds"),
    ],
    [
      next(),
      output("area", "이번 장판", "skill-area-instance"),
      cue("장판 생성"),
    ],
  ),
];

export function spatialFieldActive(
  node: { kind: string; values: Record<string, FieldValue> },
  field: Field,
) {
  return (
    node.kind !== "skill-range-placement" ||
    !["positionTime", "facingTime"].includes(field.id) ||
    node.values.origin === "caster"
  );
}
