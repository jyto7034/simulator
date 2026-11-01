import { useEffect } from 'react';
import { Navigate, Route, Routes, useLocation } from 'react-router';
import { DataContext } from './data';
import { Layout } from './components/Layout';
import { HomePage } from './pages/Home';
import { CategoryPage } from './pages/Category';
import { DocumentPage } from './pages/Document';
import { ScenesPage, ScenePage } from './pages/Scenes';
import { SearchPage } from './pages/Search';
import { AboutPage, NotFound, ScreensPage } from './pages/Reference';
import type { BootData } from './types';

export function App({data}:{data:BootData}){
  const location=useLocation();
  useEffect(()=>{
    const p=data.meta.pages.find(p=>p.path===location.pathname),scene=data.meta.scenes.find(s=>'/scenes/'+s.id+'/'===location.pathname),category=data.meta.categories.find(c=>'/categories/'+c.id+'/'===location.pathname);
    const title=p?.title||scene?.title||category?.title||({'/wiki/archive/':'아카이브','/wiki/archive/search/':'아카이브 검색','/search/':'통합 검색','/scenes/':'장면·플레이어 행동','/screens/':'전체 화면 제작 목록','/about/':'출처와 갱신'} as Record<string,string>)[location.pathname];
    document.title=title?title+' | ARK 게임 위키':data.meta.title;
  },[location.pathname,data.meta]);
  return <DataContext.Provider value={data}><Routes><Route element={<Layout/>}>
    <Route index element={<HomePage/>}/>
    <Route path="categories/tags" element={<Navigate replace to="/wiki/content/tags/README/"/>}/>
    <Route path="categories/:id" element={<CategoryPage key={location.pathname}/>}/>
    <Route path="wiki/archive" element={<CategoryPage archive/>}/>
    <Route path="wiki/archive/search" element={<SearchPage archive/>}/>
    <Route path="wiki/*" element={<DocumentPage key={location.pathname}/>}/><Route path="scenes" element={<ScenesPage/>}/><Route path="scenes/:id" element={<ScenePage key={location.pathname}/>}/>
    <Route path="search" element={<SearchPage/>}/><Route path="screens" element={<ScreensPage/>}/><Route path="about" element={<AboutPage/>}/><Route path="*" element={<NotFound/>}/>
  </Route></Routes></DataContext.Provider>;
}
