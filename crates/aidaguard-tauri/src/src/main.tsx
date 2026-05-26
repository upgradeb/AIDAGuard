import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { ConfigProvider, theme as antTheme } from "antd";
import zhCN from "antd/locale/zh_CN";
import enUS from "antd/locale/en_US";
import { I18nextProvider, useTranslation } from "react-i18next";
import i18n from "./i18n";
import App from "./App";
import { useThemeStore } from "./store/useThemeStore";
import "./App.css";

const antdLocales: Record<string, typeof zhCN> = { zh: zhCN, en: enUS };

function Root() {
  const resolved = useThemeStore((s) => s.resolved);
  const presetColors = useThemeStore((s) => s.presetColors);
  const { i18n: i18nInst } = useTranslation();
  const locale = antdLocales[i18nInst.language] || zhCN;

  return (
    <ConfigProvider
      locale={locale}
      theme={{
        algorithm:
          resolved === "dark" ? antTheme.darkAlgorithm : antTheme.defaultAlgorithm,
        token: {
          colorPrimary: presetColors.colorPrimary,
          borderRadius: presetColors.borderRadius,
        },
      }}
    >
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </ConfigProvider>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <I18nextProvider i18n={i18n}>
      <Root />
    </I18nextProvider>
  </React.StrictMode>
);
