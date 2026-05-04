import { Segmented } from "antd";
import { useTranslation } from "react-i18next";
import { useThemeStore } from "../store/useThemeStore";

export default function ThemeSwitcher() {
  const theme = useThemeStore((s) => s.theme);
  const setTheme = useThemeStore((s) => s.setTheme);
  const { t } = useTranslation();

  return (
    <Segmented
      value={theme}
      onChange={(val) => setTheme(val as "light" | "dark" | "system")}
      options={[
        { value: "light", label: t("浅色") },
        { value: "dark", label: t("深色") },
        { value: "system", label: t("跟随系统") },
      ]}
    />
  );
}
