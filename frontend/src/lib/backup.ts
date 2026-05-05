import type { Backup } from "@/components/backup";

export async function get_backups(): Promise<Backup[]> {
  const response = await fetch("/api/backups");
  if (!response.ok) {
    throw new Error("error getting backups");
  }

  return await response.json();
}

export async function get_status(): Promise<boolean> {
  const response = await fetch("/api/status");
  if (!response.ok) {
    throw new Error("error getting status");
  }

  const data = (await response.json()) as { backing_up: boolean };
  return data.backing_up;
}

export const statusQueryOptions = {
  queryKey: ["status"],
  queryFn: get_status,
  staleTime: 10 * 1000,
  refetchInterval: (query: { state: { data?: boolean } }) => {
    return query.state.data ? 2000 : 10000;
  },
};
