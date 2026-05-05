import { cn } from "@/lib/utils";

const styles = {
  running: "bg-green-500",
  starting: "bg-yellow-500 animate-pulse",
  stopped: "bg-gray-400",
  error: "bg-red-500",
  unknown: "bg-gray-300",
} as const;

export type ContainerStatus = keyof typeof styles;

export default function StatusDot({ status }: { status?: ContainerStatus }) {
  const s = status ?? "unknown"
  return (
    <span className={cn("inline-block h-2.5 w-2.5 rounded-full", styles[s])} title={s} />
  )
}
