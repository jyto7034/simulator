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

export const targetPortTypes = [
  ["skill-target", "이번 조회 대상"],
  ["target-effect-profile", "대상 효과 설정"],
  ["target-state-profile", "대상 상태 정의"],
  ["target-state-instance", "이번 대상 상태"],
] as const;
const instance = () =>
  input("instance", "이번 대상 상태", "target-state-instance");

// Authoring contracts only: effects, time and battle entities belong to Core.
export const targetCatalog: Definition[] = [
  define(
    "range-targets",
    "범위 대상별 실행",
    "스킬 동작",
    "접수된 이번 사용의 시전자 또는 지정 타일에 배치 범위를 고정하고 살아 있는 유효 대상을 한 번 조회합니다. 대상마다 이번 사용·조회·대상 ID를 보존한 흐름을 동기적으로 실행합니다. 모든 대상 처리 뒤 완료는 빈 범위에서도 한 번 발생합니다. 대상별 흐름은 완료 흐름과 별개이며 시간 대기나 대상 간 값 전달을 포함하지 않습니다. 장판·진입 구독을 만들지 않습니다.",
    [
      incoming(),
      input("use", "이번 사용", "skill-use"),
      input("area", "배치 범위", "area-profile"),
      input("filter", "대상 조건", "ground-target-filter"),
    ],
    [
      output("each", "대상마다", "flow"),
      output("target", "이번 조회 대상", "skill-target"),
      output("out", "전체 처리 완료", "flow"),
    ],
  ),
  define(
    "target-state",
    "시간제 대상 상태",
    "스킬 상태",
    "여러 효과 설정을 하나의 양수 전투 시간 수명으로 묶습니다. 정의와 대상별 활성은 다릅니다. 적용 시 모든 효과의 자격·면역을 함께 검사하여 전부 적용하거나 거절합니다. 같은 효과 종류를 중복 연결하지 않습니다. 만료·대상 소멸·전투 종료·명시 해제에서 이번 활성의 효과만 정리합니다. 시전자 이탈 정책은 별도로 지정합니다. Pause에는 진행하지 않고 만료가 같은 시각 후속 처리보다 우선합니다. 상태 자체가 사용 잠금을 유지하지 않습니다.",
    [
      input("duration", "공통 지속 시간", "seconds"),
      {
        ...input("effects", "함께 적용할 효과", "target-effect-profile"),
        multiple: true,
      },
    ],
    [output("state", "상태 정의", "target-state-profile")],
    [
      choice("sourceExit", "시전자 사망·철수", [
        unresolved,
        ["end", "함께 종료"],
        ["independent", "남은 시간 유지"],
      ]),
    ],
  ),
  define(
    "target-taunt",
    "시전자 집결 도발",
    "스킬 값",
    "시전자를 적대적 기본 공격·공격 스킬의 주 대상으로 삼아 현재 위치로 정상 추적합니다. 다른 아군의 기존 저지를 해제하고 도발 중 새 저지는 시전자만 허용합니다. 시전자의 기존 저지 수·조건, 지형·경로·충돌은 유지하며 접근 불가 시 정상 대기합니다. 새 공격·순간이동·벽 통과를 만들지 않습니다. 도발 시작 때 미방출 대상 공격은 무환불 취소하고 이미 방출한 효과는 유지합니다. 도발 면역이면 상태 묶음을 거절합니다. 나중에 성공한 도발은 이전 도발 활성과 부가 효과를 종료하며 이전 상태를 복원하지 않습니다. 시전자 사망·철수에 함께 종료하는 수명이 필요합니다.",
    [],
    [output("effect", "도발 효과", "target-effect-profile")],
  ),
  define(
    "target-speed",
    "이동 속도 증가",
    "스킬 값",
    "대상 상태가 살아 있는 동안 원래 이동 속도 계산에 양수 백분율 증가 기여분을 적용합니다. 여러 대상 상태의 같은 종류 증가분은 가장 강한 값만 사용합니다. 종료는 이번 활성의 기여분만 제거하며 원래 속도를 저장해 덮어쓰지 않습니다. 이동 불가·지형·경로 규칙을 우회하지 않습니다.",
    [],
    [output("effect", "속도 증가 효과", "target-effect-profile")],
    [number("percent", "이동 속도 증가율 (%)")],
  ),
  define(
    "target-attack-limit",
    "공격 거리 상한",
    "스킬 값",
    "적대적 기본 공격·공격 스킬의 시작과 방출 때 대상과 주 공격 대상의 실제 평면 연속 위치 사이 거리를 타일 단위 상한으로 제한합니다. 대형·공중 대상도 같은 기준 위치를 쓰며 연출과 판정 타일로 거리를 대신하지 않습니다. 원래 사거리가 더 짧으면 그대로 제한합니다. 여러 상한은 가장 작은 값을 적용합니다. 미방출 공격이 조건을 잃으면 취소하며 이미 방출한 효과는 유지합니다. 새 근접 공격이나 추가 피해를 주지 않습니다.",
    [],
    [output("effect", "공격 거리 제한", "target-effect-profile")],
    [number("tiles", "공격 거리 상한 (타일)")],
  ),
  define(
    "target-apply",
    "대상 상태 적용",
    "스킬 동작",
    "같은 조회의 대상별 흐름에서 연결한 상태를 원자적으로 적용합니다. 대상·출처·면역을 재검사하고 거절 시 저지 관계와 다른 효과를 바꾸지 않습니다. 성공하면 이번 사용·대상·정의·적용 회차의 독립 활성을 만듭니다. 동일 출처·대상·정의의 재적용은 기존 활성을 종료하고 새 시간으로 교체합니다. 새 도발은 다른 출처의 이전 도발 활성도 대체합니다. 대상마다 수명을 분리하며 전체 조회 완료 뒤 사용이 끝나도 부여한 상태는 자기 수명을 따릅니다.",
    [
      incoming(),
      input("target", "이번 조회 대상", "skill-target"),
      input("state", "상태 정의", "target-state-profile"),
    ],
    [
      output("applied", "상태 적용 성공", "flow"),
      output("rejected", "상태 적용 거절", "flow"),
      output("instance", "이번 대상 상태", "target-state-instance"),
      cue("상태 적용"),
    ],
  ),
  define(
    "target-clear",
    "이번 대상 상태 해제",
    "스킬 동작",
    "같은 적용 성공 문맥 또는 해당 활성의 종료 사건에서 이번 대상 상태만 정리합니다. 다른 대상·시전자·재적용 회차는 건드리지 않습니다. 이미 종료했다면 무변경 완료이며 자연 완료 전용 효과나 환불을 만들지 않습니다. 자동 수명 정리는 이 노드를 연결하지 않아도 실행됩니다.",
    [incoming(), instance()],
    [next()],
  ),
  define(
    "target-ended",
    "대상 상태 종료 사건",
    "스킬 사건",
    "이번 대상 상태의 만료·명시 해제·대체·대상 소멸·설정한 시전자 이탈·전투 종료에서 정리 후 한 번 발생합니다. 이 활성의 도발·속도·거리 제한과 구독을 정리하고 남은 다른 출처의 규칙을 다시 계산합니다. 과거 위치·공격 대상·저지 관계를 복원하지 않습니다. 이 사건은 자연 완료와 동의어가 아니며 종료한 대상 문맥으로 상태를 다시 적용할 수 없습니다. 오래된 종료는 새 활성에 영향을 주지 않습니다.",
    [instance()],
    [next(), cue("상태 종료")],
  ),
];
