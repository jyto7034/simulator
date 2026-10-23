import type { Definition } from "../types";
import {
  define,
  input,
  output,
  incoming,
  next,
  cue,
  integer,
} from "./skill-definition";
import { bankInput, grantOutput } from "./inventory-catalog";

export const portalPortTypes = [
  ["portal-policy", "시간 제한 준비물 설정"],
  ["portal-charge", "이번 준비물 전개"],
  ["portal-release-right", "이번 전개의 사출 권리"],
  ["frozen-volley", "확정한 유한 사출량"],
] as const;
export const portalCatalog: Definition[] = [
  define(
    "damage-mark",
    "출처별 비중첩 회수 상태",
    "스킬 상태",
    "피해를 입힌 출처별로 대상에게 하나만 남는 양수 전투 시간 상태입니다. 재부여는 남은 시간을 새 지속 시간으로 갱신하며 스택·피해·약화를 만들지 않습니다. 누가 부여하는지와 사망 보상은 연결한 회수 구독이 정합니다. 만료 시각부터 없는 상태이며 대상 사망·전투 종료에 정리합니다.",
    [input("duration", "회수 상태 지속 시간", "seconds")],
    [output("mark", "회수 상태 정의", "damage-mark")],
  ),
  define(
    "damage-recovery",
    "귀속 피해와 상태 보유 적 사망",
    "스킬 사건",
    "장착 파편 사본의 현재 소유자에게 귀속되는 모든 양수 실제 HP 피해에 회수 상태를 부여/갱신합니다. 기본 공격·스킬·장비 파생·지속 피해를 포함하고 피해 0·보호막만 손상·무적 거절은 제외합니다. 첫 피해가 치명적이어도 상태 부여를 사망 정산 전에 인정합니다. 자기 상태가 남은 적이 사망하면 막타 주인과 무관하게 저장소의 획득 권리를 한 번 만듭니다. 같은 적·사본의 한 사망은 하나의 기본 권리이며 여러 적중/범위/보구로 중복하지 않습니다. 다른 사본은 자기 상태와 권리를 따로 갖습니다. 공명 잠금은 이 패시브를 막지 않습니다. 회수 정산은 다음 표적 선택 전에 끝납니다.",
    [input("mark", "부여할 회수 상태", "damage-mark"), bankInput()],
    [next(), grantOutput()],
  ),
  define(
    "portal-policy",
    "가속 전개·유예·붕괴 설정",
    "스킬 값",
    "첫 간격 뒤부터 준비물을 하나씩 만듭니다. i번째 생성 간격은 첫 간격에서 마지막 간격까지 최대 전개 수에 따라 선형으로 줄이며 밀리초 올림을 사용합니다. 최대 전개 수는 재고/대여 한도와 별개입니다. 최대 수에 도달하면 새 전개를 멈추고 유예합니다. 첫 붕괴 시각은 전개 시작+전체 기한과 최대 도달+유예 중 빠른 쪽입니다. 자원 부족으로 생성이 멈춰도 기한은 흐르며 부족 시간은 새 생성으로 몰아 보충하지 않습니다. 붕괴 시작 후에는 새로 만들지 않고 오래된 준비물부터 붕괴 간격마다 하나씩 버립니다. 같은 시각 붕괴/기한이 입력보다 우선합니다.",
    [
      input("first", "첫 생성 간격", "seconds"),
      input("last", "마지막 생성 간격", "seconds"),
      input("hold", "최대 전개 후 유예", "seconds"),
      input("deadline", "전개 시작부터 전체 기한", "seconds"),
      input("decay", "준비물 붕괴 간격", "seconds"),
    ],
    [output("policy", "전개 설정", "portal-policy")],
    [integer("maximum", "한 번의 최대 전개 수", 1)],
  ),
  define(
    "portal-program",
    "비용을 내는 준비물 전개",
    "스킬 값",
    "전개 설정과 사본 원장을 연결합니다. 준비물 하나가 실제로 생성될 때만 원장 1을 원자적으로 지불하고 하나의 발사 권리를 만듭니다. 부족하면 부분 생성하지 않고 전개 기한 동안 기다립니다. 시전 시작에 전량 소비하거나 발사/명중 때 재청구하지 않습니다. 붕괴·취소로 버린 준비물의 비용·대출 이력은 환불하지 않습니다. 접수 때 첫 생성 비용의 지불 가능성을 검사하며 별도 자기 HP 비용은 없습니다.",
    [bankInput(), input("policy", "전개 설정", "portal-policy")],
    [output("program", "전개 행동 설정", "skill-program")],
  ),
  define(
    "run-portals",
    "접수한 준비물 집중 시작",
    "스킬 동작",
    "같은 사용 허가의 전개 설정을 시작하고 차징·연속 사출 전체 동안 하나의 공명 잠금을 유지합니다. 이동·새 기본 공격은 하지 않습니다. 첫 공명 소비 이후 준비물마다 새 시전을 만들지 않습니다. Hard CC는 미방출 준비물/후속 발사를 무환불 취소하고 침묵은 새 재입력만 막습니다. 사망·철수·직접 종료·전투 종료도 정리합니다. 이미 방출한 투사체는 별도 수명을 따릅니다. 전개 중 준비물을 모두 잃으면 종료합니다. 새 자원을 얻어도 종료한 집중을 자동 재개하지 않습니다.",
    [
      incoming(),
      input("use", "이번 사용", "skill-use"),
      input("program", "전개 행동 설정", "skill-program"),
    ],
    [
      next(),
      output("charge", "이번 전개", "portal-charge"),
      output("cancelled", "집중 중단", "flow"),
      output("expired", "준비물 소진 종료", "flow"),
      cue("전개 시작"),
    ],
  ),
  define(
    "portal-release-request",
    "전개 중 사출 재입력",
    "스킬 사건",
    "같은 직원·사본·이번 전개의 활성 권리를 가진 수동 재입력입니다. 공격 태세·행동 가능·침묵/Hard CC 없음과 남은 준비물 1개 이상을 검사합니다. 자신의 사용 잠금만 예외이며 추가 공명 만충/소비를 요구하지 않습니다. 전개 만료/동시 붕괴 뒤 유효 수량을 사용하며 클라이언트 과거 시각으로 소급하지 않습니다. 거절은 전개/비용/권리를 바꾸지 않습니다.",
    [input("charge", "이번 전개", "portal-charge")],
    [next(), output("right", "이번 사출 권리", "portal-release-right")],
  ),
  define(
    "portal-freeze",
    "남은 준비물 사출량 확정",
    "스킬 동작",
    "같은 재입력 성공 권리를 한 번 소비하여 추가 생성과 남은 준비물의 붕괴를 멈추고 유한한 사출 목록을 고정합니다. 이후 처치로 얻은 재고를 이 목록에 끼워 넣지 않습니다. 사출 차례 대기만으로 준비물이 추가 소실되지 않습니다. 중단하면 남은 미방출 목록을 무환불 폐기합니다. 동일 재입력 ID는 새 목록을 중복 생성하지 않습니다.",
    [incoming(), input("right", "이번 사출 권리", "portal-release-right")],
    [
      next(),
      output("volley", "확정 사출 목록", "frozen-volley"),
      cue("사출량 확정"),
    ],
  ),
  define(
    "serial-projectiles",
    "착탄 정산을 기다리는 순차 사출",
    "스킬 동작",
    "확정 목록을 한 번 소비합니다. 살아 있는 유효 적을 선택하고 현재 발사 원점에서 보구 한 발을 실제 투사체로 내보냅니다. 그 탄환의 적중/빗나감/소멸, 범위 피해·상태·사망·회수까지 모두 완료한 뒤 다음 표적을 새로 선택합니다. 비행 중 두 번째 탄환을 내보내지 않습니다. 상태 없는 적이 없으면 같은 적도 다시 선택할 수 있습니다. 유효 대상이 없으면 남은 목록을 무환불 폐기하고 종료합니다. 자연 완료와 외부/직접 중단을 구분하며 이번 전개 사용을 한 번 종료합니다. 이미 방출한 탄환의 뒤늦은 결과는 새 발사나 종료한 집중 재개를 만들지 않습니다.",
    [
      incoming(),
      input("volley", "확정 사출 목록", "frozen-volley"),
      input("profile", "보구 투사체", "projectile-profile"),
      input("selector", "매 발의 새 대상 선택", "enemy-selector"),
    ],
    [
      output("completed", "전체 사출 자연 완료", "flow"),
      output("cancelled", "사출 중단", "flow"),
      output("batch", "이번 사출 묶음", "projectile-batch"),
      cue("순차 사출"),
    ],
  ),
  define(
    "portal-stop-input",
    "전개·사출 직접 종료 입력",
    "스킬 사건",
    "이번 전개의 차징 또는 연속 사출 중 직접 종료 메뉴를 제공합니다. 미방출 준비물·후속 발사를 무환불 폐기하고 공명 잠금과 제어를 정리합니다. 이미 방출한 독립 투사체는 정산까지 남습니다. 중복 종료는 무변경이며 자연 완료 후속 효과를 만들지 않습니다.",
    [input("charge", "이번 전개", "portal-charge")],
    [cue("집중 직접 종료")],
  ),
];
