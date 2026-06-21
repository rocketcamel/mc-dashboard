import { useEffect, useRef, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select.tsx";
import { ScrollArea } from "./ui/scroll-area.tsx";
import { SERVERS, useLogStream, type World } from "@/lib/world";

export default function LogViewer() {
  const [selected, setSelected] = useState<World>("main");
  const [autoScroll] = useState(true);
  const [logs] = useLogStream(selected);
  const viewport = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (autoScroll) {
      viewport.current.scrollTop = viewport.current.scrollHeight;
    }
  }, [logs, autoScroll]);

  return (
    <Card className="w-full">
      <CardHeader className="flex items-center justify-between space-y-0 pb-4 border-b">
        <CardTitle className="text-sm font-medium">Logs</CardTitle>

        <Select
          onValueChange={(server) => setSelected(server as World)}
          defaultValue="main"
        >
          <SelectTrigger>
            <SelectValue placeholder="Select Server" />
          </SelectTrigger>
          <SelectContent>
            {SERVERS.map((s) => (
              <SelectItem key={s} value={s}>
                {s}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </CardHeader>

      <CardContent>
        <ScrollArea className="h-100">
          <div className="h-full" ref={viewport}>
            {logs.length === 0 && (
              <span className="text-muted-foreground">Waiting for logs...</span>
            )}
            {logs.map((msg, i) => (
              <div
                key={i}
                className={
                  msg.kind === "error" ? "text-destructive" : "text-foreground"
                }
              >
                {msg.data}
              </div>
            ))}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
