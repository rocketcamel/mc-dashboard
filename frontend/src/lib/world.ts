export type World = "main" | "creative";

export async function backup_world(world: World, backup_file_name = "latest.tar.gz") {
  const response = await fetch("/api/backup_world", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ server_name: world, backup_file_name })
  })

  if (!response.ok) {
    throw new Error("error backing up world")
  }
  return response.json()
}
