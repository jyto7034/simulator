import type { Definition } from "../types";
import { define, input, output, number, integer } from "./skill-definition";

const identifier = (id: string, label: string) => ({
  id,
  label,
  valueType: "text" as const,
  default: "",
});
export const weaponStatePortTypes = [
  ["weapon-evolution", "무기 사본의 영구 승급"],
] as const;
export const weaponStateCatalog: Definition[] = [
  define(
    "weapon-mark-owner",
    "무기 사본의 공유 표식 소유권",
    "스킬 상태",
    "연결한 표식 저장소의 소유권을 승급 정의의 실제 무기 사본·상태 ID로 고정합니다. 기본/승급 단계·파편 형태의 별도 그래프도 같은 무기 사본과 상태 ID면 같은 저장소를 참조합니다. 파편 사본을 키에 섞지 않습니다. 무기 이전·강화·승급은 신규 부여가 아니며 전투 종료에는 정리합니다. 같은 상태 ID의 서로 다른 저장 규칙은 컴파일 오류입니다.",
    [
      input("marks", "공유할 표식", "target-marks"),
      input("evolution", "무기 정의와 단계", "weapon-evolution"),
    ],
    [],
    [identifier("key", "형태 간 공유 상태 ID")],
  ),
  define(
    "integer-amount",
    "정수 수량 값",
    "스킬 값",
    "0 이상의 정수 수량입니다. 빈 값은 미정이며 0으로 대신하지 않습니다.",
    [],
    [output("value", "수량", "number")],
    [integer("value", "수량")],
  ),
  define(
    "weapon-evolution",
    "개화와 무기 사본의 영구 승급",
    "스킬 상태",
    "지정 파편 사본의 개화 해금 거래에 명시적으로 선택한 보유 무기 사본 하나를 함께 승급합니다. 무기 정의·현재 단계·소유권과 개화 비용을 모두 검증한 뒤 함께 확정하며 대상 없음·불일치는 무변경 거절입니다. 무기 인스턴스 ID와 부착 상태를 유지합니다. 전투 종료·공명 소비·파편 교체·기본형 선택으로 되돌리지 않습니다. 전투 진입 시 자동 승급하거나 같은 종류 전부를 승급하지 않습니다. 새 획득품에는 소급하지 않습니다. 반복 해금은 승급을 중복 지급하지 않습니다.",
    [],
    [output("evolution", "영구 승급 정의", "weapon-evolution")],
    [
      identifier("fragment", "개화할 파편 정의 ID"),
      identifier("form", "해금 형태 ID"),
      identifier("weapon", "동반 승급 무기 정의 ID"),
      identifier("base", "승급 전 단계 ID"),
      identifier("upgraded", "승급 후 단계 ID"),
    ],
  ),
  define(
    "weapon-kill-growth",
    "무기 주 대상 막타의 원정 성장",
    "스킬 상태",
    "지정 무기 사본의 직접 기본 공격·기본 공격 교체가 확정한 주 대상의 실제 사망에서 한 번 획득합니다. 주변 타격·독립 스킬·지속·반사·proc·침식·임시 소환·비사망 이탈은 제외합니다. 환상체→엘리트→일반의 한 분류만 사용합니다. 무기 사본과 상태 ID로 분리하며 이전·강화·전투 종료에 보존, 원정 종료에 제거, 후퇴·체크포인트에서는 저장 시점으로 복원합니다. 새 획득 전의 저장량×스택당 피해를 주 대상 기본 피해에 더하고 주변에는 주변 계수를 곱합니다. gameplay 상한 없이 안전 정수 포화와 최종 피해 clamp를 적용합니다. 다른 무기 인스턴스 생성에는 이전하지 않습니다.",
    [
      input("normal", "일반 적 막타 획득량", "number"),
      input("elite", "엘리트 막타 획득량", "number"),
      input("abnormality", "환상체 막타 획득량", "number"),
    ],
    [],
    [
      identifier("weapon", "무기 정의 ID"),
      identifier("key", "무기 사본 상태 ID"),
      number("damage", "스택당 주 대상 추가 피해"),
      { ...number("secondaryPercent", "주변 추가 피해 비율 (%)"), max: 100 },
    ],
  ),
];
