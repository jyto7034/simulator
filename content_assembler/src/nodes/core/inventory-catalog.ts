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
  integer,
} from "./skill-definition";

export const inventoryPortTypes = [
  ["resource-bank", "사본 자원 저장소"],
  ["resource-grant", "이번 획득 권리"],
  ["count-rule", "행동 회차의 개수 계산"],
] as const;
export const bankInput = () => input("bank", "자원 저장소", "resource-bank");
export const grantOutput = () =>
  output("grant", "이번 획득 권리", "resource-grant");

// These are authoring declarations, not mutable balances or a battle executor.
export const inventoryCatalog: Definition[] = [
  define(
    "stock",
    "사본별 정수 재고",
    "스킬 상태",
    "장착 파편 사본의 비음수 정수 재고입니다. 전투 수명은 전투 진입 때 한 번 초기 지급하며 직원 교체·철수·사망·재배치로 다시 지급하지 않습니다. 배치 수명은 배치 때 초기화하고 소유자 이탈 때 폐기합니다. 상한 적용은 획득 증폭 뒤이며 초과분은 버립니다. 두 종류 재고는 서로 다른 선언을 참조합니다. 전투 종료에는 양수 재고를 이월하지 않습니다. 선언은 획득 사건이 아닙니다.",
    [],
    [output("bank", "재고 정의", "resource-bank")],
    [
      integer("initial", "초기 재고"),
      choice("lifetime", "재고 수명", [
        unresolved,
        ["battle", "전투 · 재배치에도 유지"],
        ["deployment", "배치 · 이탈 시 폐기"],
      ]),
      choice("limit", "재고 상한", [
        unresolved,
        ["bounded", "지정 상한"],
        ["unbounded", "상한 없음"],
      ]),
      integer("capacity", "최대 재고", 1),
    ],
  ),
  define(
    "credit-ledger",
    "전투 지급과 자동 대여 원장",
    "스킬 상태",
    "파편 사본별 전투 잔고와 원정의 미상환액·직전 전투 대출 이력을 구분합니다. 시작 잔고=기본 지급−직전 미상환액, 대여 가능=직전 전투에 새 대여를 쓰지 않음입니다. 양수 이익은 이월하지 않고 종료 미상환액=max(0,−잔고)입니다. 생성 비용을 양수 잔고로 못 내면 허용 전투에서 하한 −대여 한도까지 자동 대여합니다. 이번에 빚을 갚아도 대출 사용 이력은 지우지 않습니다. 음수 시작은 새 대출이 아닙니다. 같은 사본의 미참전 전투는 정산하지 않고, 전투에 장착해 진입한 사본은 실제 배치와 무관하게 승패 확정 때 한 번 정산합니다. 직원 교체·재배치로 초기화하지 않습니다. Retreat는 진입 전 savepoint, 원정 종료는 초기 상태로 복원합니다. 합성·양도는 원장 승계 계약 없이는 실행용 발행을 허용하지 않습니다.",
    [],
    [output("bank", "원장 정의", "resource-bank")],
    [
      integer("grant", "전투 기본 지급", 1),
      integer("creditLimit", "미상환 대여 한도", 1),
    ],
  ),
  define(
    "stock-periodic",
    "재고 획득 주기",
    "스킬 사건",
    "같은 사본의 전투 시간으로 첫 온전한 주기 뒤부터 획득 권리를 만듭니다. 장착 중은 전투 진입부터 배치 여부와 무관하고 배치 중은 살아서 배치된 동안만 셉니다. Pause에는 흐르지 않고 중지 시간을 보충하지 않습니다. 공명 잠금·Hard CC·침묵은 이 패시브를 막지 않습니다. 저장소와 발생 회차를 권리에 보존하며 재고 변경은 연결한 획득 동작이 수행합니다.",
    [bankInput(), input("interval", "획득 주기", "seconds")],
    [next(), grantOutput()],
    [
      choice("scope", "주기 진행 조건", [
        unresolved,
        ["equipped", "전투 중 장착"],
        ["deployed", "생존·배치 중"],
      ]),
    ],
  ),
  define(
    "resource-credit",
    "획득 권리의 자원 지급",
    "스킬 동작",
    "같은 사건 성공 흐름의 저장소·사본·회차에 결속된 권리를 한 번 소비해 양수 정수 재고를 더합니다. 주기, 관계 주기, 생성 적 처치, 회수 상태 사망의 권리를 받습니다. 같은 권리에 지급을 중복 연결하지 않습니다. 원장의 음수 잔고에는 먼저 빚을 갚는 결과가 되고 대출 이력은 유지합니다. 공명 획득 잠금과 별개이며 다른 사본으로 지급하거나 타이머를 초기화하지 않습니다.",
    [
      incoming(),
      bankInput(),
      input("grant", "이번 획득 권리", "resource-grant"),
    ],
    [next(), cue("재고 지급")],
    [integer("amount", "기본 획득량", 1)],
  ),
  define(
    "count-constant",
    "정수 개수",
    "스킬 값",
    "0 이상의 안전한 정수 개수입니다. 행동 계획의 타수·상태 스택·비용에 사용하며 공명이나 실제 피해 결과와 구분합니다. 빈 값은 미정입니다.",
    [],
    [output("count", "개수 계산", "count-rule")],
    [integer("value", "개수")],
  ),
  define(
    "count-positive",
    "양수 정수 개수",
    "스킬 값",
    "1 이상의 안전한 정수 개수입니다. 적어도 한 번 실행해야 하는 기본 타수 등에 사용합니다. 빈 값은 미정이며 0을 입력해 필수 행동을 조용히 생략하지 않습니다.",
    [],
    [output("count", "양수 개수 계산", "count-rule")],
    [integer("value", "개수", 1)],
  ),
  define(
    "count-stock",
    "행동 시작 재고 고정",
    "스킬 값",
    "행동 실행기가 처음 실제 행동을 시작할 때 해당 사본의 비음수 재고를 한 번 고정합니다. 다음 기본 공격 대기 중에는 아직 고정하지 않습니다. 같은 실행 회차의 모든 참조는 같은 값을 읽고 이후 획득·소비로 바뀌지 않습니다. 다른 사용·회차의 값을 공유하지 않습니다. 음수 원장에는 사용할 수 없습니다.",
    [bankInput()],
    [output("count", "시작 재고 계산", "count-rule")],
  ),
  define(
    "count-quotient",
    "몫과 최소 개수",
    "스킬 값",
    "같은 행동 시작 값으로 max(최소 개수, floor(입력 개수 / 나눌 수))를 계산합니다. 매 소비 직전의 잔고로 다시 나누지 않습니다. 산술 오버플로는 실행용 발행/실행 오류이며 조용히 잘라내지 않습니다.",
    [input("count", "기준 개수", "count-rule")],
    [output("count", "단계 개수 계산", "count-rule")],
    [integer("divisor", "나눌 수", 1), integer("minimum", "최소 개수")],
  ),
  define(
    "count-sum",
    "개수 합계",
    "스킬 값",
    "같은 행동 회차의 두 비음수 개수를 더합니다. 실제 소비가 필요한 자리에 예상 합계를 소비 영수증으로 대신하지 않습니다. 안전한 정수 범위를 넘으면 거절합니다.",
    [
      input("first", "첫 개수", "count-rule"),
      input("second", "추가 개수", "count-rule"),
    ],
    [output("count", "합계 계산", "count-rule")],
  ),
  define(
    "count-paid",
    "이번 단계 실제 소비 개수",
    "스킬 값",
    "가장 가까운 자원 소비 단계가 성공했을 때 확정한 실제 소비량입니다. 그 단계의 본문과 단계 후 상태에서만 읽을 수 있습니다. 전량 소비 뒤 현재 잔고 0이나 행동 시작 재고로 대신하지 않습니다. 소비 단계 밖·거절 경로·다른 단계로 전달할 수 없습니다.",
    [],
    [output("count", "이번 단계 소비 계산", "count-rule")],
  ),
];
