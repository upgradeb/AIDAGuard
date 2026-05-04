import { useEffect } from "react";
import {
  Card,
  Button,
  Tag,
  Typography,
  Space,
  List,
  message,
  theme,
  Alert,
  Popconfirm,
} from "antd";
import {
  SettingOutlined,
  UndoOutlined,
  CheckCircleOutlined,
  WarningOutlined,
  ApiOutlined,
} from "@ant-design/icons";
import { useTranslation } from "react-i18next";
import { useToolsStore } from "../store/useToolsStore";
import { useProxyStore } from "../store/useProxyStore";

export default function ToolsConfig() {
  const { token } = theme.useToken();
  const { t } = useTranslation();
  const tools = useToolsStore((s) => s.tools);
  const loading = useToolsStore((s) => s.loading);
  const applying = useToolsStore((s) => s.applying);
  const error = useToolsStore((s) => s.error);
  const fetchTools = useToolsStore((s) => s.fetchTools);
  const apply = useToolsStore((s) => s.apply);
  const restore = useToolsStore((s) => s.restore);
  const restoreAll = useToolsStore((s) => s.restoreAll);
  const proxyStatus = useProxyStore((s) => s.status);

  useEffect(() => {
    fetchTools();
  }, []);

  const isRunning = proxyStatus?.status === "running";
  const proxyUrl = `http://127.0.0.1:${proxyStatus?.port || 19000}`;

  const handleApply = async (toolId: string) => {
    try {
      await apply(toolId);
      message.success(t("Configuration Applied"));
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleRestore = async (toolId: string) => {
    try {
      await restore(toolId);
      message.success(t("Configuration Restored"));
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleRestoreAll = async () => {
    try {
      await restoreAll();
      message.success(t("All Configurations Restored"));
    } catch (e) {
      message.error(String(e));
    }
  };

  const configuredCount = tools.filter((t) => t.configured).length;
  const installedCount = tools.filter((t) => t.installed).length;

  return (
    <div style={{ height: "100%", overflow: "auto" }}>
      {/* 提示 */}
      {!isRunning && (
        <Alert
          type="warning"
          showIcon
          message={t("Proxy Not Started")}
          description={t("Start the proxy to preview configuration effects below. Configuration will redirect all requests to the local proxy address.")}
          style={{ marginBottom: 16, borderRadius: 8 }}
        />
      )}
      {error && (
        <Alert
          type="error"
          showIcon
          message={t("Operation Failed")}
          description={error}
          closable
          style={{ marginBottom: 16, borderRadius: 8 }}
        />
      )}

      {/* 概览信息 */}
      <Card
        size="small"
        style={{
          borderRadius: 12,
          border: `1px solid ${token.colorBorderSecondary}`,
          marginBottom: 16,
        }}
      >
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            flexWrap: "wrap",
            gap: 8,
          }}
        >
          <Space size={12}>
            <Typography.Text strong>
              {t("{{installedCount}}/8 Tools Detected", { installedCount })}
            </Typography.Text>
            {configuredCount > 0 && (
              <Tag color="blue">{t("{{configuredCount}} Configured", { configuredCount })}</Tag>
            )}
            <Tag color="geekblue">
              <ApiOutlined /> {proxyUrl}
            </Tag>
          </Space>
          <Space>
            <Button onClick={fetchTools} loading={loading}>
              {t("Rescan")}
            </Button>
            <Popconfirm
              title={t("Restore all tools to their original configuration?")}
              onConfirm={handleRestoreAll}
              okText={t("OK")}
              cancelText={t("Cancel")}
            >
              <Button icon={<UndoOutlined />} danger>
                {t("Restore All")}
              </Button>
            </Popconfirm>
          </Space>
        </div>
      </Card>

      {/* 工具列表 */}
      <Card
        size="small"
        style={{
          borderRadius: 12,
          border: `1px solid ${token.colorBorderSecondary}`,
        }}
        styles={{ body: { padding: 0 } }}
      >
        <List
          dataSource={tools}
          loading={loading}
          renderItem={(tool) => (
            <List.Item
              style={{ padding: "14px 20px" }}
              actions={[
                tool.installed ? (
                  <Button
                    key="apply"
                    type="primary"
                    size="small"
                    icon={<SettingOutlined />}
                    loading={applying === tool.toolId}
                    onClick={() => handleApply(tool.toolId)}
                  >
                    {t("Configure")}
                  </Button>
                ) : null,
                tool.installed ? (
                  <Popconfirm
                    key="restore"
                    title={t("Restore original configuration?")}
                    onConfirm={() => handleRestore(tool.toolId)}
                    okText={t("OK")}
                    cancelText={t("Cancel")}
                  >
                    <Button
                      size="small"
                      icon={<UndoOutlined />}
                      loading={applying === tool.toolId}
                    >
                      {t("Restore")}
                    </Button>
                  </Popconfirm>
                ) : (
                  <Typography.Text
                    key="na"
                    type="secondary"
                    style={{ fontSize: 12 }}
                  >
                    {t("Not Installed")}
                  </Typography.Text>
                ),
              ]}
            >
              <List.Item.Meta
                avatar={
                  tool.installed ? (
                    tool.configured ? (
                      <CheckCircleOutlined
                        style={{ fontSize: 20, color: token.colorSuccess }}
                      />
                    ) : (
                      <WarningOutlined
                        style={{ fontSize: 20, color: token.colorWarning }}
                      />
                    )
                  ) : (
                    <span
                      style={{
                        fontSize: 20,
                        color: token.colorTextQuaternary,
                      }}
                    >
                      —
                    </span>
                  )
                }
                title={
                  <Space size={8}>
                    <Typography.Text strong>{tool.toolName}</Typography.Text>
                    {tool.installed ? (
                      <Tag color="green" style={{ fontSize: 11 }}>
                        {t("Installed")}
                      </Tag>
                    ) : (
                      <Tag color="default" style={{ fontSize: 11 }}>
                        {t("Not Installed")}
                      </Tag>
                    )}
                    {tool.configured && (
                      <Tag color="blue" style={{ fontSize: 11 }}>
                        {t("Configured")}
                      </Tag>
                    )}
                  </Space>
                }
                description={
                  <div style={{ fontSize: 12 }}>
                    <div style={{ color: token.colorTextSecondary, marginBottom: 4 }}>
                      {t("Config File: ")}<code>{tool.configPath}</code>
                    </div>
                    {tool.configured ? (
                      <div style={{ color: token.colorPrimary, fontWeight: 500, marginTop: 2 }}>
                        {t("Proxied — Requests will be scanned by Aidaguard for sensitive data")}
                      </div>
                    ) : tool.installed ? (
                      <div style={{ color: token.colorWarning, fontWeight: 500, marginTop: 2 }}>
                        {t("Not Proxied — Click \"Configure\" to route through local proxy")}
                      </div>
                    ) : (
                      <div style={{ color: token.colorTextQuaternary }}>
                        {t("Install this tool to enable one-click configuration")}
                      </div>
                    )}
                  </div>
                }
              />
            </List.Item>
          )}
        />
      </Card>
    </div>
  );
}
