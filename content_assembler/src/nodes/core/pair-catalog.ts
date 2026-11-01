import type { Definition } from "../types";
import {
  cue,
  define,
  incoming,
  input,
  next,
  number,
  output,
} from "./skill-definition";
export const pairPortTypes = [
  ["paired-areas", "이번 두 영역"],
  ["pair-pulse", "두 영역의 이번 주기"],
  ["pair-heal-result", "양쪽 기본 회복 결과"],
  ["pair-transfer-result", "추가 회복 결과"],
  ["attack-speed-buff", "기본 공격 속도 버프"],
] as const;
const pair = () => input("pair", "이번 두 영역", "paired-areas");
export const pairCatalog: Definition[] = [
  define(
    "two-tile-selection",
    "두 타일 선택 설정",
    "스킬 값",
    "첫 위치와 두 번째 위치를 순서대로 선택합니다. 두 타일 모두 같은 선택 범위/자격을 만족해야 하며 같아도 허용합니다. 최종 접수에서 두 위치와 공명 비용을 함께 검사합니다. 한쪽 선택이나 취소는 비용·영역을 만들지 않습니다. 지정 범위는 효과 범위와 별개입니다.",
    [input("range", "두 타일의 선택 범위", "tile-pattern")],
    [output("selection", "대상 선택 설정", "target-selection")],
  ),
  define(
    "paired-areas",
    "독립 두 영역 동시 생성",
    "스킬 동작",
    "두 위치를 접수한 같은 사용에서 첫/두 번째 타일의 고정 영역을 원자적으로 생성합니다. 패턴은 회전하지 않습니다. 같은 사본의 이전 두 영역은 새 생성 성공 때 함께 교체하고 실패 시 유지합니다. 두 영역은 같은 양수 수명을 공유하며 생성 동작 뒤 사용을 끝내고 자유 행동/공명 충전을 재개합니다. 시전자 사망·철수·전투 종료에 함께 정리하며 Hard CC·침묵은 이미 생성한 영역을 취소하지 않습니다. 다른 사본의 영역과 교집합을 만들지 않습니다.",
    [
      incoming(),
      input("use", "이번 사용", "skill-use"),
      input("first", "첫 위치 효과 패턴", "tile-pattern"),
      input("second", "두 번째 위치 효과 패턴", "tile-pattern"),
      input("duration", "공통 유지 시간", "seconds"),
    ],
    [
      next(),
      output("pair", "이번 두 영역", "paired-areas"),
      cue("두 영역 생성"),
    ],
  ),
  define(
    "pair-pulse",
    "두 영역 동기 회복 주기",
    "스킬 사건",
    "같은 두 영역의 첫 주기 뒤부터 양쪽 현재 아군 집합을 한 회차로 고정합니다. 범위 안 시전자도 포함합니다. 겹친 아군은 현재 타일의 맨해튼 거리로 가까운 중심에 한 번만 배정하고 동률은 첫 영역입니다. 교집합 버프 자격은 이 배정과 별개입니다. 종료 시각에는 주기 대신 정리가 우선하고 Pause의 밀린 회차는 없습니다. 주기/대상/영역/사본 ID를 보존합니다.",
    [pair(), input("interval", "회복 주기", "seconds")],
    [next(), output("pulse", "이번 회복 주기", "pair-pulse")],
  ),
  define(
    "pair-base-heal",
    "양쪽 기본 회복 확정",
    "스킬 동작",
    "같은 주기의 고정 대상 집합에 기본 회복을 대상당 한 번씩 적용하고 모두 처리한 뒤 양쪽 결과를 고정합니다. 각 대상의 요청량·실제 HP 회복·HP 상한으로 남은 양·거절을 구분합니다. 면역/회복 불가/소멸로 거절된 양은 초과분이 아닙니다. 빈 영역은 결과/예산 0입니다. 시전자 회복 증가와 수혜자 보정은 정상 파이프라인에서 적용합니다. 다른 주기 결과를 합치지 않습니다.",
    [
      incoming(),
      input("pulse", "이번 회복 주기", "pair-pulse"),
      input("power", "대상별 기본 회복", "healing-power"),
    ],
    [
      next(),
      output("result", "양쪽 기본 회복 결과", "pair-heal-result"),
      cue("기본 회복"),
    ],
  ),
  define(
    "pair-transfer",
    "반대 영역 추가 회복",
    "스킬 동작",
    "같은 주기의 양쪽 기본 회복이 모두 확정된 뒤 각 쪽 HP 상한 초과분 합계 × 전달률을 내림해 반대편 예산을 만듭니다. 반대 소속의 현재 부상 아군에 남은 결손 HP 비례로 배분하고 정수 잔여는 안정된 ID 순서로 나눕니다. 배정이 HP 상한을 넘거나 대상이 사라지면 남은 양을 버립니다. 빈 쪽은 예산 0이며 전달된 회복/초과분을 다시 전달하지 않습니다. 시전자의 동일 회복 증가를 재적용하지 않고 수혜자의 받는 회복 보정은 따릅니다. 주기당 한 번만 실행합니다.",
    [
      incoming(),
      input("pulse", "이번 회복 주기", "pair-pulse"),
      input("result", "양쪽 기본 회복 결과", "pair-heal-result"),
    ],
    [
      next(),
      output("result", "전달 결과 · 재전달 불가", "pair-transfer-result"),
      cue("순환 회복"),
    ],
    [{ ...number("percent", "초과분 전달률 (%)"), max: 100 }],
  ),
  define(
    "attack-speed-buff",
    "기본 공격 속도 증가",
    "스킬 값",
    "기본 공격/기본 치료의 공격 속도에 더하는 양수 포인트입니다. 중립 100 기준이며 주기 퍼센트 감소나 파편/회복 판정 가속이 아닙니다. 같은 버프의 여러 출처는 가장 큰 기여분만 적용하고 종료는 해당 출처만 제거합니다.",
    [],
    [output("buff", "공격 속도 버프", "attack-speed-buff")],
    [number("points", "기본 공격 속도 증가 (포인트)")],
  ),
  define(
    "pair-intersection",
    "같은 시전 교집합 버프",
    "스킬 상태",
    "이번 두 영역이 모두 살아 있는 동안 실제 교집합 안의 아군(범위 안 시전자 포함)에 연결한 버프를 한 번 부여합니다. HP 회복 성공이나 주기 집계 소속을 조건으로 삼지 않습니다. 교집합 진입 시 부여하고 이탈·대상 소멸·두 영역 종료/교체에 이번 출처만 제거합니다. 다른 사본의 한 영역과 교집합을 조합하지 않습니다. 선언의 연결이 수명 구독이며 별도 실행 노드를 빠뜨려 정리가 생략되지 않습니다.",
    [pair(), input("buff", "교집합 버프", "attack-speed-buff")],
    [cue("교집합 버프 변화")],
  ),
];
