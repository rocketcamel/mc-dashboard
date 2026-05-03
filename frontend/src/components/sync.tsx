import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "./ui/dropdown-menu";
import { Button } from "./ui/button";
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from "./ui/alert-dialog";
import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { backup_world } from "@/lib/world";
import { toast } from "sonner";

type SyncMode = "regular" | "full" | null;

interface SyncProps {
  disabled?: boolean;
}

export default function Sync({ disabled }: SyncProps) {
  const [selectedMode, setSelectedMode] = useState<SyncMode>(null);

  const { mutate } = useMutation({
    mutationFn: () => backup_world("creative"),
    onSuccess: () => {
      toast.success("Successfully synced creative world")
    }
  })

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" disabled={disabled}>Sync Creative World</Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent className="w-48">
          <DropdownMenuItem className="cursor-pointer" onSelect={() => setSelectedMode("regular")}>
            Quick Sync
          </DropdownMenuItem>
          <DropdownMenuItem disabled className="cursor-pointer" onSelect={() => setSelectedMode("full")}>
            Save & Sync
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <AlertDialog open={selectedMode !== null} onOpenChange={(open) => !open && setSelectedMode(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {selectedMode === "regular" ? "Quick Sync" : "Save & Sync"}
            </AlertDialogTitle>
            <AlertDialogDescription className="text-md font-semibold">
              {selectedMode === "regular" ? (
                "Sync the creative world with the main world?"
              ) : (
                <>
                  Save the main world first, then sync the creative world?
                  <p className="text-muted-foreground text-xs mt-2">
                    Note: This will take around twice as long as it saves the main world before syncing.
                    This is desireable if you just made a change to the main world, and don't want to wait 15 minutes.
                  </p>
                </>
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={() => {
              mutate()
            }}>Confirm</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
