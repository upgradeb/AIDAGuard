import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { Sun, Moon, Monitor } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useThemeStore } from "../store/useThemeStore";

export default function ThemeSwitcher() {
  const theme = useThemeStore((s) => s.theme);
  const setTheme = useThemeStore((s) => s.setTheme);
  const { t } = useTranslation();

  return (
    <ToggleGroup
      type="single"
      value={theme}
      onValueChange={(v) => {
        if (v) setTheme(v as "light" | "dark" | "system");
      }}
    >
      <ToggleGroupItem value="light" aria-label={t("Light")}>
        <Sun className="h-4 w-4" />
      </ToggleGroupItem>
      <ToggleGroupItem value="dark" aria-label={t("Dark")}>
        <Moon className="h-4 w-4" />
      </ToggleGroupItem>
      <ToggleGroupItem value="system" aria-label={t("System")}>
        <Monitor className="h-4 w-4" />
      </ToggleGroupItem>
    </ToggleGroup>
  );
}
