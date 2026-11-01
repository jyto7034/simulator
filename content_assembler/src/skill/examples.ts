import {
  createSkill,
  id,
  makeBlock as b,
  type SkillDocument,
  type SkillRoutine,
} from "./model";
const routine = (
  name: string,
  event: SkillRoutine["event"],
  condition: string,
  blocks: SkillRoutine["blocks"],
): SkillRoutine => ({ id: id(), name, event, condition, blocks });
export function exampleSkill(example: "butterfly" | "hood"): SkillDocument {
  const doc = createSkill();
  if (example === "butterfly") {
    const unit = id(),
      tile = id();
    doc.name = "죽나장 · 타일에 남는 나비";
    doc.description =
      "개화형 설계 예제. 무기 피해로 쌓인 유닛 스택을 사망 타일로 이전하고, 사용 시 타일별로 집계합니다. 계수·집계 시점·소비 규칙은 미정입니다.";
    doc.states = [
      {
        id: unit,
        name: "유닛 나비 스택",
        scope: "unit",
        lifetime: "",
        initial: "0",
      },
      {
        id: tile,
        name: "타일 나비 스택",
        scope: "tile",
        lifetime: "",
        initial: "0",
      },
    ];
    doc.routines = [
      routine(
        "무기 피해 → 스택 축적",
        "damage",
        "죽나장 무기에서 발생한 해당 피해",
        [b("changeState", { state: unit, target: "피격 유닛", amount: "" })],
      ),
      routine("사망 → 타일로 이전", "death", "나비 스택을 보유한 유닛", [
        b("transferState", {
          state: unit,
          destination: tile,
          target: "사망 지점의 타일",
        }),
      ]),
      routine("사용 → 집계와 폭발", "use", "개화 파편의 사용 자격 충족", [
        b("query", {
          target: "나비 스택을 집계할 타일",
          condition: "",
          result: "대상 타일 목록",
        }),
        b("repeat", { count: "대상 타일 목록", interval: "" }, [
          b("aggregate", {
            expression: "타일 스택 + 타일 위 유닛들의 스택 합계",
            result: "기초스택",
          }),
          b("aggregate", { expression: "", result: "폭발 등급·횟수" }),
          b("damage", { target: "현재 타일의 적", amount: "", damageType: "" }),
        ]),
      ]),
    ];
    doc.routines[2].blocks[1].note =
      "타일별 처리 순서·집계 시점·스택 소비 여부는 미정. 폭발 횟수의 실제 반복은 후속 모델링 대상.";
  } else {
    const normal = id(),
      enhanced = id();
    doc.name = "두건 · 창 배치와 동시 사격";
    doc.description =
      "2026-10-19 기본형 설계 예제. 전투를 멈춰 배치·조준하고, 최종 확정 시 소비와 전체 발사·전투 재개를 함께 처리합니다. 개화형은 보류합니다.";
    doc.states = [
      {
        id: normal,
        name: "일반 창",
        scope: "source",
        lifetime: "",
        initial: "",
      },
      {
        id: enhanced,
        name: "강화 창",
        scope: "source",
        lifetime: "",
        initial: "",
      },
    ];
    doc.routines = [
      routine("시간 경과 → 일반 창", "timer", "전투 진행 중 · 획득 주기 미정", [
        b("changeState", { state: normal, target: "제공 사본", amount: "1" }),
      ]),
      routine(
        "늑대 처치 → 강화 창",
        "death",
        "별도 사냥감 늑대 · 처치자 인정 범위 미정",
        [
          b("changeState", {
            state: enhanced,
            target: "제공 사본",
            amount: "1",
          }),
        ],
      ),
      routine(
        "배치·조준 → 전체 발사",
        "use",
        "Core가 준비를 시작하며 전투 정지",
        [
          b("input", {
            request: "일반·강화 창의 타일 배치 또는 이전 배치 재사용",
            result: "창 배치 목록",
          }),
          b(
            "branch",
            { condition: "사격 방식이 정밀 사격" },
            [
              b("input", {
                request: "배치 순서대로 각 창의 방향 지정",
                result: "사격 계획",
              }),
            ],
            [
              b("input", {
                request: "목표 타일 하나 선택 → 창별 방향 계산",
                result: "사격 계획",
              }),
            ],
          ),
          b("commit", {
            cost: "일반·강화 창 및 공명",
            operation:
              "전체 사격 계획 검증 후 소비·모든 탄환 발사·전투 재개 확정",
          }),
        ],
      ),
      routine("강화탄 적중 → 피해·속박", "impact", "이번 파편의 강화탄", [
        b("damage", { target: "적중한 적", amount: "", damageType: "" }),
        b("control", { target: "적중한 적", control: "속박", duration: "2" }),
      ]),
    ];
    doc.routines[2].blocks[2].note =
      "동시 발사와 동시 적중은 다름. 확정 전 단절은 임시 입력을 폐기하고 정지 유지. 확정 후 응답 유실은 소비·발사를 되돌리지 않음. 일반탄·늑대 출현·상한은 후속 모델링.";
  }
  return doc;
}
