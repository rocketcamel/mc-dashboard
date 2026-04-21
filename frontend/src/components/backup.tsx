import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "./ui/dropdown-menu";
import { Button } from "./ui/button";
import { format } from "date-fns";
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from "./ui/alert-dialog";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { get_backups } from "@/lib/backup";
import { Loader2 } from "lucide-react";

export type Backup = {
  filename: string
  bytes: number
  date: Date
}

export default function Backup() {
  const [selectedBackup, setSelectedBackup] = useState<Backup | null>(null);
  const { data: backups, isPending, error } = useQuery({
    queryKey: ["backups"],
    queryFn: get_backups,
    staleTime: 60 * 1000,
  })

  if (error) {
    return (
      <Button disabled className="bg-destructive">Error Getting Backups</Button>
    )
  }
  if (isPending) {
    return (
      <Button disabled>
        <Loader2 className="animate-spin" />
      </Button>
    )
  }

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger>
          <Button variant="outline">Backup Main World</Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent className="w-40">
          {backups!.map((b, i) => (
            <>
              <DropdownMenuItem key={i} onSelect={() => setSelectedBackup(b)}>{formatDate(b.date)} - {formatSize(b.bytes)}</DropdownMenuItem>
            </>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>

      <AlertDialog open={selectedBackup !== null} onOpenChange={(open) => !open && setSelectedBackup(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>World Backup</AlertDialogTitle>
            <AlertDialogDescription className="text-md font-semibold">
              Backup main world at {selectedBackup?.date && formatDate(selectedBackup.date)}? ({selectedBackup && formatSize(selectedBackup.bytes)})
              <p className="text-destructive text-xs mt-2">Warning: This is irreversible after around 3 hours</p>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction>Confirm</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}

function formatDate(date: Date): string {
  return format(date, "MMM dd, h:mm a")
}

function formatSize(bytes: number): string {
  if (bytes < 1024 ** 2) {
    return `${bytes}B`
  }
  if (bytes < 1024 ** 3) {
    return `${(bytes / 1024 ** 2).toPrecision(2)}M`
  }
  return `${(bytes / 1024 ** 3).toPrecision(2)}G`
}
