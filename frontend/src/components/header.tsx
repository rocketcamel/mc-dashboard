import { User, LogOut } from "lucide-react";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "./ui/dropdown-menu";

export default function Header() {
  return (
    <div className="flex max-w-6xl mx-auto justify-between p-4 items-center">
      <h1 className="text-xl font-bold">mc-rocket-management</h1>
      <div className="rounded-full w-10 h-10 flex items-center justify-center bg-muted/50">
        <DropdownMenu>
          <DropdownMenuTrigger className="p-2.5 cursor-pointer rounded-full focus-visible:ring-2 focus-visible:ring-zinc-500/50 focus-visible:outline-none">
            <User />
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuLabel>My Account</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem className="justify-between">Logout <LogOut /></DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  )
}
