import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

export type Theme = "light" | "dark" | "system";

export function isTheme(value: unknown): value is Theme {
  return value === "light" || value === "dark" || value === "system";
}

type UiState = {
  theme: Theme;
  setTheme: (theme: Theme) => void;
};

// Persist UI preferences only. Game documents and edit history belong elsewhere.
export const useUiStore = create<UiState>()(
  persist(
    (set) => ({
      theme: "system",
      setTheme: (theme) => set({ theme }),
    }),
    {
      name: "content-assembler-ui",
      version: 1,
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({ theme: state.theme }),
      merge: (persisted, current) => {
        const saved = persisted as { theme?: unknown } | undefined;
        return {
          ...current,
          theme: isTheme(saved?.theme) ? saved.theme : current.theme,
        };
      },
    },
  ),
);
