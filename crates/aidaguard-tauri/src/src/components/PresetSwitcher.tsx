import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useTranslation } from "react-i18next";
import { useThemeStore, type ThemePreset } from "../store/useThemeStore";

const PRESET_COLORS: Record<ThemePreset, string> = {
  emerald: "#10b981",
  indigo: "#6366f1",
};

export default function PresetSwitcher() {
  const preset = useThemeStore((s) => s.preset);
  const setPreset = useThemeStore((s) => s.setPreset);
  const { t } = useTranslation();

  return (
    <ToggleGroup
      type="single"
      value={preset}
      onValueChange={(v) => {
        if (v) setPreset(v as ThemePreset);
      }}
    >
      <ToggleGroupItem value="emerald" aria-label={t("Emerald Green")}>
        <span className="inline-block w-4 h-4 rounded mr-1.5" style={{ background: PRESET_COLORS.emerald }} />
        <span className="text-sm">{t("Emerald Green")}</span>
      </ToggleGroupItem>
      <ToggleGroupItem value="indigo" aria-label={t("Indigo Purple")}>
        <span className="inline-block w-4 h-4 rounded mr-1.5" style={{ background: PRESET_COLORS.indigo }} />
        <span className="text-sm">{t("Indigo Purple")}</span>
      </ToggleGroupItem>
    </ToggleGroup>
  );
}
