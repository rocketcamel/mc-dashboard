import { useEffect, useRef, useState } from "react";

export type World = "main" | "creative";

export type LogMessage = {
  kind: "log" | "error";
  data: string;
};

export type ViewState = "connected" | "connecting" | "off";
export const SERVERS = ["main", "creative"];

export async function get_world_statuses() {
  const response = await fetch("/api/world_status");
  if (!response.ok) {
    throw new Error("error getting world statuses");
  }
  return response.json();
}

export async function backup_world(
  world: World,
  backup_file_name = "latest.tar.gz",
) {
  const response = await fetch("/api/backup_world", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ server_name: world, backup_file_name }),
  });

  if (!response.ok) {
    throw new Error("error backing up world");
  }
  return response.json();
}

export async function sync_world(
  from_server_name: World,
  destination_server_name: World,
) {
  const response = await fetch("/api/sync_world", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ from_server_name, destination_server_name }),
  });

  if (!response.ok) {
    throw new Error("error syncing world");
  }
  return response.json();
}

export type QueryLogsResponse = {
  data: LogMessage[];
};

async function query_logs(world: World): Promise<QueryLogsResponse> {
  const response = await fetch(`/api/logs/query?world=${world}`);

  if (!response.ok) {
    throw new Error("error retreiving logs");
  }

  return response.json();
}

export function useLogStream(world: World): [LogMessage[], ViewState] {
  const MAX_LOGS = 500;

  const [logs, setLogs] = useState<LogMessage[]>([]);
  const [viewState, setViewState] = useState<ViewState>("connecting");
  const buffer = useRef<LogMessage[]>([]);
  const flush = useRef(false);

  const push = (log: LogMessage) => {
    const b = buffer.current;
    b.push(log);

    if (b.length > MAX_LOGS) b.splice(0, b.length - MAX_LOGS);

    if (!flush.current) {
      flush.current = true;

      requestAnimationFrame(() => {
        flush.current = false;
        setLogs(buffer.current.slice());
      });
    }
  };

  useEffect(() => {
    if (!world) return;
    buffer.current.length = 0;

    const create_logs = async (ws: WebSocket) => {
      const current_logs = await query_logs(world);
      buffer.current = current_logs.data.slice(-MAX_LOGS);

      setLogs(buffer.current.slice());

      ws.onopen = () => setViewState("connected");
      ws.onclose = () => setViewState("off");

      ws.onmessage = (e) => {
        const msg = JSON.parse(e.data) as LogMessage;
        push(msg);
      };
    };

    const ws = new WebSocket(`/api/logs/stream/${world}`);
    create_logs(ws);

    return () => ws.close();
  }, [world]);

  return [logs, viewState];
}
