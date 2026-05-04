import { useEffect } from "react";
import { Row, Col, Card, Button, Tag, Typography, theme, Space, Alert, Select } from "antd";
import {
  PlayCircleOutlined,
  PauseCircleOutlined,
  ReloadOutlined,
  ThunderboltOutlined,
  ClockCircleOutlined,
  DatabaseOutlined,
  ApiOutlined,
  SafetyOutlined,
  GlobalOutlined,
} from "@ant-design/icons";
import { useProxyStore } from "../store/useProxyStore";
import { useAuditStore } from "../store/useAuditStore";
import { useUpstreamStore } from "../store/useUpstreamStore";
import StatCard from "../components/StatCard";
import EventFeed from "../components/EventFeed";
import RuleHitChart from "../components/RuleHitChart";

export default function Dashboard() {
  const { token } = theme.useToken();
  const status = useProxyStore((s) => s.status);
  const loading = useProxyStore((s) => s.loading);
  const error = useProxyStore((s) => s.error);
  const recentEvents = useProxyStore((s) => s.recentEvents);
  const start = useProxyStore((s) => s.startProxy);
  const stop = useProxyStore((s) => s.stopProxy);
  const fetchStatus = useProxyStore((s) => s.fetchStatus);
  const stats = useAuditStore((s) => s.stats);
  const fetchStats = useAuditStore((s) => s.fetchStats);
  const upstreams = useUpstreamStore((s) => s.upstreams);
  const fetchUpstreams = useUpstreamStore((s) => s.fetchUpstreams);
  const setDefaultUpstream = useUpstreamStore((s) => s.setDefaultUpstream);

  useEffect(() => {
    fetchStatus();
    fetchStats();
    fetchUpstreams();
    const interval = setInterval(() => {
      fetchStatus();
      fetchStats();
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  const isRunning = status?.status === "running";
  const defaultUpstream = upstreams.find((u) => u.default);
  const proxyUrl = `http://127.0.0.1:${status?.port || 19000}`;

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const formatUptime = (secs: number) => {
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    if (h > 0) return `${h}h ${m}m`;
    return `${m}m`;
  };

  const cardStyle = {
    borderRadius: 12,
    border: `1px solid ${token.colorBorderSecondary}`,
  };

  return (
    <div style={{ height: "100%", overflow: "auto" }}>
      {error && (
        <Alert
          type="error"
          showIcon
          message="代理操作失败"
          description={error}
          closable
          onClose={() => useProxyStore.setState({ error: null })}
          style={{ marginBottom: 16, borderRadius: 8 }}
        />
      )}

      {/* 代理信息卡片 */}
      <Card size="small" style={{ ...cardStyle, marginBottom: 24 }}>
        {/* 第一行：状态 + 地址 + 操作按钮 */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            flexWrap: "wrap",
            gap: 12,
          }}
        >
          <Space size={16} wrap>
            <Space size={8}>
              <span
                style={{
                  width: 10,
                  height: 10,
                  borderRadius: "50%",
                  background: isRunning ? "#22c55e" : "#9ca3af",
                  display: "inline-block",
                }}
              />
              <Typography.Text strong style={{ fontSize: 15 }}>
                {isRunning ? "代理运行中" : "代理已停止"}
              </Typography.Text>
            </Space>

            <Tag icon={<GlobalOutlined />} color={isRunning ? "green" : "default"}>
              {proxyUrl}
            </Tag>

            {isRunning && (
              <>
                <Tag icon={<ClockCircleOutlined />}>
                  运行 {formatUptime(status?.uptimeSecs || 0)}
                </Tag>
                <Tag icon={<SafetyOutlined />}>
                  {status?.rulesCount ?? 0} 条规则
                </Tag>
                <Tag icon={<DatabaseOutlined />}>
                  存储 {status?.storageEnabled ? "开" : "关"}
                </Tag>
              </>
            )}
          </Space>

          <Space>
            <Button
              icon={<ReloadOutlined />}
              size="small"
              onClick={() => { fetchStatus(); fetchStats(); }}
            />
            {isRunning ? (
              <Button
                icon={<PauseCircleOutlined />}
                size="small"
                danger
                onClick={stop}
                loading={loading}
              >
                停止
              </Button>
            ) : (
              <Button
                type="primary"
                icon={<PlayCircleOutlined />}
                size="small"
                onClick={start}
                loading={loading}
              >
                启动代理
              </Button>
            )}
          </Space>
        </div>

        {/* 第二行：对接模型选择器 */}
        <div
          style={{
            marginTop: 16,
            paddingTop: 16,
            borderTop: `1px solid ${token.colorBorderSecondary}`,
            display: "flex",
            alignItems: "center",
            gap: 12,
            flexWrap: "wrap",
          }}
        >
          <Space size={8}>
            <ApiOutlined style={{ color: token.colorPrimary }} />
            <Typography.Text style={{ fontSize: 13 }}>对接模型</Typography.Text>
          </Space>
          <Select
            style={{ minWidth: 280 }}
            placeholder="选择默认上游 LLM"
            value={defaultUpstream?.name || undefined}
            onChange={async (name) => {
              await setDefaultUpstream(name);
              fetchUpstreams();
            }}
            options={upstreams.map((u) => ({
              value: u.name,
              label: `${u.name} — ${u.url}`,
            }))}
            notFoundContent={
              <Typography.Text type="secondary" style={{ padding: 8, display: "block" }}>
                暂无上游，请前往「大模型接入」添加
              </Typography.Text>
            }
          />
          {defaultUpstream && (
            <Tag color="blue" style={{ fontSize: 12 }}>
              {defaultUpstream.url}
              {defaultUpstream.models.length > 0 &&
                ` (${defaultUpstream.models.join(", ")})`}
            </Tag>
          )}
          <Typography.Text type="secondary" style={{ fontSize: 11 }}>
            {isRunning ? "切换后需重启代理生效" : "启动代理前选择目标模型"}
          </Typography.Text>
        </div>
      </Card>

      {/* 统计卡片 */}
      <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
        <Col xs={24} sm={12} lg={6}>
          <StatCard
            title="今日检测"
            value={stats?.todayCount ?? 0}
            icon={<ThunderboltOutlined />}
            color="#3b82f6"
          />
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <StatCard
            title="本周检测"
            value={stats?.weekCount ?? 0}
            icon={<ThunderboltOutlined />}
            color="#8b5cf6"
          />
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <StatCard
            title="总计检测"
            value={stats?.totalCount ?? 0}
            icon={<ThunderboltOutlined />}
            color="#f59e0b"
          />
        </Col>
        <Col xs={24} sm={12} lg={6}>
          <StatCard
            title="数据库大小"
            value={formatBytes(stats?.dbSizeBytes ?? 0)}
            icon={<DatabaseOutlined />}
            color="#22c55e"
          />
        </Col>
      </Row>

      {/* 图表 + 实时事件 */}
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={12}>
          <Card
            title="规则命中分布"
            size="small"
            style={cardStyle}
          >
            <RuleHitChart data={stats?.ruleDistribution ?? []} />
          </Card>
        </Col>
        <Col xs={24} lg={12}>
          <Card
            title="最近事件"
            size="small"
            style={{ ...cardStyle, maxHeight: 380, overflow: "auto" }}
          >
            <EventFeed events={recentEvents} />
          </Card>
        </Col>
      </Row>
    </div>
  );
}
