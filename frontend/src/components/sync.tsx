import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "./ui/dropdown-menu";
import { Button } from "./ui/button";
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from "./ui/alert-dialog";
import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { backup_world, sync_world } from "@/lib/world";
import { toast } from "sonner";

type SyncMode = "regular" | "full" | null;

interface SyncProps {
  disabled?: boolean;
}

export default function Sync({ disabled }: SyncProps) {
  const [selectedMode, setSelectedMode] = useState<SyncMode>(null);
  const queryClient = useQueryClient();

  const { mutate } = useMutation({
    mutationFn: (mode: Exclude<SyncMode, null>) => {
      switch (mode) {
        case "regular":
          return backup_world("creative")
        case "full":
          return sync_world("main", "creative")
      }
    },
    onSuccess: () => {
      toast.success("Successfully synced creative world")
      queryClient.invalidateQueries({ queryKey: ["status"] })
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
          <DropdownMenuItem className="cursor-pointer" onSelect={() => setSelectedMode("full")}>
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
              {selectedMode === "full" ? (
                <>
                  Save the main world first, then sync the creative world?
                  <p className="text-muted-foreground text-xs mt-2">
                    Note: This will take around twice as long as it saves the main world before syncing.
                    This is desireable if you just made a change to the main world, and don't want to wait 15 minutes.
                  </p>
                </>
              ) : (
                "Sync the creative world with the main world?"
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={() => {
              if (selectedMode) {
                mutate(selectedMode)
              }
            }}>Confirm</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
