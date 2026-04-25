import { queryOptions } from "@tanstack/react-query";
import { ApiError } from "./api-error";

export const authQueryOptions = queryOptions({
  queryKey: ["auth"],
  queryFn: async () => {
    const response = await fetch("/api/auth/me");
    if (!response.ok) return null;
    return response.json();
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
    throw new ApiError(response.status, "error logging in");
  }

  return response.json();
}
