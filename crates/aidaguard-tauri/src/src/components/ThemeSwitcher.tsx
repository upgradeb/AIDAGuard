import { Segmented } from "antd";
import { useThemeStore } from "../store/useThemeStore";

export default function ThemeSwitcher() {
  const theme = useThemeStore((s) => s.theme);
  const setTheme = useThemeStore((s) => s.setTheme);

  return (
    <Segmented
      value={theme}
      onChange={(val) => setTheme(val as "light" | "dark" | "system")}
      options={[
        { value: "light", label: "浅色" },
        { value: "dark", label: "深色" },
        { value: "system", label: "跟随系统" },
      ]}
    />
  );
}
