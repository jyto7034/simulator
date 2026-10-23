import { renderToString } from 'react-dom/server';
import { StaticRouter } from 'react-router';
import { App } from './App';
import type { BootData } from './types';

export function render(path:string,data:BootData){
  const base=data.meta.config.basePath.replace(/\/$/,'');
  return renderToString(<StaticRouter location={base+path} basename={data.meta.config.basePath}><App data={data}/></StaticRouter>);
}
