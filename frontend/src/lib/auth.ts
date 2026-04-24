import { queryOptions } from "@tanstack/react-query";

export const authQueryOptions = queryOptions({
  queryKey: ["auth"],
  queryFn: async () => {
    const response = await fetch("/api/auth/me");
    if (!response.ok) return null;
    return await response.json();
  },
  staleTime: 1000 * 60 * 5,
});

export async function login(username: string, password: string) {
  const response = await fetch("/api/auth/login", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ username, auth: password }),
  });

  if (!response.ok) {
    throw new Error("error logging in", { cause: response.statusText });
  }

  return await response.json();
}
