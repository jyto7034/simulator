import type { Definition } from "../types";
import {
  choice,
  cue,
  define,
  incoming,
  input,
  next,
  output,
  unresolved,
} from "./skill-definition";

export const channelPortTypes = [["skill-channel", "이번 집중"]] as const;
const channel = () => input("channel", "이번 집중", "skill-channel");

// Authoring contracts only. The Core executor will own timers and termination.
export const channelCatalog: Definition[] = [
  define(
    "channel-begin",
    "집중 시작",
    "스킬 동작",
    "접수된 이번 사용에서 집중을 한 번 시작합니다. 양수 최대 전투 시간 동안 사용·공명 획득 잠금과 전술 이동·교대 제한을 유지합니다. 기본 공격 정책을 적용하되 이미 발사한 공격은 유지합니다. 만료·직접 취소·사망·철수·전투 종료에서 끝나며 표준 중단은 Hard CC에서도 끝납니다. 외부 중단 보호는 진행 중인 집중만 보호하며 상태 면역이 아닙니다. 침묵은 진행 중인 집중을 취소하지 않습니다. 종료 시 종속 장판·효과를 정리하고 사용 잠금을 해제한 뒤 종료 사건을 한 번 냅니다. 환불·자동 재개는 없습니다.",
    [
      incoming(),
      input("use", "이번 사용", "skill-use"),
      input("duration", "최대 집중 시간", "seconds"),
    ],
    [next(), output("channel", "이번 집중", "skill-channel"), cue("집중 시작")],
    [
      choice("basicAttack", "집중 중 기본 공격", [
        unresolved,
        ["pause", "중지"],
        ["allow", "유지"],
      ]),
      choice("interruption", "외부 중단", [
        unresolved,
        ["standard", "표준 · Hard CC 취소"],
        ["protected", "진행 중 집중 보호"],
      ]),
    ],
  ),
  define(
    "channel-cancel-request",
    "집중 직접 취소 입력",
    "스킬 사건",
    "현재 살아 있는 이번 집중에 대한 플레이어의 직접 취소 입력입니다. 새 스킬 시전이나 비용 지불이 아니며 침묵·사용 잠금 중에도 허용합니다. 다른 시전 회차나 이미 종료한 집중에는 발생하지 않습니다. 집중 취소 동작으로 연결합니다.",
    [channel()],
    [output("out", "취소 요청", "flow")],
  ),
  define(
    "channel-cancel",
    "집중 취소",
    "스킬 동작",
    "같은 집중의 시작 후 흐름 또는 직접 취소 입력에서 집중을 종료합니다. 정리와 사용 잠금 해제는 집중의 공통 종료 절차를 따릅니다. 최대 시간·사망 등과 겹쳐도 한 번만 종료하며 재활성화하거나 비용을 환불하지 않습니다. 종료 후 처리는 집중 종료 사건에 연결합니다.",
    [incoming(), channel()],
    [],
  ),
  define(
    "channel-ended",
    "집중 종료 사건",
    "스킬 사건",
    "이번 집중의 모든 종료 원인에서 한 번 발생합니다. 종속 장판·체류 진행·상태 기여분 정리와 사용 잠금 해제 뒤 연출을 종료합니다. 다른 시전자·시전 회차의 장판이나 독립 수명 효과는 유지합니다. 종료 흐름으로 이 집중의 장판을 다시 만들 수 없습니다.",
    [channel()],
    [output("out", "정리 후 종료", "flow"), cue("집중 종료")],
  ),
  define(
    "channel-area",
    "집중 종속 장판 생성",
    "스킬 동작",
    "이번 집중이 살아 있는 시작 후 흐름에서 배치 범위를 고정해 장판 하나를 만듭니다. 시전자의 입력 접수 당시 위치·방향을 사용하고 이후 추종하지 않습니다. 수명은 이번 집중에 종속되며 별도 지속 시간을 더하지 않습니다. 집중의 모든 종료 원인에서 장판 구독·관계·상태를 먼저 정리합니다. 생성 자체는 피해나 회복을 주지 않으며 기존 장판 사건을 연결할 수 있습니다.",
    [incoming(), channel(), input("area", "배치 범위", "area-profile")],
    [
      next(),
      output("area", "이번 장판", "skill-area-instance"),
      cue("장판 생성"),
    ],
  ),
];
