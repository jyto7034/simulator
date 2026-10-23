import type { Definition } from "../types";
import {
  choice,
  cue,
  define,
  incoming,
  input,
  next,
  number,
  output,
  unresolved,
} from "./skill-definition";
export const phasePortTypes = [
  ["channel-phase", "집중 단계 정의"],
  ["damage-result", "확정 피해 결과"],
  ["healing-result", "확정 회복 결과"],
  ["cast-outcomes", "시전별 결과 집계"],
  ["count-range-tier", "개수별 범위 항목"],
] as const;
const optional = (id: string, label: string, type: string) => ({
  ...input(id, label, type),
  required: false,
});
const multiple = (id: string, label: string, type: string) => ({
  ...input(id, label, type),
  multiple: true,
});
const channel = () => input("channel", "이번 집중", "skill-channel");
export const phaseCatalog: Definition[] = [
  define(
    "channel-phase",
    "유한 집중 단계 정의",
    "스킬 값",
    "양수 지속 시간·배치 범위와 다음 단계로 유한한 집중 순서를 정의합니다. 단계 시작에 종속 영역을 만들고 끝나면 정리한 뒤 다음 단계로 진행합니다. 마지막 단계가 끝나면 자연 완료입니다. 중단은 후속 단계를 시작하지 않습니다. 시간 대기·실제 효과는 단계별 영역의 사건/동작에 명시합니다. 정의를 여러 시전에 써도 실행 회차 상태는 공유하지 않습니다.",
    [
      input("duration", "단계 유지 시간", "seconds"),
      input("area", "단계 배치 범위", "area-profile"),
      optional("next", "다음 단계", "channel-phase"),
    ],
    [output("phase", "단계 정의", "channel-phase")],
  ),
  define(
    "sequence-channel",
    "단계 순서 집중 시작",
    "스킬 동작",
    "같은 접수 성공 사용에서 유한 단계 순서를 한 번 시작합니다. 기존 집중과 같은 사용/공명 잠금·전술 이동 제한·표준 Hard CC 취소·침묵 중 유지 규칙을 따릅니다. 진행 중 단계만 종속 영역을 갖습니다. 모든 단계가 자연 완료해야 완료 출력이 발생합니다. 다른 종료는 중단 출력으로 분리합니다. 먼저 영역·구독을 정리하고 사용 종료/자원 종료 처리를 확정합니다. 출력의 동기 완료 보상까지 마친 뒤 예약 자동 요청을 시작합니다. 완료/중단은 회차당 한 번이며 이미 적용한 피해·회복을 취소하지 않습니다.",
    [
      incoming(),
      input("use", "이번 사용", "skill-use"),
      input("first", "첫 단계", "channel-phase"),
    ],
    [
      next(),
      output("channel", "이번 집중", "skill-channel"),
      output("completed", "자연 완료", "flow"),
      output("cancelled", "중단", "flow"),
      cue("집중 시작"),
    ],
    [
      choice("basicAttack", "집중 중 기본 공격", [
        unresolved,
        ["pause", "중지"],
        ["allow", "유지"],
      ]),
      choice("interruption", "외부 중단", [
        unresolved,
        ["standard", "표준 · Hard CC 취소"],
        ["protected", "진행 중 집중 보호"],
      ]),
    ],
  ),
  define(
    "phase-area",
    "이번 단계의 종속 영역",
    "스킬 상태",
    "집중과 그 순서에 속한 단계 정의를 함께 참조합니다. 해당 단계가 시작할 때만 실제 영역이 생성되고 그 단계 종료/집중 중단에 정리됩니다. 아직 시작하지 않은 단계는 대상·주기 사건을 내지 않습니다. 장판 사건을 이 참조에 연결하며 다른 집중의 단계와 섞지 않습니다. 위치·방향은 이번 사용이 접수한 조준으로 고정합니다.",
    [channel(), input("phase", "단계 정의", "channel-phase")],
    [output("area", "이번 단계 영역", "skill-area-instance")],
  ),
  define(
    "cast-outcomes",
    "시전별 실제 결과 집계",
    "스킬 상태",
    "같은 집중에 속한 피해/회복 결과를 회차별로 관찰합니다. 실제 적 HP 손실이 양수인 적중과 실제 아군 HP 회복이 양수인 적중이 각각 있었는지 기록합니다. 두 결과는 서로 다른 대상/주기여도 되지만 같은 사용이어야 합니다. 요청량·보호막 손실·면역/거절·초과 회복은 성공으로 세지 않습니다. 중복 결과 ID는 한 번만 관찰하고 완료 시 집계를 고정합니다. 다른 시전·무기 효과의 결과는 포함하지 않습니다.",
    [
      channel(),
      multiple("damage", "관찰할 피해 결과", "damage-result"),
      multiple("healing", "관찰할 회복 결과", "healing-result"),
    ],
    [output("summary", "이번 시전 결과", "cast-outcomes")],
  ),
  define(
    "completion-award",
    "자연 완료 보상 계산",
    "스킬 동작",
    "같은 집중의 자연 완료 흐름에서만 기본 보상과 실제 적 피해/아군 회복 모두 성공 시 추가 보상을 합산합니다. 대상 수·판정 주기와 무관하게 사용 회차당 한 번 계산하고 뒤의 자원 획득에서 한 번 확정합니다. 중단/실패에는 보상하지 않습니다. 단순 처리 완료나 다른 사용의 집계를 완주로 인정하지 않습니다.",
    [incoming(), input("summary", "이번 시전 결과", "cast-outcomes")],
    [next(), output("amount", "총 보상 개수", "number")],
    [
      { ...number("base", "완주 기본 개수"), integer: true },
      { ...number("bonus", "피해·회복 성공 추가 개수"), integer: true },
    ],
  ),
  define(
    "count-healing",
    "소비 개수 비례 회복",
    "스킬 값",
    "같은 사용에서 확정 소비한 자원 개수로 회복 요청량을 계산합니다. ceil(기본 요청량 × (1 + 소비 개수 × 개당 증가율 / 100))이며 대상별 실제 회복 결과가 아닙니다. 0개는 기본 성능입니다. 이후 같은 출처 보정을 중복 적용하지 않으며 숫자 손실/범위 초과는 컴파일 진단입니다.",
    [
      input("base", "기본 회복 요청량", "healing-power"),
      input("count", "확정 소비 개수", "number"),
    ],
    [output("power", "회복 요청량", "healing-power")],
    [number("percent", "개당 증가율 (%)")],
  ),
  define(
    "count-damage",
    "소비 개수 비례 피해",
    "스킬 값",
    "같은 사용의 확정 소비 개수를 사용해 ceil(기본 위력 × (1 + 소비 개수 × 개당 증가율 / 100))을 계산합니다. 0개는 기본 위력입니다. 실제 HP 피해는 피해 파이프라인 결과에서 읽으며 여기서 추정하지 않습니다.",
    [
      input("base", "기본 피해 위력", "damage-power"),
      input("count", "확정 소비 개수", "number"),
    ],
    [output("power", "피해 위력", "damage-power")],
    [number("percent", "개당 증가율 (%)")],
  ),
  define(
    "count-range-tier",
    "개수별 범위 항목",
    "스킬 값",
    "정확한 소비 개수 하나에 대응하는 명시적 타일 패턴입니다. 같은 선택기에 중복 개수를 넣지 않습니다. 범위 확대 선택기에서는 높은 개수의 패턴이 낮은 개수의 패턴을 포함해야 합니다. 미정 패턴을 임의 원/사각형으로 바꾸지 않습니다.",
    [input("range", "효과 타일 패턴", "tile-pattern")],
    [output("tier", "개수별 범위", "count-range-tier")],
    [{ ...number("count", "소비 개수"), integer: true }],
  ),
  define(
    "count-range",
    "소비 개수별 범위 선택",
    "스킬 값",
    "같은 사용의 실제 소비 개수와 일치하는 항목을 선택합니다. 접수 조건에서 가능한 모든 개수를 명시해야 하며 빠진 항목·중복·축소되는 패턴은 진단합니다. 하나의 선택 범위를 효과 범위로 자동 확대하지 않습니다.",
    [
      input("count", "확정 소비 개수", "number"),
      multiple("tiers", "개수별 범위 항목", "count-range-tier"),
    ],
    [output("range", "선택한 타일 패턴", "tile-pattern")],
  ),
  define(
    "corrupted-filter",
    "침식체 적 조건",
    "스킬 값",
    "영역의 출처와 적대하는 살아 있는 침식체만 선택합니다. 아군·환상체는 제외합니다. 지상/공중 여부만으로 분류를 바꾸지 않으며 실제 강제 이동 자격은 이동 동작에서 다시 판정합니다.",
    [],
    [output("filter", "침식체 적 조건", "ground-target-filter")],
  ),
  define(
    "ground-pull",
    "대상 단계별 흡인",
    "스킬 동작",
    "같은 영역 주기의 침식체 적을 현재 실제 위치에서 고정 영역 중심 쪽으로 지정 거리만큼 당깁니다. 중심을 넘지 않고 기존 강제 이동 면역·지형·충돌을 따르며 장애물 앞에서 멈춥니다. 순간이동·벽 통과·아군/환상체 이동을 만들지 않습니다. 이후 대상 조회는 이동한 실제 위치를 사용합니다. 거절 시 위치를 바꾸지 않으며 처리 완료는 이동 성공이 아닙니다.",
    [incoming(), input("member", "이번 대상 관계", "ground-member")],
    [next(), cue("흡인 처리")],
    [number("tiles", "판정당 흡인 거리 (타일)")],
  ),
];
