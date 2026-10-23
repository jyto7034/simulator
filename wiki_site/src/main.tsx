import { createRoot, hydrateRoot } from 'react-dom/client';
import { BrowserRouter } from 'react-router';
import { App } from './App';
import type { BootData } from './types';
import 'pretendard/dist/web/variable/pretendardvariable-dynamic-subset.css';
import '@fontsource/noto-serif-kr/700.css';
import './styles.css';

async function start(){
  const embedded=document.getElementById('wiki-boot'),root=document.getElementById('root')!;
  const data:BootData=embedded?JSON.parse(embedded.textContent!):{meta:await fetch(import.meta.env.BASE_URL+'generated/meta.json').then(r=>{if(!r.ok)throw new Error('위키 데이터를 생성한 뒤 다시 실행해 주세요.');return r.json();})};
  const app=<BrowserRouter basename={data.meta.config.basePath}><App data={data}/></BrowserRouter>;
  if(data.prerendered)hydrateRoot(root,app);else createRoot(root).render(app);
}
start().catch(error=>{document.getElementById('root')!.textContent='위키를 열지 못했습니다: '+error.message;});
