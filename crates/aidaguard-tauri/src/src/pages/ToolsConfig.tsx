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
import { useToolsStore } from "../store/useToolsStore";
import { useProxyStore } from "../store/useProxyStore";

export default function ToolsConfig() {
  const { token } = theme.useToken();
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
      message.success("配置已应用");
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleRestore = async (toolId: string) => {
    try {
      await restore(toolId);
      message.success("配置已恢复");
    } catch (e) {
      message.error(String(e));
    }
  };

  const handleRestoreAll = async () => {
    try {
      await restoreAll();
      message.success("全部配置已恢复");
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
          message="代理未启动"
          description="启动代理后才能在下方预览配置效果。配置将把所有请求重定向到本地代理地址。"
          style={{ marginBottom: 16, borderRadius: 8 }}
        />
      )}
      {error && (
        <Alert
          type="error"
          showIcon
          message="操作失败"
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
              已检测 {installedCount}/8 个工具
            </Typography.Text>
            {configuredCount > 0 && (
              <Tag color="blue">{configuredCount} 个已配置</Tag>
            )}
            <Tag color="geekblue">
              <ApiOutlined /> {proxyUrl}
            </Tag>
          </Space>
          <Space>
            <Button onClick={fetchTools} loading={loading}>
              重新检测
            </Button>
            <Popconfirm
              title="确定恢复所有工具的原始配置？"
              onConfirm={handleRestoreAll}
              okText="确定"
              cancelText="取消"
            >
              <Button icon={<UndoOutlined />} danger>
                全部恢复
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
                tool.installed && tool.configured ? (
                  <Button
                    key="restore"
                    size="small"
                    icon={<UndoOutlined />}
                    loading={applying === tool.toolId}
                    onClick={() => handleRestore(tool.toolId)}
                  >
                    恢复
                  </Button>
                ) : tool.installed ? (
                  <Button
                    key="apply"
                    type="primary"
                    size="small"
                    icon={<SettingOutlined />}
                    loading={applying === tool.toolId}
                    onClick={() => handleApply(tool.toolId)}
                  >
                    配置
                  </Button>
                ) : (
                  <Typography.Text
                    key="na"
                    type="secondary"
                    style={{ fontSize: 12 }}
                  >
                    未安装
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
                        已安装
                      </Tag>
                    ) : (
                      <Tag color="default" style={{ fontSize: 11 }}>
                        未安装
                      </Tag>
                    )}
                    {tool.configured && (
                      <Tag color="blue" style={{ fontSize: 11 }}>
                        已配置
                      </Tag>
                    )}
                  </Space>
                }
                description={
                  <div style={{ fontSize: 12 }}>
                    <div style={{ color: token.colorTextSecondary, marginBottom: 4 }}>
                      配置文件：<code>{tool.configPath}</code>
                    </div>
                    {tool.currentEndpoint && (
                      <div style={{ color: token.colorTextSecondary }}>
                        当前端点：{tool.currentEndpoint}
                        {tool.previewEndpoint && (
                          <span style={{ color: token.colorPrimary }}>
                            {" "}
                            → {tool.previewEndpoint}
                          </span>
                        )}
                      </div>
                    )}
                    {!tool.installed && (
                      <div style={{ color: token.colorTextQuaternary }}>
                        安装此工具后即可一键配置
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
