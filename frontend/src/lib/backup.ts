import type { Backup } from "@/components/backup";

export async function get_backups(): Promise<Backup[]> {
  const response = await fetch("/api/backups");
  if (!response.ok) {
    throw new Error("error getting backups");
  }

  return await response.json();
}
