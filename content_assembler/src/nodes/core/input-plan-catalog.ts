import type { Definition } from "../types";
import {
  define,
  input,
  output,
  incoming,
  next,
  cue,
  choice,
  unresolved,
} from "./skill-definition";

export const inputPlanPortTypes = [
  ["layout-memory", "사본별 이전 배치"],
  ["input-preparation", "정지 중 입력 준비"],
  ["shot-layout", "종류·순서가 있는 배치"],
  ["shot-plan", "확정 전 전체 사격 계획"],
] as const;
const preparation = () =>
  input("preparation", "이번 입력 준비", "input-preparation");
const layout = () => input("layout", "이번 배치", "shot-layout");
const planOut = () => output("plan", "전체 사격 계획", "shot-plan");
export const inputPlanCatalog: Definition[] = [
  define(
    "layout-memory",
    "이전 성공 배치 기억",
    "스킬 상태",
    "같은 파편 사본의 두 재고와 연결한 마지막 성공 배치의 종류·수량·순서·타일만 전투 동안 기억합니다. 방향·피해·비용은 저장하지 않습니다. 미확정 입력·일반 취소·연결 단절은 갱신하지 않습니다. 전투 수명으로 소유자 재배치에도 유지하되 전투 종료에 비웁니다. 기억을 재사용해도 이번 조준을 새로 해야 합니다.",
    [
      input("normal", "일반 재고", "resource-bank"),
      input("enhanced", "강화 재고", "resource-bank"),
    ],
    [output("memory", "배치 기억", "layout-memory")],
  ),
  define(
    "plan-prepare",
    "전투 정지와 입력 준비 시작",
    "스킬 사건",
    "생존·배치·행동 가능한 파편 소유자의 수동 입력에서 시작합니다. 두 재고 합계가 0이면 시작하지 않습니다. 설정한 공명 조건을 사전 조회한 뒤 Core가 준비 ID·사본·전장 회차·기준 리비전·진입 시 배속을 결속하고 전투를 정지합니다. 배치·조준 동안 타이머·늑대 출현·피해·회복은 진행하지 않습니다. 아직 공명/재고를 소비하거나 실탄을 발사하지 않습니다. 일반 취소는 임시 입력을 폐기하고 이전 배속/정지를 복원합니다. 연결 단절은 임시 입력을 폐기하고 정지 상태를 유지합니다. 전투 종료·소유자 이탈은 준비를 정리하며 오래된 준비를 재사용하지 않습니다. 프로세스 종료 복원은 진입 전 savepoint 계약을 따릅니다.",
    [
      input("memory", "사본 배치 기억", "layout-memory"),
      input("range", "창 배치 가능 범위", "tile-pattern"),
      input("settings", "공명 설정", "resonance-settings"),
      input("requirement", "공명 사전 조건", "resonance-requirement"),
    ],
    [
      next(),
      output("preparation", "이번 입력 준비", "input-preparation"),
      output("request", "이번 최종 요청", "skill-request"),
      output("cancelled", "일반 취소", "flow"),
      output("disconnected", "확정 전 단절", "flow"),
      cue("준비 시작"),
    ],
  ),
  define(
    "plan-layout",
    "종류별 창 배치 입력",
    "스킬 동작",
    "이번 준비의 재고 수만큼 일반/강화 창을 순서 있는 목록으로 배치합니다. 좌클릭은 일반, 우클릭은 강화이며 각 항목에 고유 순번·종류·유효 타일을 보존합니다. 이전 위치 재사용을 선택하면 종류별 수량·순서·유효 타일·중복 정책을 모두 검사합니다. 하나라도 맞지 않으면 이유를 표시하고 전체 직접 배치로 전환하며 임의 부족분 보충/종류 교체를 하지 않습니다. 낙창과 두건 형상은 준비 연출이며 직원 배치·점유·저지를 이동시키지 않습니다.",
    [incoming(), preparation()],
    [
      next(),
      output("layout", "종류·순서·원점 목록", "shot-layout"),
      cue("배치 연출"),
    ],
    [
      choice("duplicates", "같은 타일 중복 배치", [
        unresolved,
        ["reject", "허용하지 않음"],
        ["allow", "허용"],
      ]),
    ],
  ),
  define(
    "plan-aim-mode",
    "사격 방식 선택 분기",
    "스킬 조건",
    "이번 준비 메뉴에서 선택한 정밀 사격/집중 사격 중 한 경로로만 진행합니다. 어느 방식도 이 단계에서 발사하지 않습니다. 준비 ID와 배치 목록을 유지하며 다른 준비의 결과를 합치지 않습니다.",
    [incoming(), layout()],
    [
      output("precise", "정밀 사격", "flow"),
      output("focused", "집중 사격", "flow"),
    ],
  ),
  define(
    "plan-precise-aim",
    "배치 순서대로 방향 저장",
    "스킬 동작",
    "정밀 사격 경로에서 배치 순서대로 각 원점의 직선 방향을 조준하고 클릭마다 방향만 저장합니다. 원점과 같은 위치의 영벡터·비정상 좌표는 거절하고 재입력을 요구합니다. 마지막 창의 방향까지 유효해야 전체 계획을 내며 아직 탄환·피해·비용을 만들지 않습니다. 두건 형상의 이동은 연출이고 실제 직원 위치는 유지합니다.",
    [incoming(), layout()],
    [next(), planOut(), cue("정밀 조준")],
  ),
  define(
    "plan-focused-aim",
    "한 타일로 전체 방향 계산",
    "스킬 동작",
    "집중 사격 경로에서 유효 목표 타일 하나를 선택하고 각 창 원점에서 그 타일 중심으로 향하는 방향을 계산합니다. 어느 원점과라도 같은 타일이면 영벡터이므로 목표를 다시 고르게 합니다. 방향을 다른 창에서 복사하거나 마지막 방향으로 자동 보정하지 않습니다. 모든 종류·순번·원점을 그대로 보존한 전체 계획만 내며 아직 발사하지 않습니다.",
    [incoming(), layout()],
    [next(), planOut(), cue("집중 조준")],
  ),
  define(
    "plan-merge",
    "선택된 사격 계획 합류",
    "스킬 동작",
    "같은 준비·배치의 정밀/집중 두 결과를 받되 실제로 도달한 한 분기의 계획만 전달합니다. 두 계획을 합쳐 발수를 늘리지 않고 실행하지 않은 분기의 값을 읽지 않습니다. 각 분기의 완료 흐름과 해당 데이터 출력이 짝이어야 합니다. 합류 전후 임의 효과 실행·다른 준비의 흐름 혼합은 허용하지 않습니다.",
    [
      incoming(),
      input("precise", "정밀 사격 계획", "shot-plan"),
      input("focused", "집중 사격 계획", "shot-plan"),
    ],
    [next(), planOut()],
  ),
  define(
    "plan-commit",
    "계획·비용·전체 방출 일괄 확정",
    "스킬 동작",
    "같은 준비의 최종 허가와 전체 계획·두 탄환 정의를 받아 모든 종류별 수량·타일·방향·소유자·리비전·공명 조건을 최신 상태로 재검증합니다. 창 전량/공명 소비, 모든 탄환의 같은 시각 방출, 배치 기억 갱신, 사용 동작 종료와 전투 재개를 하나의 원자 경계로 확정합니다. 하나라도 실패하면 부분 소비·부분 방출·기억 변경 없이 준비를 유지합니다. 성공 시 원래 진행 배속으로 재개하며 준비 전에 수동 정지였다면 정상 배속으로 재개합니다. 동일 준비/요청 ID의 재전송은 기존 확정 결과만 반환하고 소비/발사를 반복하지 않습니다. 확정 뒤 응답 유실은 결과를 되돌리지 않습니다. 동시 방출과 개별 도착·적중 시각을 구분합니다.",
    [
      incoming(),
      input("permit", "최종 사용 허가", "resonance-permit"),
      input("plan", "전체 사격 계획", "shot-plan"),
      input("normal", "일반탄 정의", "projectile-profile"),
      input("enhanced", "강화탄 정의", "projectile-profile"),
    ],
    [
      next(),
      output("rejected", "전체 확정 거절", "flow"),
      output("batch", "동시 방출한 묶음", "projectile-batch"),
      cue("전체 동시 방출"),
    ],
  ),
];
