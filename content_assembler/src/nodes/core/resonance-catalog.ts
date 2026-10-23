import type { Definition, Field, GraphNode } from "../types";
import {
  choice,
  define,
  incoming,
  input,
  next,
  number,
  output,
  state,
  unresolved,
} from "./skill-definition";

export const resonancePortTypes = [
  ["resonance-amount", "공명량"],
  ["resonance-settings", "공명 설정"],
  ["resonance-snapshot", "공명 조회 결과"],
  ["resonance-requirement", "공명 요구 조건"],
  ["resonance-cost", "공명 비용 명세"],
  ["resonance-permit", "공명 사용 허가"],
  ["skill-use", "이번 사용"],
  ["resonance-step", "이번 지불 단계"],
  ["resonance-receipt", "공명 지불 결과"],
] as const;

const optional = (id: string, label: string, type: string) => ({
  ...input(id, label, type),
  required: false,
});
const amount = (id: string, label: string) =>
  optional(id, label, "resonance-amount");
const usagePort = () => input("use", "이번 사용", "skill-use");
const request = () => input("request", "사용 요청", "skill-request");
const receipt = () => output("receipt", "지불 결과", "resonance-receipt");
const rejected = () => output("rejected", "거절", "flow");

export const resonanceCatalog: Definition[] = [
  define(
    "resonance-amount",
    "공명량",
    "스킬 값",
    "공명 단위의 0 이상 정수입니다. 소수는 입력할 수 없습니다. 저장 상한·최소 요구량·고정 비용에 연결합니다. 빈 값은 미정이며 0으로 대체하지 않습니다.",
    [],
    [output("value", "공명량", "resonance-amount")],
    [{ ...number("value", "공명량"), integer: true }],
  ),
  define(
    "resonance-settings",
    "공명 저장 설정",
    "스킬 값",
    "이 스킬 출처의 최대 공명을 한 곳에서 정의합니다. 입력 요청의 직원·장착 스킬 사본에 적용됩니다. 배치 초기량·충전·획득 잠금은 공통 규칙을 따릅니다.",
    [input("capacity", "저장 상한", "resonance-amount")],
    [output("settings", "공명 설정", "resonance-settings")],
  ),
  define(
    "resonance-read",
    "공명 조회",
    "스킬 값",
    "요청의 직원·장착 스킬과 공명 설정에 결합된 조회입니다. 현재량·최대량은 관측 시점의 읽기 전용 결과이며 지불 허가가 아닙니다. 접수·후속 지불은 같은 참조와 계산 규칙을 지불 직전 최신 상태로 다시 평가합니다.",
    [request(), input("settings", "공명 설정", "resonance-settings")],
    [
      output("snapshot", "조회 결과", "resonance-snapshot"),
      output("current", "관측 현재량", "resonance-amount"),
      output("maximum", "관측 최대량", "resonance-amount"),
    ],
  ),
  define(
    "resonance-input",
    "요청의 투입량·횟수",
    "스킬 값",
    "플레이어가 이 요청에서 선택한 투입 공명과 횟수를 읽습니다. 없는 선택값은 0으로 대체하지 않으며 접수 시 선택 범위와 투입 공명·횟수의 정수 여부를 검증합니다. 선택량 방식은 이 요청의 투입량 출력을 사용합니다.",
    [request()],
    [
      output("amount", "선택 투입량", "resonance-amount"),
      output("count", "선택 횟수", "number"),
    ],
  ),
  define(
    "resonance-requirement",
    "공명 요구 조건",
    "스킬 값",
    "만충·최소량 이상·이번 비용 지불 가능·공명 조건 없음을 선택합니다. 공명 조건 없음도 행동 자격과 비용 지불을 면제하지 않습니다. 최소량 방식에서만 최소 공명 입력이 필요합니다.",
    [amount("minimum", "최소 공명")],
    [output("requirement", "요구 조건", "resonance-requirement")],
    [
      choice("mode", "공명 조건", [
        unresolved,
        ["full", "만충"],
        ["minimum", "최소량 이상"],
        ["payable", "이번 비용 지불 가능"],
        ["none", "공명 조건 없음"],
      ]),
    ],
  ),
  define(
    "resonance-cost",
    "공명 비용 계산",
    "스킬 값",
    "소비 방식을 정의하는 읽기 전용 명세입니다. 고정량/횟수 비례는 단위 비용, 선택량은 요청 투입량과 최소·최대 입력을 사용합니다. 공명량과 비율(0~100%)은 정수만 사용합니다. 비율 비용은 항상 올림한 뒤 최소 비용을 적용합니다. 지불 시 최신 기준으로 재계산하고 잔량이 부족하면 거절합니다. 여기서는 공명을 차감하지 않습니다.",
    [
      input("snapshot", "공명 조회", "resonance-snapshot"),
      amount("amount", "단위 비용"),
      amount("selected", "요청 투입량"),
      amount("minimum", "선택 최소량"),
      amount("maximum", "선택 최대량"),
      optional("count", "소비 횟수", "number"),
    ],
    [output("cost", "비용 명세", "resonance-cost")],
    [
      choice("mode", "소비 방식", [
        unresolved,
        ["fixed", "고정량"],
        ["all", "현재량 전부"],
        ["max-percent", "최대량의 비율"],
        ["current-percent", "현재량의 비율"],
        ["selected", "선택한 양"],
        ["count", "횟수 비례"],
        ["zero", "공명 소비 없음"],
      ]),
      { ...number("percent", "소비 비율 (%)"), max: 100, integer: true },
      { ...number("minimumCost", "비율 최소 비용"), integer: true },
    ],
  ),
  define(
    "resonance-qualify",
    "공명과 첫 사용 자격",
    "스킬 조건",
    "요청 종류의 공통 수동/자동 사용 자격·추가 자원 조건·대상·공명 요구 조건·비용 지불 가능 여부를 검사합니다. 통과한 분기에만 요청·상태·공명 설정·조건·비용에 결합된 허가가 있습니다. 비활성 결과를 연결하면 그 상태의 부재도 검사합니다. 행동 시작 설정을 연결하면 상태 분기·필수 대상·첫 생성 비용 등 그 프로그램의 시작 조건을 함께 검사하고 허가에 고정합니다. 최종 사격 계획은 같은 준비 요청의 전체 배치·방향·재고를 검사하며 일반 접수 대신 계획 확정 노드만 소비합니다. 검사만 수행하며 접수와 지불은 아직 하지 않습니다.",
    [
      incoming(),
      request(),
      optional("vacancy", "비활성 결과", "skill-vacancy"),
      optional("terms", "추가 자원 조건", "counter-terms"),
      optional("program", "행동 시작 설정", "skill-program"),
      optional("plan", "최종 사격 계획", "shot-plan"),
      input("snapshot", "공명 조회", "resonance-snapshot"),
      input("requirement", "공명 조건", "resonance-requirement"),
      input("cost", "공명 비용", "resonance-cost"),
    ],
    [next(), rejected(), output("permit", "사용 허가", "resonance-permit")],
  ),
  define(
    "resonance-admit",
    "입력 접수와 공명 지불",
    "스킬 동작",
    "허가의 요청·소유자·출처·대상·비활성·행동 자격·조건·비용을 최신 상태로 재검증합니다. 요청 ID마다 한 번만 접수·공명과 추가 자원 차감·조준·사용 자격과 사용 잠금을 함께 확정합니다. 추가 자원 소비 결과는 해당 조건을 연결한 성공 경로에만 있습니다. 실패는 모두 무변경이며 성공 분기에만 이번 사용과 지불 결과가 있습니다. 효과는 이후 별도 적용합니다. 접수 뒤 취소에는 환불하지 않습니다.",
    [incoming(), input("permit", "사용 허가", "resonance-permit")],
    [
      next(),
      rejected(),
      output("use", "이번 사용", "skill-use"),
      receipt(),
      output("counterReceipt", "확정 자원 소비", "counter-receipt"),
    ],
  ),
  define(
    "resonance-install",
    "접수한 사용의 상태 적용",
    "스킬 동작",
    "접수한 사용과 같은 상태 선언에 새 활성·타이머·공격 교체를 한 번 적용합니다. 추가 공명 차감은 없습니다. 활성에 마무리 권리를 둡니다. 즉시 추가 공격하거나 공격 주기를 초기화하지 않습니다. 전투 시간으로 유지하며 시간제 효과가 끝날 때까지 해당 사용 잠금을 유지합니다. 접수 뒤 취소는 환불하지 않고 미방출 동작을 정리합니다.",
    [
      incoming(),
      usagePort(),
      state(),
      input("duration", "유지 시간", "seconds"),
      input("profile", "공격 설정", "attack-profile"),
    ],
    [
      output("activation", "생성한 활성", "skill-activation"),
      output("cue", "변형 시작", "presentation-event"),
    ],
  ),
  define(
    "resonance-stage",
    "사용의 후속 단계",
    "스킬 동작",
    "이 노드 ID로 구분하는 유한한 후속 단계입니다. 같은 사용의 성공 흐름에서만 단계 권리를 만듭니다. 단계당 한 번만 지불할 수 있으며 종료된 사용에는 권리가 없습니다. 이후 지불 성공 뒤 해당 효과를 연결합니다.",
    [incoming(), usagePort()],
    [next(), output("step", "지불 단계", "resonance-step")],
  ),
  define(
    "resonance-periodic",
    "사용 중 주기 비용 사건",
    "스킬 사건",
    "접수 성공한 이번 사용에 구독합니다. 첫 주기 뒤부터 전투 시간 간격마다 고유한 회차 권리를 만듭니다. 지정 회수 또는 사용 종료까지 유지합니다. 정지 중 진행·따라잡기 없음. 사용 종료·취소 시 구독을 정리하고 새 회차를 만들지 않습니다. 중도 비용 부족의 처리는 후속 지불 정책에 따릅니다. 완료 출력은 지정 회수의 정상 완료에서만 발생합니다.",
    [usagePort(), input("interval", "지불 주기", "seconds")],
    [
      output("out", "이번 주기", "flow"),
      output("step", "지불 단계", "resonance-step"),
      output("completed", "주기 완료", "flow"),
    ],
    [
      choice("limit", "주기 종료 기준", [
        unresolved,
        ["count", "지정 회수"],
        ["use-end", "사용 종료까지"],
      ]),
      number("count", "최대 지불 회수", 1),
    ],
  ),
  define(
    "resonance-pay",
    "후속 공명 지불",
    "스킬 동작",
    "같은 사용의 단계/회차 권리와 비용 명세를 최신 상태로 재검증해 한 번만 지불합니다. 새 사용·활성·재입력 권리를 생성하거나 수명을 연장하지 않습니다. 성공 흐름에만 실제 지불 결과를 제공합니다. 비용 부족 시 해당 비용·효과를 적용하지 않고 선택한 중단 정책을 따릅니다. 이미 낸 비용은 환불하지 않습니다.",
    [
      incoming(),
      input("step", "지불 단계", "resonance-step"),
      input("cost", "공명 비용", "resonance-cost"),
    ],
    [next(), rejected(), receipt()],
    [
      choice("insufficient", "비용 부족 처리", [
        unresolved,
        ["stop-use", "이번 사용 종료"],
        ["skip-step", "이번 단계 생략"],
      ]),
    ],
  ),
  define(
    "resonance-result",
    "공명 지불 결과 조회",
    "스킬 동작",
    "지불 성공 흐름에서 확정한 실제 소비량과 지불 직후 잔량을 읽습니다. 이후 공명 재조회나 UI 예상 비용으로 대신하지 않습니다. 이 결과는 같은 요청·사용·단계의 효과에만 사용합니다.",
    [incoming(), input("receipt", "지불 결과", "resonance-receipt")],
    [
      next(),
      output("paid", "실제 소비량", "resonance-amount"),
      output("remaining", "지불 직후 잔량", "resonance-amount"),
    ],
  ),
  define(
    "resonance-end",
    "이번 사용 종료",
    "스킬 동작",
    "해당 사용의 동작·유지 의무를 마친 뒤 명시적으로 종료합니다. 그 사용의 후속 지불 권리·주기 구독을 정리하며 다른 출처의 잠금을 해제하지 않습니다. 이미 지불한 비용은 환불하지 않습니다. 독립 생성 효과는 각자 수명을 따릅니다.",
    [incoming(), usagePort()],
    [next()],
  ),
];

// One applicability rule is shared by inspector visibility and semantic validation.
export function resonanceFieldActive(
  node: Pick<GraphNode, "kind" | "values">,
  field: Pick<Field, "id">,
) {
  if (node.kind === "skill-resonance-periodic" && field.id === "count")
    return node.values.limit === "count";
  return (
    node.kind !== "skill-resonance-cost" ||
    field.id === "mode" ||
    ["max-percent", "current-percent"].includes(String(node.values.mode))
  );
}

export function resonanceRequiredPorts(node: GraphNode): string[] {
  if (node.kind === "skill-resonance-requirement")
    return node.values.mode === "minimum" ? ["minimum"] : [];
  if (node.kind !== "skill-resonance-cost") return [];
  switch (node.values.mode) {
    case "fixed":
      return ["amount"];
    case "count":
      return ["amount", "count"];
    case "selected":
      return ["selected", "minimum", "maximum"];
    default:
      return [];
  }
}

export function resonanceInputActive(node: GraphNode, port: string) {
  if (node.kind === "skill-resonance-cost" && port !== "snapshot")
    return resonanceRequiredPorts(node).includes(port);
  if (node.kind === "skill-resonance-requirement")
    return resonanceRequiredPorts(node).includes(port);
  return true;
}
