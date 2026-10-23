# Core 전체 현재 검증

사용자가 전체 Core 실패 유무를 확인했다. 이전138+추가17 교정은 개별/합집합 증거였고, G11 4개는 최신 manifest 환경변수를 지정했을 때 통과했다. 기본 DEFAULT_INPUTS가 구 U12 계약을 가리키는 한 무설정 실패0이 아니다. 기존 테스트 교정의 남은 기본 locator를 현재 U14 g11_inputs001 manifest로 변경한다. 역사 fixture/결과/receipt 및 G11 oracle는 변경하지 않는다.

기본 feature game_core --all-targets --no-fail-fast를 실행한다. G11 환경변수는 제거해 실제 기본경로를 검사한다. 샌드박스에서 이미 EPERM이 확인된 Unix socket1은 별도 허용 환경에서 현재 빌드 binary로 실행한다. 이는 성공으로 간주하는 skip이 아니다. 기존6ignore는 실제 cache/관리행위 전제를 가진 별도 U11 검사이며 새로 ignore하지 않는다. --doc는 별도실행한다. all-features 전체 실행과 서버/Unity/성능SLO를 뜻하지 않는다.

예상 빌드1–3분+전체실행4–5분(직전lib233초),60초 runner관측.10분 초과/새실패는 진행·원문 진단. 실행입력고정·소유세션종료후만수정. 결과를 한 실행의 실제 수로 집계하고 socket별도결과·ignore·doc시험0을 구분한다.

완료: full001 exit0,2168통과/기존ignore6/socket1환경분리. socket002 현재binary 허용환경1통과. doc003 exit0/문서시험0. 총단위2097+통합72=2169통과/실패0. 최종입력해시동일. 기본G11환경변수미지정4개모두PASS.
