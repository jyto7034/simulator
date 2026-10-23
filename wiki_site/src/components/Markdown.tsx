import { useEffect, useRef } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { useData } from '../data';

// HTML is produced by the verified build-time Markdown renderer, not user input.
export function Markdown({ html }: { html: string }) {
  const ref=useRef<HTMLElement>(null),location=useLocation(),navigate=useNavigate(),{meta}=useData();
  useEffect(()=>{
    if(!location.hash)return;
    const id=decodeURIComponent(location.hash.slice(1));
    requestAnimationFrame(()=>{
      const element=document.getElementById(id);
      let parent=element?.parentElement;
      while(parent){if(parent instanceof HTMLDetailsElement)parent.open=true;parent=parent.parentElement;}
      element?.scrollIntoView({block:'start'});
    });
  },[html,location.hash]);
  return <article ref={ref} className="wiki-prose" id="article-body" dangerouslySetInnerHTML={{__html:html}} onClick={e=>{
    if(e.button!==0||e.metaKey||e.ctrlKey||e.shiftKey||e.altKey)return;
    const anchor=(e.target as Element).closest('a');if(!anchor||anchor.target||anchor.hasAttribute('download'))return;
    const u=new URL(anchor.href),base=meta.config.basePath.replace(/\/$/,'');
    if(u.origin===window.location.origin&&u.pathname.startsWith(base+'/')&&!u.pathname.includes('/generated/')){
      e.preventDefault();navigate(u.pathname.slice(base.length)+u.search+u.hash);
    }
  }} />;
}
