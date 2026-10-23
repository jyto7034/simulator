import type { Definition } from "../types";
import {
  define,
  input,
  output,
  next,
  cue,
  number,
  choice,
  unresolved,
  integer,
} from "./skill-definition";
import { bankInput, grantOutput } from "./inventory-catalog";

export const spawnPortTypes = [
  ["hostile-profile", "생성 적 정의"],
  ["hostile-cohort", "이번 사본의 생성 적 집합"],
] as const;
export const spawnCatalog: Definition[] = [
  define(
    "hostile-profile",
    "경로를 따르는 적 유닛",
    "스킬 값",
    "일반 적과 같은 HP·방어·이동·저지·공격·탈출 판정을 갖는 실제 전투 유닛 정의입니다. 경로를 따라 전진하고 저지 상대/공격 가능한 아군 또는 방어 대상을 공격합니다. 적의 일반 타겟팅·공격 주기·제어 면역 기본 규칙을 사용합니다. 출현 위치와 귀속은 생성기가 정하며 연출만 있는 소환물이 아닙니다. 모든 미정 능력치를 명시적으로 남깁니다.",
    [
      input("range", "적 기본 공격 타일 범위", "tile-pattern"),
      input("interval", "적 기본 공격 주기", "seconds"),
    ],
    [output("profile", "적 정의", "hostile-profile")],
    [
      integer("hp", "최대 HP", 1),
      integer("power", "기본 공격 위력"),
      number("speed", "이동 속도 (타일/초)"),
      integer("physicalDefense", "물리 방어"),
      integer("magicDefense", "마법 방어"),
      integer("block", "저지 요구량", 1),
      integer("leak", "탈출 시 방어 대상 피해", 1),
      choice("damageType", "적 공격 피해 유형", [
        unresolved,
        ["physical", "물리"],
        ["magic", "마법"],
      ]),
    ],
  ),
  define(
    "hostile-family-weakness",
    "출처 계열 한정 적 약점",
    "스킬 값",
    "기본 적 정의에 지정 콘텐츠 계열의 파편 또는 무기에서 나온 피해에 한정한 받는 피해 증가를 더합니다. 해당 장비를 착용한 직원의 다른 피해로 확장하지 않습니다. 피해 source의 계열 ID를 비교하며 표시 이름을 해석하지 않습니다. 계열 ID는 콘텐츠 연결 시 검증할 식별자이고 미등록 계열은 실행용 발행 오류입니다.",
    [input("base", "기본 적 정의", "hostile-profile")],
    [output("profile", "약점 포함 적 정의", "hostile-profile")],
    [
      {
        id: "family",
        label: "파편·무기 계열 ID",
        valueType: "text",
        default: "",
      },
      number("percent", "해당 출처 피해 증가 (%)"),
    ],
  ),
  define(
    "hostile-spawner",
    "장착 중 적 주기 출현",
    "스킬 사건",
    "파편 사본을 장착해 진입한 전투에서 배치 여부와 무관하게 전투 시간의 첫 온전한 주기 뒤부터 적을 생성합니다. 유효 적 출현 경로의 안정된 순서를 순환하여 입구에 생성하고 정상 경로를 따르게 합니다. Pause에는 멈추며 소유자 철수·사망으로 장착 사본의 주기를 초기화하지 않습니다. 생성 적은 아군을 공격하고 실제 남은 적으로 전투 종료 판정에 포함합니다. 아직 발생하지 않은 미래 출현 주기 자체는 필수 웨이브가 아닙니다. 같은 시각의 승패 확정이 새 출현보다 우선하며 전투 종료 뒤 더 생성하지 않습니다. 전장에 유효 출현 경로가 없으면 콘텐츠 연결 오류로 거절합니다.",
    [
      input("profile", "생성할 적", "hostile-profile"),
      input("interval", "적 출현 주기", "seconds"),
    ],
    [output("cohort", "이 사본의 생성 적", "hostile-cohort"), cue("적 출현")],
  ),
  define(
    "cohort-killed",
    "생성 적의 처치 보상 권리",
    "스킬 사건",
    "이 생성기의 실제 적이 아군 귀속 피해로 HP 0 사망 확정되면 원래 생성한 파편 사본의 저장소에 획득 권리를 한 번 만듭니다. 다른 아군의 막타도 인정하지만 다른 생성기/사본의 적과 보상을 공유하지 않습니다. 탈출·수동 제거·전투 종료 정리는 처치가 아닙니다. 같은 적의 다단 피해·중복 사망 콜백은 권리를 중복 생성하지 않습니다. 방출 뒤 소유자가 이탈했어도 전투 수명 저장소의 보상은 원래 사본에 남습니다.",
    [input("cohort", "생성 적 집합", "hostile-cohort"), bankInput()],
    [next(), grantOutput()],
  ),
];
