from pathlib import Path
import json, hashlib
ROOT=Path(__file__).resolve().parents[1]
P=ROOT/'evidence/source_ledger.json'
def add(path, ranges, anchors=(), kind='source'):
    data=json.loads(P.read_text()) if P.exists() else {'scope':'읽은 실제 본문 구간. 앵커 해시는 전체 파일 또는 dirty snapshot 해시가 아니다.','sources':[]}
    row=next((r for r in data['sources'] if r['path']==path),None)
    if row is None:
        row={'id':f'S{len(data["sources"])+1:03}','path':path,'kind':kind,'ranges':[],'anchors':[],'file_sha256':None}
        data['sources'].append(row)
    for r in ranges:
        if list(r) not in row['ranges']:row['ranges'].append(list(r))
    for a in anchors:
        if not any(x['text']==a for x in row['anchors']):row['anchors'].append({'text':a,'sha256_utf8':hashlib.sha256(a.encode()).hexdigest()})
    P.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
    return row['id']
if __name__=='__main__':
    print(len(json.loads(P.read_text())['sources']) if P.exists() else 0)
