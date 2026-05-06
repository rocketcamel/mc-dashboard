import { Badge } from "@/components/ui/badge"
import { cn } from "@/lib/utils"

const config = {
  running: { label: "Running", className: "bg-green-500/15 text-green-700 dark:text-green-400" },
  starting: { label: "Starting", className: "bg-yellow-500/15 text-yellow-700 dark:text-yellow-400 animate-pulse" },
  stopped: { label: "Stopped", className: "bg-gray-500/15 text-gray-600 dark:text-gray-400" },
  error: { label: "Error", className: "bg-red-500/15 text-red-700 dark:text-red-400" },
  unknown: { label: "Unknown", className: "bg-gray-500/10 text-gray-500" },
  fetching: { label: "Fetching...", className: "bg-gray-500/10 text-gray-500" },
} as const

export type ContainerStatus = keyof typeof config

export default function StatusBadge({ status, isPending }: { status?: ContainerStatus, isPending?: boolean }) {
  const s = status ?? (isPending ? "fetching" : "unknown")
  const { label, className } = config[s]
  return (
    <Badge variant="outline" className={cn("border-transparent", className)}>
      {label}
    </Badge>
  )
}
