import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import config from './site.config.json' with { type: 'json' };

export default defineConfig(({ isSsrBuild }) => ({
  base: config.basePath,
  plugins: [react(), tailwindcss()],
  server: { host: '127.0.0.1', port: 4178, strictPort: true },
  preview: { host: '127.0.0.1', port: 4178, strictPort: true },
  build: { sourcemap: false, copyPublicDir: !isSsrBuild },
}));
