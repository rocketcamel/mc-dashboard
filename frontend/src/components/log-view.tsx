import { useEffect, useRef, useState } from "react";
import { Badge } from "./ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select.tsx";
import { ScrollArea } from "./ui/scroll-area.tsx";
import { useLogStream, type World } from "@/lib/world";
import { Loader2 } from "lucide-react";

const SERVERS = ["main", "creative"];

export default function LogViewer() {
  const [selected, setSelected] = useState<World>("main");
  const [autoScroll, setAutoScroll] = useState(true);
  const [logs, viewState] = useLogStream(selected);
  const bottom = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (autoScroll) {
      bottom.current?.scrollIntoView()
    }
  }, [logs, autoScroll])

  return (
    <Card className="w-full">
      <CardHeader className="flex items-center justify-between space-y-0 pb-4 border-b">
        <CardTitle className="text-sm font-medium">Logs</CardTitle>

        <Select onValueChange={setSelected as any}>
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
        <ScrollArea className="h-[400px]">
          {logs.length === 0 && (
            <span className="text-muted-foreground">Waiting for logs...</span>
          )}
          {logs.map((msg, i) => (
            <div key={i} className={msg.kind === "error" ? "text-destructive" : "text-foreground"}>
              {msg.data}
            </div>
          ))}
          <div ref={bottom}></div>
        </ScrollArea>
      </CardContent>
    </Card>
  )
}
