import { useEffect, type PropsWithChildren } from "react";

import { useUiStore } from "@/stores/ui-store";
import { TooltipProvider } from "@/components/ui/tooltip";

export function AppProviders({ children }: PropsWithChildren) {
  const theme = useUiStore((state) => state.theme);

  useEffect(() => {
    const preference = window.matchMedia("(prefers-color-scheme: dark)");
    const applyTheme = () => {
      const dark =
        theme === "dark" || (theme === "system" && preference.matches);
      document.documentElement.classList.toggle("dark", dark);
      document.documentElement.style.colorScheme = dark ? "dark" : "light";
    };

    applyTheme();
    preference.addEventListener("change", applyTheme);
    return () => preference.removeEventListener("change", applyTheme);
  }, [theme]);

  return <TooltipProvider delayDuration={300}>{children}</TooltipProvider>;
}
