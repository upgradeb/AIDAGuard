import { List, Space, Tag, Typography, theme } from "antd";
import { WarningOutlined } from "@ant-design/icons";
import type { DetectionEvent } from "../types";
import dayjs from "dayjs";

interface EventFeedProps {
  events: DetectionEvent[];
}

export default function EventFeed({ events }: EventFeedProps) {
  const { token } = theme.useToken();

  if (events.length === 0) {
    return (
      <div
        style={{
          textAlign: "center",
          padding: 32,
          color: token.colorTextQuaternary,
        }}
      >
        <WarningOutlined style={{ fontSize: 32, marginBottom: 8 }} />
        <br />
        暂无检测事件
      </div>
    );
  }

  return (
    <List
      dataSource={events}
      renderItem={(item) => (
        <List.Item
          style={{ padding: "8px 0", borderBottom: `1px solid ${token.colorBorderSecondary}` }}
        >
          <div style={{ width: "100%" }}>
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                marginBottom: 4,
              }}
            >
              <Space size={8}>
                <Tag color="orange">{item.ruleId}</Tag>
                <Tag>{item.strategy}</Tag>
              </Space>
              <Typography.Text type="secondary" style={{ fontSize: 12 }}>
                {dayjs(item.timestampMs).format("HH:mm:ss")}
              </Typography.Text>
            </div>
            <Typography.Text
              type="secondary"
              style={{ fontSize: 12 }}
              ellipsis
            >
              {item.requestPath} · HTTP {item.responseStatus}
            </Typography.Text>
          </div>
        </List.Item>
      )}
    />
  );
}
