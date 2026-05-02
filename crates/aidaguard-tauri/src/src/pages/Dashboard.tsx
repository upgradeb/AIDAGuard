import { useEffect } from "react";
import { Row, Col, Card, Button, Tag, Typography, theme, Space, Alert } from "antd";
import {
  PlayCircleOutlined,
  PauseCircleOutlined,
  ReloadOutlined,
  ThunderboltOutlined,
  CloudServerOutlined,
  ClockCircleOutlined,
  DatabaseOutlined,
} from "@ant-design/icons";
import { useProxyStore } from "../store/useProxyStore";
import { useAuditStore } from "../store/useAuditStore";
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

  useEffect(() => {
    fetchStatus();
    fetchStats();
    const interval = setInterval(() => {
      fetchStatus();
      fetchStats();
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  const isRunning = status?.status === "running";

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

  return (
    <div>
      {/* 错误提示 */}
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

      {/* Status bar */}
      <Card
        size="small"
        style={{
          marginBottom: 24,
          borderRadius: 12,
          border: `1px solid ${token.colorBorderSecondary}`,
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
          }}
        >
          <Space size={16}>
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
              <Typography.Text strong>
                {isRunning ? "运行中" : "已停止"}
              </Typography.Text>
            </Space>

            {isRunning && (
              <>
                <Tag icon={<CloudServerOutlined />}>
                  端口: {status?.port}
                </Tag>
                <Tag icon={<ClockCircleOutlined />}>
                  运行: {formatUptime(status?.uptimeSecs || 0)}
                </Tag>
                <Tag icon={<DatabaseOutlined />}>
                  规则: {status?.rulesCount}
                </Tag>
              </>
            )}
          </Space>

          <Space>
            <Button
              icon={<ReloadOutlined />}
              size="small"
              onClick={fetchStatus}
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
                启动
              </Button>
            )}
          </Space>
        </div>
      </Card>

      {/* Stat cards */}
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

      {/* Charts + Live feed */}
      <Row gutter={[16, 16]}>
        <Col xs={24} lg={12}>
          <Card
            title="规则命中分布"
            size="small"
            style={{
              borderRadius: 12,
              border: `1px solid ${token.colorBorderSecondary}`,
            }}
          >
            <RuleHitChart data={stats?.ruleDistribution ?? []} />
          </Card>
        </Col>
        <Col xs={24} lg={12}>
          <Card
            title="最近事件"
            size="small"
            style={{
              borderRadius: 12,
              border: `1px solid ${token.colorBorderSecondary}`,
              maxHeight: 380,
              overflow: "auto",
            }}
          >
            <EventFeed events={recentEvents} />
          </Card>
        </Col>
      </Row>
    </div>
  );
}
