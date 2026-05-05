export type World = "main" | "creative";

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
