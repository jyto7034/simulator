import type { Definition } from "../types";
import {
  define,
  input,
  output,
  cue,
  number,
  choice,
  unresolved,
  optionalInput,
  multipleInput,
  integer,
} from "./skill-definition";

export const projectilePortTypes = [
  ["projectile-profile", "실제 투사체 정의"],
  ["projectile-batch", "방출한 투사체 묶음"],
  ["projectile-result", "개별 투사체 정산 결과"],
  ["enemy-selector", "자동 적 선택 설정"],
  ["damage-mark", "출처별 회수 상태"],
] as const;
export const projectileCatalog: Definition[] = [
  define(
    "projectile-profile",
    "투사체 전달과 적중 효과",
    "스킬 값",
    "방출 뒤 실제 전투 시간으로 이동하는 투사체입니다. 방향 관통은 입력 계획의 원점·방향을 고정하고 적과 연속 충돌하며 같은 적은 해당 탄환에서 한 번만 맞습니다. 표적 착탄은 방출 때 고른 대상의 고정 목적 위치 또는 추적 위치로 이동하고 착탄 타일의 작성된 패턴 안 현재 적에게 한 번씩 피해를 줍니다. 표적을 다른 적으로 바꾸지 않습니다. 속도·거리·수명·충돌 폭과 대상/장애물 정책을 명시합니다. 최대 거리·수명 또는 장애물 종료도 정산 완료이며 빗나간 대상에게 효과를 주지 않습니다. 피해 뒤 상태는 개별 적중마다 적용합니다. 이미 방출한 투사체는 시전자 이탈/시전 중단에도 원래 수명을 따르되 전투 종료에는 정리합니다. 같은 묶음의 다른 탄환은 같은 적을 별도로 맞힐 수 있습니다.",
    [
      input("power", "피해 위력", "damage-power"),
      input("lifetime", "최대 비행 시간", "seconds"),
      optionalInput("area", "착탄 효과 타일 패턴", "tile-pattern"),
      multipleInput("after", "적중 뒤 상태", "hit-status"),
    ],
    [output("profile", "투사체 정의", "projectile-profile")],
    [
      choice("delivery", "전달 방식", [
        unresolved,
        ["directional", "고정 방향 관통"],
        ["impact", "표적 착탄 범위"],
      ]),
      choice("tracking", "비행 조준", [
        unresolved,
        ["fixed", "방출 위치·방향 고정"],
        ["homing", "고른 표적만 추적"],
      ]),
      choice("damageType", "피해 유형", [
        unresolved,
        ["physical", "물리"],
        ["magic", "마법"],
        ["true", "고정"],
      ]),
      number("speed", "비행 속도 (타일/초)"),
      number("range", "최대 비행 거리 (타일)"),
      number("width", "충돌 폭 (타일)"),
      choice("pierce", "방향 관통 횟수", [
        unresolved,
        ["unbounded", "유효 적 모두"],
        ["count", "최대 적중 수"],
      ]),
      integer("hits", "최대 적중 수", 1),
      choice("air", "공중 적", [
        unresolved,
        ["include", "포함"],
        ["exclude", "제외"],
      ]),
      choice("obstacle", "장애물 충돌", [
        unresolved,
        ["stop", "종료"],
        ["pass", "통과"],
      ]),
    ],
  ),
  define(
    "enemy-selector",
    "자동 적 선택 순서",
    "스킬 값",
    "살아 있는 적 중 현재 선택 범위·공중 자격을 만족하는 후보를 고릅니다. 루트 진행도는 시전자와의 직선거리가 아닙니다. 회수 상태 우선은 같은 파편 출처의 미만료 상태가 없는 적을 먼저 고르고 없으면 전체 후보로 돌아갑니다. 동률은 스폰 순서, 유닛 ID 순입니다. 저지 우선은 해당 시전자의 기존 저지 상대를 먼저 고르며 무기 변경으로 이 스킬 설정을 바꾸지 않습니다. 후보가 없으면 실패하며 임의 타일이나 죽은 적으로 대체하지 않습니다.",
    [
      input("range", "자동 선택 범위", "tile-pattern"),
      optionalInput("mark", "우선 제외할 자기 상태", "damage-mark"),
    ],
    [output("selector", "자동 선택 설정", "enemy-selector")],
    [
      choice("order", "선택 우선순위", [
        unresolved,
        ["route", "루트 진행도 우선"],
        ["block-route", "내 저지 상대·루트 진행도"],
        ["unmarked-route", "자기 상태 없음·루트 진행도"],
      ]),
      choice("air", "공중 후보", [
        unresolved,
        ["include", "포함"],
        ["exclude", "제외"],
      ]),
    ],
  ),
  define(
    "projectile-results",
    "방출 묶음의 개별 정산 사건",
    "스킬 사건",
    "이미 방출한 묶음의 개별 투사체가 적중·빗나감·거리/수명/장애물 종료로 최종 정산될 때 한 번 발생합니다. 결과는 사용·묶음·탄환 ID와 각 적의 실제 피해/상태/사망/회수 완료를 구분해 보존합니다. 정산은 적중 성공이나 HP 손실과 동의어가 아닙니다. 묶음 전체 완료나 새 시전 권리를 만들지 않으며 취소한 제어를 되살리지 않습니다.",
    [input("batch", "방출 묶음", "projectile-batch")],
    [
      output("out", "개별 정산 완료", "flow"),
      output("result", "개별 결과", "projectile-result"),
      cue("투사체 정산"),
    ],
  ),
];
