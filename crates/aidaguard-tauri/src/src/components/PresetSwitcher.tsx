import { Segmented, Space } from "antd";
import { useTranslation } from "react-i18next";
import { useThemeStore, PRESETS, type ThemePreset } from "../store/useThemeStore";

const SWATCH_SIZE = 16;

function ColorSwatch({ color }: { color: string }) {
  return (
    <span
      style={{
        display: "inline-block",
        width: SWATCH_SIZE,
        height: SWATCH_SIZE,
        borderRadius: 4,
        background: color,
        verticalAlign: "middle",
      }}
    />
  );
}

export default function PresetSwitcher() {
  const preset = useThemeStore((s) => s.preset);
  const setPreset = useThemeStore((s) => s.setPreset);
  const { t } = useTranslation();

  const handleChange = (val: string) => {
    setPreset(val as ThemePreset);
  };

  return (
    <Segmented
      value={preset}
      onChange={handleChange}
      options={[
        {
          value: "emerald",
          label: (
            <Space size={6}>
              <ColorSwatch color={PRESETS.emerald.colorPrimary} />
              <span>{t("Emerald Green")}</span>
            </Space>
          ),
        },
        {
          value: "indigo",
          label: (
            <Space size={6}>
              <ColorSwatch color={PRESETS.indigo.colorPrimary} />
              <span>{t("Indigo Purple")}</span>
            </Space>
          ),
        },
      ]}
    />
  );
}
