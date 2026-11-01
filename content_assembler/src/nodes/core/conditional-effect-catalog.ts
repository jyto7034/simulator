import type { Definition } from "../types";
import {
  define,
  input,
  output,
  multipleInput,
  number,
  integer,
  choice,
  unresolved,
} from "./skill-definition";

const condition = () => input("condition", "활성 장착 조건", "equipment-match");
const projectile = () => input("target", "변경할 투사체", "projectile-profile");
export const conditionalEffectCatalog: Definition[] = [
  define(
    "modifier-projectile-status",
    "장착 조건의 적중 후 상태",
    "스킬 상태",
    "연결한 투사체의 유효 직접 적중마다 원래 피해 뒤 살아 있는 적에게 상태를 적용합니다. 착탄 범위의 각 적을 포함하고 빗나감·파생 폭발·다른 무기에는 적용하지 않습니다. 원래 후속 상태와 공존하며 피해→기존 상태→추가 상태→사망/회수 정산을 끝내야 다음 직렬 사출을 허용합니다.",
    [condition(), projectile(), input("status", "추가할 상태", "hit-status")],
    [],
    [],
  ),
  define(
    "modifier-projectile-push",
    "장착 조건의 적중 밀어내기",
    "스킬 상태",
    "연결한 방향 투사체의 직접 피해 뒤 비행 방향으로 대상을 지정 거리만큼 즉시 옮깁니다. 대상의 이동 가능 마지막 타일에서 멈추고 벽·경계·점유를 통과하지 않습니다. 이동 면역은 위치만 유지하며 피해·후속 기절·원래 탄환 관통은 유지합니다. 추가 충돌 피해·재적중을 만들지 않습니다. 같은 탄환/대상은 한 번만 처리합니다.",
    [condition(), projectile()],
    [],
    [number("tiles", "밀어내기 거리 (타일)")],
  ),
  define(
    "modifier-projectile-wound",
    "지정 탄환의 출처별 스택 약화",
    "스킬 상태",
    "한 장착 사본의 이 선언이 연결한 투사체 직접 피해만 증폭합니다. 대상별 기존 스택×증가율을 합산해 피해를 처리한 뒤 생존 대상에게 새 스택을 부여하고 전체 수명을 갱신합니다. 첫 탄환은 자기 새 스택으로 증폭하지 않습니다. 유효 적중이면 보호막만 감소·HP 피해 0도 부여하고 회피·완전 무효는 제외합니다. 다른 착용자·무기·파생 폭발·침식은 공유하지 않습니다. 상한 초과는 버리고 만료·대상 이탈·전투 종료에 제거합니다. 여러 동시 적중은 안정 탄환 ID순으로 처리합니다.",
    [
      condition(),
      multipleInput(
        "profiles",
        "같은 약화를 사용하는 탄환",
        "projectile-profile",
        true,
      ),
      input("duration", "약화 지속 시간", "seconds"),
    ],
    [output("wound", "출처별 탄환 약화", "projectile-wound")],
    [
      integer("amount", "적중당 새 스택", 1),
      integer("cap", "최대 스택", 1),
      number("percent", "스택당 직접 탄환 피해 증가 (%)"),
    ],
  ),
  define(
    "modifier-wound-grant",
    "장착 조건의 탄환 스택 부여 배율",
    "스킬 상태",
    "지정한 스택 약화 선언의 새 부여량에만 배율을 곱하고 내립니다. 기존 스택·상한·스택당 피해·지속 시간을 바꾸지 않습니다.",
    [condition(), input("target", "스택 약화 선언", "projectile-wound")],
    [],
    [number("factor", "새 부여 배율")],
  ),
  define(
    "modifier-projectile-explosion",
    "관통 사건의 독립 폭발",
    "스킬 상태",
    "직접 피해와 적중 상태·밀어내기를 처리한 뒤 탄환이 적을 지나 계속 비행하는 실제 관통 사건마다 적중 당시 중심에 폭발을 한 번 예약합니다. 마지막 적중으로 소멸한 탄환은 관통이 아닙니다. 폭발 때 범위 안 생존 적에게 대상당 한 번 피해를 주며 원래 적도 포함합니다. 아군은 제외하고 공중은 명시합니다. 서로 다른 관통 폭발은 겹쳐 맞을 수 있으나 같은 폭발은 중복하지 않습니다. 장착 사본의 파생 피해이며 기본 공격·관통·표식·기절·다시 폭발을 유발하지 않습니다. 방출된 예약은 시전자 이탈에도 유지하고 전투 종료에 제거합니다.",
    [
      condition(),
      projectile(),
      input("power", "폭발 기준 위력", "damage-power"),
      input("range", "폭발 타일", "tile-pattern"),
      input("explosion", "폭발 피해 규칙", "explosion-rule"),
    ],
    [],
    [
      choice("air", "공중 대상", [
        unresolved,
        ["include", "포함"],
        ["exclude", "제외"],
      ]),
    ],
  ),
  define(
    "modifier-continuous-volley",
    "HP 비용의 무한 직렬 사출",
    "스킬 상태",
    "연결한 포탈 행동을 HP 비용 집중 사출로 전환합니다. 시작 공명은 원래대로 한 번 소비하고 종료까지 획득을 잠급니다. 전개·재고 소비·대여·붕괴·총 발수·최대 기한 대신 한 발의 정산 후 지정 간격을 기다려 다음 표적을 선택합니다. 재고가 0/음수여도 시작하며 기존 빚과 회수는 유지합니다. 시작 후 매 온전한 비용 주기에 당시 최대 HP×비율을 올림해 직접 차감합니다. 보호막·감면·피해 HP 하한을 거치지 않으며 치유는 허용합니다. 적이 없어도 비용은 계속 지불합니다. 동일 시각 비용/사망이 새 사출보다 먼저입니다. 직접 중지·사망·철수·공통 집중 중단으로 끝나고 이미 방출한 탄환만 남습니다. Pause는 비용과 사출을 모두 동결합니다.",
    [
      condition(),
      input("target", "전환할 포탈 시작 설정", "skill-program"),
      input("interval", "정산 후 다음 발사 간격", "seconds"),
      input("costInterval", "HP 비용 주기", "seconds"),
    ],
    [],
    [{ ...number("percent", "최대 HP 비용 (%)"), max: 100 }],
  ),
  define(
    "modifier-single-support",
    "장착 조건의 단일 아군 지원",
    "스킬 상태",
    "연결한 범위 지원 행동의 대상 선택을 지정 범위 안 아군 한 명으로 바꿉니다. 원래 대상 자격을 유지하고 대상·비용을 함께 검사합니다. 능력치 기여는 명시한 대체 효과를 사용하고 개인 보호막은 지정 배율만 적용합니다. 지원 범위 증폭은 적용하지 않고 선택 거리는 별도입니다. 집중형은 선택 대상이 선택 범위를 벗어나거나 무효가 되면 종료하며 다른 대상에 옮기지 않습니다. 방출형 보호막은 범위 이탈 후에도 원래 수명을 유지합니다. 다른 효과·시간·공명·중단·기본 공격은 원래대로입니다.",
    [
      condition(),
      input("target", "범위 지원 행동", "action-plan"),
      input("range", "단일 대상 선택 범위", "tile-pattern"),
      multipleInput(
        "effects",
        "집중형의 대체 능력치 기여",
        "target-effect-profile",
      ),
    ],
    [],
    [
      choice(
        "mode",
        "효과 전환",
        [
          ["attributes", "명시한 능력치로 대체"],
          ["shield", "개인 보호량 증폭"],
        ],
        "attributes",
      ),
      number("factor", "개인 보호막 배율", 1),
    ],
  ),
  define(
    "modifier-coincident-area-burst",
    "두 중심 일치의 일회 영역 전환",
    "스킬 상태",
    "같은 접수 사용의 두 중심 타일이 완전히 같고 장착 조건이 활성일 때 영역 생성을 일회 판정으로 교체합니다. 비용은 원래대로 소비합니다. 지연 뒤 지정한 같은 중심의 범위에서 대상별 한 번 판정하고 사용을 끝냅니다. 환상체/보스 분류를 우선해 최대 HP 비율 고정 피해를 주고, 그 외 아군(시전자 포함)·침식체는 직접 사망시킵니다. 일반 적·중립은 제외합니다. 비례 피해는 보호막·무적·생존 하한·페이즈 규칙을 따르며 직접 사망은 HP 하한을 거치지 않습니다. 회복·전달·버프·지속 영역을 생성하지 않습니다. 다른 시전/부분 교집합은 발동하지 않습니다. 시전자 이탈·전투 종료는 지연 판정을 취소합니다. 발견 전 플레이어 설명·경고·위험 조준을 추가하지 않습니다.",
    [
      condition(),
      input("target", "교체할 두 영역 생성", "paired-areas"),
      input("range", "일회 판정 범위", "tile-pattern"),
      input("delay", "출현 후 판정 지연", "seconds"),
    ],
    [],
    [{ ...number("percent", "환상체·보스 최대 HP 피해 (%)"), max: 100 }],
  ),
];
