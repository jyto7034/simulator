import { useEffect, useRef, useState } from 'react';
import { Moon, Sun } from 'lucide-react';

export function ThemeToggle() {
  const [dark, setDark] = useState(false);
  const preference = useRef<string | null>(null);
  useEffect(() => {
    const system = window.matchMedia('(prefers-color-scheme: dark)');
    try { preference.current = localStorage.getItem('ark-wiki-theme'); } catch { /* Storage may be disabled. */ }
    function sync() {
      const next = preference.current === 'dark' || (preference.current !== 'light' && system.matches);
      document.documentElement.dataset.theme = next ? 'dark' : 'light';
      setDark(next);
    }
    function storage(event: StorageEvent) {
      if (event.key === 'ark-wiki-theme' || event.key === null) {
        preference.current = event.newValue;
        sync();
      }
    }
    sync();
    system.addEventListener('change', sync);
    window.addEventListener('storage', storage);
    return () => { system.removeEventListener('change', sync); window.removeEventListener('storage', storage); };
  }, []);
  function toggle() {
    const next = !dark;
    preference.current = next ? 'dark' : 'light';
    document.documentElement.dataset.theme = preference.current;
    setDark(next);
    try { localStorage.setItem('ark-wiki-theme', preference.current); } catch { /* Keep the current session usable. */ }
  }
  return <button className="icon-button" type="button" onClick={toggle} aria-label="다크 테마" aria-pressed={dark} title="밝은 테마 / 어두운 테마 전환">
    <Moon className="theme-moon size-4" aria-hidden="true" /><Sun className="theme-sun size-4" aria-hidden="true" />
  </button>;
}
