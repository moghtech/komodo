import { createContext, useContext, useEffect, useState } from "react";
import { Button } from "@ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@ui/dropdown-menu";
import { CheckCircle, Moon, Sun } from "lucide-react";

type Theme = "dark" | "light" | "system";

type ThemeProviderProps = {
  children: React.ReactNode;
  defaultTheme?: Theme;
  storageKey?: string;
};

type ThemeProviderState = {
  theme: Theme;
  currentTheme: Exclude<Theme, "system">;
  setTheme: (theme: Theme) => void;
};

const initialState: ThemeProviderState = {
  theme: "system",
  currentTheme: "dark",
  setTheme: () => null,
};

const ThemeProviderContext = createContext<ThemeProviderState>(initialState);

const systemTheme = () =>
  window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";

export function ThemeProvider({
  children,
  defaultTheme = "system",
  storageKey = "vite-ui-theme",
  ...props
}: ThemeProviderProps) {
  const [theme, setTheme] = useState<Theme>(
    () => (localStorage.getItem(storageKey) as Theme) || defaultTheme
  );
  // Tracks the current theme
  //   - if theme is light or dark, equal to theme.
  //   - if theme is system, tracks current theme with pool loop
  const [currentTheme, setCurrentTheme] = useState<Exclude<Theme, "system">>(
    theme === "system" ? systemTheme() : theme
  );

  useEffect(() => {
    if (theme === "system") {
      setCurrentTheme(systemTheme());
      // For 'system' theme, need to poll
      // matchMedia for update to theme.
      const interval = setInterval(() => {
        setCurrentTheme(systemTheme());
      }, 5_000);
      return () => clearInterval(interval);
    } else {
      setCurrentTheme(theme);
    }
  }, [theme]);

  useEffect(() => {
    const root = window.document.documentElement;
    root.classList.add(currentTheme);
    return () => root.classList.remove(currentTheme);
  }, [currentTheme]);

  const value = {
    theme,
    currentTheme,
    setTheme: (theme: Theme) => {
      localStorage.setItem(storageKey, theme);
      setTheme(theme);
    },
  };

  return (
    <ThemeProviderContext.Provider {...props} value={value}>
      {children}
    </ThemeProviderContext.Provider>
  );
}

export const useTheme = () => {
  const context = useContext(ThemeProviderContext);

  if (context === undefined)
    throw new Error("useTheme must be used within a ThemeProvider");

  return context;
};

export function ThemeToggle() {
  const { theme, setTheme } = useTheme();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon">
          <Sun className="w-4 h-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
          <Moon className="absolute w-4 h-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
          <span className="sr-only">Toggle theme</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" sideOffset={20}>
        <DropdownMenuItem
          className="cursor-pointer flex items-center justify-between"
          onClick={() => setTheme("light")}
        >
          Light
          {theme === "light" && <CheckCircle className="w-3 h-3" />}
        </DropdownMenuItem>
        <DropdownMenuItem
          className="cursor-pointer flex items-center justify-between"
          onClick={() => setTheme("dark")}
        >
          Dark
          {theme === "dark" && <CheckCircle className="w-3 h-3" />}
        </DropdownMenuItem>
        <DropdownMenuItem
          className="cursor-pointer flex items-center justify-between"
          onClick={() => setTheme("system")}
        >
          System
          {theme === "system" && <CheckCircle className="w-3 h-3" />}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
