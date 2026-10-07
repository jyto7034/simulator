# 전장 컨셉아트 이미지 생성 가이드

2026-10-03 사용자 합의 기준. 이미지 생성 도구로 실제 전투가 벌어질 전장의 전체 모습을 시각화할 때 사용한다.
시설 전체의 방 연결 지도와 구분하며, **명일방주를 참고한 타일 전장, 방해물 외 1타일 규격, 어두운 폐허 시설,
이동 경로 표시 없는 배경 이미지**를 공통 기준으로 삼는다.

이 문서는 컨셉아트 제작 기준이다. 맵별 동선·보스 기믹·승패 조건이나 Unity 구현 완료를 확정하지 않는다.
전투의 기본 구조는 [차기 전투 규칙](../game_rules/combat.md#전투-목표와-전장),
타일과 실제 위치의 관계는 [저지와 공격 판정](../game_rules/blocking-and-targeting.md#타일과-연속-위치)을 따른다.
UI를 포함한 화면 시안은 별도의 [장면별 UI 설계](ui-layout.md)에서 다룬다.

## 생성할 이미지

명일방주의 전투 맵을 시각적 참고로 삼아, 시설 내부의 바닥 타일·지상 배치 공간·고지대·벽·방해물·출입구가
읽히는 전장 컨셉아트를 만든다. 일반 보스전도 같은 실시간 배치·경로 방어 전투의 기본 구조를 따른다.
실사 사진보다 실제 게임 전장으로 이해할 수 있는 표현을 사용한다.

기본 산출물에는 **전장 환경만** 그린다. 직원·적·보스 캐릭터, 체력바, 로스터, 버튼, 설명문, 타일 좌표,
선택 표시와 공격 범위 표시를 넣지 않는다. 특정 기믹에 필요한 시설물은 해당 맵의 요청에 따라 포함한다.

전체 전장을 위에서 비스듬히 내려다보는 기존 시안의 구도를 기본으로 사용한다. 바닥과 고저차를 함께
읽을 수 있어야 한다. 정확한 카메라 각도·투영 방식·이미지 해상도는 아직 공통 확정값이 아니다.

## 1타일 규격

**전장 내부는 방해물이 아닌 이상 동일한 1×1 타일 규격을 지킨다.** 바닥·배치 칸·고지대·작업 칸은
같은 격자에 맞춘다. 지상과 고지대가 서로 다른 크기의 타일을 사용해서는 안 된다.

- 넓은 공간과 고지대는 여러 1×1 타일을 이어서 만든다. 각 칸의 경계와 배치 자리 수가 읽혀야 한다.
- 하나의 고지대 상판을 경계 없는 대형 배치 칸처럼 그리지 않는다. 반 타일이나 임의 크기의 배치 칸을 만들지 않는다.
- 방해물은 크기 예외를 허용한다. 여러 칸에 걸치는 벽·잔해·기계도 가능하지만 주변 타일 규격을 바꾸지는 않는다.
- 원근에 따른 화면상 크기 차이와 실제 타일 규격의 차이를 구별한다. 균일한 격자를 같은 시점에서 투영한다.
- 균열·얼룩·마모는 표면 표현으로 사용한다. 장식 때문에 새로운 타일 경계나 이동 불가 구역처럼 보이지 않게 한다.

1타일 규격은 전장의 기본 단위다. 모든 맵을 같은 가로·세로 타일 수나 빈 직사각형으로 만들라는 뜻은 아니다.

## 이동 경로와 오버레이

**이미지에는 적의 이동 루트를 직접 그리지 않는다. 이동 경로 오버레이는 Unity에서 구현한다.**

| 이미지에서 제외할 표현 | 적용 기준 |
| --- | --- |
| 경로선·점선·화살표·진행 방향 표시 | 색이나 투명도와 무관하게 생성하지 않음 |
| 홀로그램·발광선 형태의 경로 미리보기 | 이미지에 합성하지 않고 Unity 표시로 남김 |
| 적이 지나는 타일만 밝히거나 다른 색으로 칠한 띠 | 바닥 재질로 이동 루트를 대신 표시하지 않음 |
| 이동 경로 전체를 따라가는 페인트·조명·홈 | 경로 안내를 배경에 고정하는 표현으로 사용하지 않음 |

벽·방해물·방 구조로 실제 통로를 만드는 것은 가능하다. 다만 넓은 바닥 위에서 적이 어느 선을 따라
이동하는지까지 이미지로 표시하지 않는다. 시설의 재질과 구역 차이는 표현할 수 있지만 적의 경로를
알려 주는 목적으로 바닥 타일을 구분하지 않는다.

출현구와 보호 지점에 필요한 물리적 문·시설물은 그릴 수 있다. 아이콘이나 방향 표시를 덧붙이지 않으며,
첫 시안의 붉은 입구·푸른 출구 색상을 모든 맵의 필수 규칙으로 사용하지 않는다.

## 폐허 시설의 분위기

**탐험 대상은 어두운 폐허 시설이다.** 밝고 깨끗하게 가동 중인 연구실을 기본 분위기로 사용하지 않는다.
기존 두 번째 시안처럼 오래 방치된 시설의 손상과 정적을 표현한다.

녹슨 철재·오염된 콘크리트·죽은 단말기·파손된 배관·잔해·희미한 비상등은 분위기를 만드는 표현 예시다.
이를 모든 맵에 같은 수량과 위치로 반복할 필요는 없다. 구역과 보스에 맞게 시설의 용도와 손상 양상을 바꾼다.

어두운 분위기에서도 타일 경계, 지상과 고지대, 방해물의 윤곽을 읽을 수 있어야 한다.
검게 뭉개진 바닥이나 짙은 안개로 전투 공간을 숨기지 않는다. 차가운 저채도 조명과 작은 경고등은
초기 시안의 표현 예시이며, 특정 색 조합이나 조도 수치를 확정한 것은 아니다.

## 제작 시 확인할 가독성

다음은 합의한 타일 규격과 전장 가독성을 지키기 위한 제작 점검 기준이다. 새로운 이동·점유 규칙을 만들지 않는다.

- 계단·경사로가 실제로 없는 연결을 암시하지 않는지 확인한다. 적의 고지대 진입을 의도하지 않았다면 장식 계단으로 오해를 만들지 않는다.
- 화면 앞쪽 벽·기둥·배관이 타일과 향후 배치될 유닛을 가리지 않게 한다. 필요하면 낮은 벽이나 절개된 구도로 표현한다.
- 배치 가능한 고지대와 단순 기계·상자를 시각적으로 구분한다. 모든 평평한 물체가 배치 칸처럼 보이지 않게 한다.
- 장식의 밝기와 바닥 무늬를 절제하여 Unity에서 표시할 공격 예고·선택·이동 오버레이가 묻히지 않게 한다.
- 보스의 큰 외형만으로 여러 타일 점유나 판정 범위를 새로 정하지 않는다. 보스 캐릭터와 판정은 별도 설계한다.

## 맵마다 따로 정할 내용

시설 구역의 용도, 전장의 크기와 외곽 형태, 출현구·보호 지점의 수와 위치, 지상·고지대·방해물 배치,
보스 기믹용 시설물은 개별 맵 설계에서 정한다. 단일 경로·두 번 꺾이는 길·특정 수의 고지대는 첫 시안을
만들기 위한 예시였으며 공통 규칙이 아니다.

이미지를 만들 때는 해당 맵에서 정한 정보를 아래 공통 프롬프트에 추가한다. 정하지 않은 수치나
임시 배치를 게임의 확정 규칙으로 기록하지 않는다.

## 재사용할 생성 프롬프트

```text
Create an environment-only battlefield concept for an Arknights-inspired
tile-based real-time tactical defense game, inside a dark abandoned facility.
Show the entire arena from an elevated three-quarter game view, with clear
ground tiles, raised deployment platforms, obstacles and physical entrances.

Every non-obstacle playable element must follow the same uniform 1x1 square
tile module. Ground and raised platform footprints align to one consistent
grid. Larger platforms consist of visibly separate full-sized tiles.
Only impassable obstacles may span irregular or multiple tile footprints.

Do not draw enemy movement routes in any form. No route lines, dashed lines,
arrows, holographic trails, glowing paths or route-specific floor colors.
Do not turn enemy routes into light-colored strips of tiles. Route overlays
will be implemented separately in Unity. Physical corridors are allowed.

Use a gloomy derelict industrial atmosphere with worn concrete, corroded
metal and sparse failing lights. Keep tile boundaries, elevation and obstacle
silhouettes readable. Avoid foreground scenery obscuring the playable area
and decorative stairs implying unintended access to raised tiles.

No characters, enemies, boss models, HUD, UI, text, labels, coordinates,
selection highlights, attack-range indicators, logos or watermarks.
Map-specific layout and setting: [insert this map's brief].
```

## 저장과 전달

생성한 이미지는 매번 Windows 다운로드 폴더로 옮긴다.

- Windows: `C:\Users\blast\Downloads`
- WSL: `/mnt/c/Users/blast/Downloads`

기존 이미지를 덮어쓰지 않고 맵 이름과 버전으로 구분한다. 예: `facility-battlefield-concept-03.png`.
이동 후 파일이 존재하는지 확인하고 저장 위치를 안내한다. 생성 도구의 기본 저장 위치에 둔 채 완료하지 않는다.

## 기존 시안의 참고 범위

| 다운로드 폴더의 파일 | 참고할 점 | 이후 이미지에 계승하지 않을 점 |
| --- | --- | --- |
| `facility-battlefield-concept-01.png` | 전장 전체 구도와 지상·고지대 표현 | 밝은 통로로 경로를 표시한 방식, 비교적 깨끗한 시설 분위기 |
| `facility-battlefield-concept-02-dark.png` | 어두운 폐허 분위기와 1타일 단위 표현 방향 | 주황색 점선·화살표 오버레이 전체 |

두 이미지는 분위기와 구도를 검토한 시안이다. 개별 타일의 정확한 배치나 실제 플레이 가능성을 검증한 도면은 아니다.
앞선 이미지와 이 가이드가 다르면, 경로 오버레이를 제외한 이 가이드의 최신 기준을 따른다.
