import { createContext, useContext, useEffect, useState } from 'react';
import type { BootData, PageMeta, SearchRecord, WikiPage } from './types';

export const DataContext = createContext<BootData | null>(null);
export function useData() {
  const data = useContext(DataContext);
  if (!data) throw new Error('위키 데이터가 없습니다.');
  return data;
}
export function pathFor(id: string) { return '/wiki/' + id + '/'; }
export function categoryPages(pages: PageMeta[], category: string) {
  return pages.filter(p => p.category === category && (!p.index || p.id === 'content/tags/README') &&
    (category !== 'abnormalities' || p.part === 'README' || p.id.startsWith('content/tools/')));
}
export function usePage(id: string) {
  const { meta, page: initial } = useData();
  const [page, setPage] = useState<WikiPage | undefined>(initial?.id === id ? initial : undefined);
  const [error, setError] = useState('');
  useEffect(() => {
    setError('');
    if (page?.id === id) return;
    const controller = new AbortController();
    fetch(meta.config.basePath + 'generated/pages/' + id + '.json', { signal: controller.signal })
      .then(r => { if (!r.ok) throw new Error('문서를 불러오지 못했습니다.'); return r.json() as Promise<WikiPage>; })
      .then(setPage).catch(e => { if (e.name !== 'AbortError') setError(e.message); });
    return () => controller.abort();
  }, [id, page?.id, meta.config.basePath]);
  return { page: page?.id === id ? page : undefined, error };
}
const searchCaches = new Map<string, Promise<SearchRecord[]>>();
export function loadSearch(base: string, archive = false) {
  const url=base+'generated/'+(archive?'archive-search.json':'search.json');
  if (!searchCaches.has(url)) searchCaches.set(url, fetch(url).then(r => {
    if (!r.ok) throw new Error('검색 자료를 불러오지 못했습니다.');
    return r.json() as Promise<SearchRecord[]>;
  }).catch(e => { searchCaches.delete(url); throw e; }));
  return searchCaches.get(url)!;
}
export const partNames: Record<string, string> = {
  README: '개요', encounter: '조우', fragment: '스킬 파편', equipment: '장비', synergies: '태그·유물', reference: '원작 참고', review: '검토 기록',
};
