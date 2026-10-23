import type { Definition } from "../types";
import {
  define,
  input,
  output,
  number,
  choice,
  unresolved,
  optionalInput,
  multipleInput,
} from "./skill-definition";
export const reactiveStatusPortTypes = [
  ["reactive-status-profile", "사건 부여 상태 정의"],
] as const;
export const reactiveStatusCatalog: Definition[] = [
  define(
    "reactive-status",
    "출처별 시간제 제어 상태",
    "스킬 상태",
    "대상·소유자·정의별 상태를 양수 전투 시간 유지합니다. 둔화는 가장 강한 유효 기여만 적용하고 빙결은 공통 Hard CC 자격·면역을 따릅니다. 갱신 모드는 같은 출처 재부여 때 새 시간으로 교체하고 보존 모드는 기존 시간을 연장하지 않습니다. 연결한 차단 상태가 같은 대상에 어느 출처로든 활성이라면 새 부여를 거절합니다. 독립 상태는 소유자 이탈 뒤 유지하며 대상 사망·소멸·전투 종료에 정리합니다. 종료 뒤 이전 상태를 자동 복원하지 않습니다.",
    [
      input("duration", "유지 시간", "seconds"),
      optionalInput(
        "blockedBy",
        "활성 중 신규 부여를 막는 상태",
        "reactive-status-profile",
      ),
    ],
    [output("status", "상태 정의", "reactive-status-profile")],
    [
      choice("effect", "제어 효과", [
        unresolved,
        ["slow", "이동 둔화"],
        ["freeze", "빙결"],
      ]),
      { ...number("percent", "이동 둔화 (%)"), max: 100 },
      choice("refresh", "같은 출처 재부여", [
        unresolved,
        ["refresh", "새 시간으로 갱신"],
        ["preserve", "기존 만료 유지"],
      ]),
    ],
  ),
  define(
    "owned-damage-status",
    "소유자 피해 뒤 상태 부여",
    "스킬 상태",
    "이 파편 소유자의 적대적 피해에서 실제 HP 손실이 양수이고 살아남은 적에게 상태를 부여합니다. 기본 공격·스킬·지속피해·이동 효과·유물 피해를 포함하되 다른 직원 소유 피해는 제외합니다. 보호막만 감소·0 피해·비용은 제외하고 상태 부여는 새 피해를 만들지 않습니다. 피해 전 상태 변환을 예약한 동일 타격에서는 변환 판정 뒤 부여하며 차단 상태를 재검사합니다. 장착·배치 중에만 구독하고 사망·철수·전투 종료에 구독을 정리합니다.",
    [input("status", "피해 뒤 부여할 상태", "reactive-status-profile")],
    [],
  ),
  define(
    "convert-status-plan",
    "타격 직전 상태를 고정해 변환",
    "스킬 값",
    "본문의 각 타격 직전에 대상의 원본 상태 존재와 출처들을 고정합니다. 해당 타격의 피해를 처리한 뒤 살아 있고 변환 가능하면 원본의 해당 정의 기여들을 소비하고 새 상태를 이 시전자 출처로 부여합니다. 새 상태 면역/거절 때는 원본을 소비하지 않습니다. 원본 출처 목록과 변환 출처를 결과에 함께 남깁니다. 이번 피해가 패시브로 새 부여한 원본은 같은 타격의 변환에 사용하지 않습니다. 새 상태가 이미 활성이라면 변환·갱신하지 않습니다. 이 본문 밖의 기본 공격·지속피해·후속 이동 효과에는 변환을 붙이지 않습니다.",
    [
      input("body", "변환 판정할 직접 타격", "action-plan"),
      input("from", "타격 전 필요한 상태", "reactive-status-profile"),
      input("to", "소비 후 부여할 상태", "reactive-status-profile"),
    ],
    [output("plan", "행동 계획", "action-plan")],
  ),
  define(
    "status-death-mobile",
    "자기 상태 대상 사망의 이동 효과",
    "스킬 상태",
    "명시한 유물 장착 중 자기 소유의 연결한 상태가 하나 이상 유효한 적이 사망하면 사망 위치에 이동 효과 하나를 생성합니다. 다른 아군의 처치도 인정하고 여러 상태가 동시에 있어도 사망 ID·소유 직원당 한 번입니다. 침식 처치는 제외합니다. 기본 액티브의 시전당 생성과는 별도 사건입니다. 연결한 이동 효과 또는 그 파생 피해의 계보가 처치한 적은 제외합니다. 기본 생성보다 사망 생성을 먼저 처리하고 동일 상한을 적용합니다. 미배치 소유자는 새 생성하지 않으며 전투 종료에 구독을 정리합니다.",
    [
      multipleInput("states", "자기 소유 상태", "reactive-status-profile"),
      input("profile", "사망 위치 생성 효과", "mobile-effect-profile"),
      input("limit", "동시 효과 상한", "count-rule"),
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
