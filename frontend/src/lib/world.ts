export type World = "main" | "creative";

export async function sync_world(world: World) {
  const response = await fetch("/api/sync", {
    method: "POST",
    body: JSON.stringify({ world })
  })

  if (!response.ok) {
    throw new Error("error syncing world")
  }
  return await response.json()
}
