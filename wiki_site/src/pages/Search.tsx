import { useDeferredValue, useEffect, useState } from 'react';
import { Link, useSearchParams } from 'react-router';
import { Search } from 'lucide-react';
import { Breadcrumb, Empty } from '../components/common';
import { Collection, documentEntry } from '../components/Collection';
import { loadSearch, useData } from '../data';
import type { SearchRecord } from '../types';

export function SearchPage({archive=false}:{archive?:boolean}){
  const {meta}=useData(),[params,setParams]=useSearchParams(),[index,setIndex]=useState<SearchRecord[]>(),[error,setError]=useState('');
  const title=archive?'아카이브 검색':'통합 검색';
  const q=params.get('q')||'',query=useDeferredValue(q),category=params.get('category')||'';
  useEffect(()=>{let active=true;setIndex(undefined);loadSearch(meta.config.basePath,archive).then(data=>{if(active)setIndex(data);}).catch(e=>{if(active)setError(e.message);});return()=>{active=false;};},[meta.config.basePath,archive]);
  function update(key:string,value:string){const next=new URLSearchParams(params);if(value)next.set(key,value);else next.delete(key);setParams(next,{replace:true});}
  const terms=query.trim().toLowerCase().split(/\s+/).filter(Boolean),matches=(s:string)=>terms.every(t=>s.toLowerCase().includes(t));
  const results=terms.length&&index?index.filter(p=>(archive?p.category==='history':p.category!=='history')&&(!category||p.category===category)&&matches(p.title+' '+p.text)).sort((a,b)=>Number(matches(b.title))-Number(matches(a.title))):[];
  const actions=!archive&&terms.length&&index&&(!category||category==='actions')?meta.scenes.flatMap(s=>s.actions.filter(a=>matches([s.title,a.title,a.input,a.result,a.note].join(' '))).map(a=>({a,s}))):[];
  const entries=[...actions.map(({s,a})=>({id:s.id+'/'+a.id,title:a.title,kind:s.title+' · '+meta.statuses[a.status],category:'ui',className:'action-result',summary:a.result,href:'/scenes/'+s.id+'/#action-'+a.id,facts:[{label:'입력',value:a.input}],status:a.note})),...results.map(r=>{const start=Math.max(0,r.text.toLowerCase().indexOf(terms[0])-30);return {...documentEntry(meta.pages.find(p=>p.id===r.id)!,meta,r.text.slice(start,start+200)+'…'),className:'document-result'};})];
  return <><Breadcrumb items={[{title}]}/><p className="eyebrow">SEARCH THE WIKI</p><h1 className="page-title mt-3">{title}</h1><p className="page-lead">{archive?'이전 합의와 검토 기록만 검색합니다.':'현재 합의 문서와 플레이어 행동을 검색합니다.'}</p>
    <div className="relative mb-4"><Search className="pointer-events-none absolute top-1/2 left-4 size-5 -translate-y-1/2 text-faint" aria-hidden="true" /><input type="search" className="field w-full py-3.5 pl-12 text-base" aria-label="검색어" value={q} onChange={e=>update('q',e.target.value)} placeholder="찾을 단어를 입력해 주세요"/></div>
    <div className="mb-5 flex flex-wrap items-center gap-4">{!archive&&<select className="field" aria-label="검색 분류" value={category} onChange={e=>update('category',e.target.value)}><option value="">모든 분류</option><option value="actions">플레이어 행동</option>{meta.categories.filter(c=>c.id!=='history').map(c=><option key={c.id} value={c.id}>{c.title}</option>)}</select>}<Link className="text-sm text-brand underline" to={(archive?'/search/':'/wiki/archive/search/')+'?q='+encodeURIComponent(q)}>{archive?'현재 위키에서 검색':'아카이브에서 검색'}</Link></div>
    <p className="mb-3 text-xs text-muted" role="status">{!index?'검색 자료를 불러오는 중입니다…':!terms.length?'검색어를 입력해 주세요.':`‘${query}’ 검색 결과 ${results.length+actions.length}개`}</p>
    {error&&<Empty>{error} <button className="text-brand underline" onClick={()=>window.location.reload()}>다시 시도</button></Empty>}
    {index&&terms.length>0&&<Collection entries={entries} scope={archive?"archive-search":"search"} label="검색 결과" />}
  </>;
}
