import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "./ui/dropdown-menu";
import { Button } from "./ui/button";
import { format } from "date-fns";
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from "./ui/alert-dialog";
import { useState } from "react";

export type Backup = {
  date: Date
}

const mockData: Backup[] = Array.from({ length: 15 }, (_, i) => ({
  date: new Date(Date.now() - i * 15 * 60 * 1000)
}))

export default function Backup() {
  const [selectedBackup, setSelectedBackup] = useState<Backup | null>(null);

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger>
          <Button variant="outline">Backup Main World</Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent className="w-36">
          {mockData.map((b, i) => (
            <>
              <DropdownMenuItem key={i} onSelect={() => setSelectedBackup(b)}>{formatDate(b.date)}</DropdownMenuItem>
            </>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>

      <AlertDialog open={selectedBackup !== null} onOpenChange={(open) => !open && setSelectedBackup(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>World Backup</AlertDialogTitle>
            <AlertDialogDescription className="text-lg font-semibold">
              Backup main world at {selectedBackup?.date && formatDate(selectedBackup.date)}?
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
