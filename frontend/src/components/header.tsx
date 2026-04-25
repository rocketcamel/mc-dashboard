import { User, LogOut } from "lucide-react";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "./ui/dropdown-menu";
import { useQuery } from "@tanstack/react-query";
import { authQueryOptions } from "@/lib/auth";

export default function Header() {
  const { data: user } = useQuery(authQueryOptions);
  const handleLogout = async () => {
    await fetch("/api/auth/logout", { method: "POST" });
    window.location.href = "/login";
  }

  return (
    <div className="flex max-w-6xl mx-auto justify-between p-4 items-center">
      <h1 className="text-xl font-bold">mc-rocket-management</h1>
      <div className="rounded-full w-10 h-10 flex items-center justify-center bg-muted/50">
        <DropdownMenu>
          <DropdownMenuTrigger className="p-2.5 cursor-pointer rounded-full focus-visible:ring-2 focus-visible:ring-zinc-500/50 focus-visible:outline-none">
            <User />
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuLabel>{user?.name ?? "My Account"}</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem className="justify-between" onClick={handleLogout}>Logout <LogOut /></DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  )
}
