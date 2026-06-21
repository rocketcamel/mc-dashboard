import {
  Suspense,
  useDeferredValue,
  useMemo,
  useState,
  type SubmitEvent,
} from "react";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Input } from "./ui/input";
import { Button } from "./ui/button";
import { SERVERS, type World } from "@/lib/world";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";
import { useQuery, useSuspenseQuery } from "@tanstack/react-query";

type Player = {
  uuid: string;
  username: string;
};

const PLAYERS: Player[] = [
  { uuid: "069a79f4-44e9-4726-a5be-fca90e38aaf5", username: "Notch" },
  { uuid: "853c80ef-3c37-49fd-aa49-938b674adae6", username: "jeb_" },
  { uuid: "8667ba71-b85a-4004-af54-457a9734eed7", username: "Steve" },
  { uuid: "ec561538-f3fd-461d-aff5-086b22154bce", username: "Alex" },
];

type WhitelistResponse = {
  username: string;
  uuid: string;
};

function player_head(uuid: string) {
  return `https://mc-heads.net/avatar/${uuid}`;
}

async function whitelist_push(username: string, world: World) {
  const response = await fetch(`/api/whitelist/${world}/add`, {
    method: "POST",
    body: JSON.stringify({ username, world }),
  });

  if (!response.ok) {
    throw new Error("failed to add username to whitelist");
  }
}

async function whitelist_get(world: World): Promise<WhitelistResponse[]> {
  const response = await fetch(`/api/whitelist/get?world=${world}`);

  if (!response.ok) {
    throw new Error("failed retrieving server whitelist");
  }
  return response.json();
}

function WhitelistSkeleton() {
  return (
    <div className="max-h-80 space-y-2 overflow-y-auto pr-1">
      {Array.from({ length: 8 }).map((_, i) => (
        <div
          key={i}
          className="flex items-center justify-between rounded-md border border-zinc-800 bg-zinc-900/40 px-3 py-2"
        >
          <div className="flex items-center gap-3">
            <div className="h-10 w-10 animate-pulse rounded-sm bg-zinc-800" />
            <div className="h-4 w-28 animate-pulse rounded bg-zinc-800" />
          </div>
          <div className="h-8 w-20 animate-pulse rounded bg-zinc-800" />
        </div>
      ))}
    </div>
  );
}

function WhitelistAdd({ onAdd }: { onAdd: (username: string) => void }) {
  const [username, setUsername] = useState("");
  const [adding, setAdding] = useState(false);

  const add = async (e: SubmitEvent) => {
    e.preventDefault();
    const value = username.trim();
    if (!value || adding) return;

    try {
      setAdding(true);
      onAdd(value);
    } catch (e) {
      console.warn(e);
    } finally {
      setAdding(false);
    }
  };

  return (
    <form onSubmit={add}>
      <div className="flex items-center justify-between mt-5 gap-3">
        <Input
          onChange={(e) => setUsername(e.target.value)}
          placeholder="add username"
          className="text-zinc-100 placeholder:text-zinc-500"
        />

        <Button type="submit" variant="outline" size="sm">
          Add
        </Button>
      </div>
    </form>
  );
}

function WhitelistPlayers({ world, search }: { world: World; search: string }) {
  const { data: players } = useSuspenseQuery({
    queryKey: ["whitelist_players", world],
    queryFn: () => whitelist_get(world),
    staleTime: 5 * 60 * 1000,
    refetchInterval: 5 * 60 * 1000,
  });

  const deferred = useDeferredValue(search);

  const filtered = useMemo(() => {
    const query = deferred.trim().toLowerCase();
    if (!query) return players;

    return players.filter((player) =>
      player.username.toLowerCase().includes(query),
    );
  }, [players, deferred]);

  return (
    <div className="max-h-80 space-y-2 overflow-y-auto pr-1">
      {filtered.map((player) => (
        <div
          key={player.uuid}
          className="flex items-center justify-between rounded-md border border-zinc-800 px-3 py-2"
        >
          <div className="flex min-w-0 items-center gap-3">
            <img
              src={player_head(player.uuid)}
              alt={`${player.username} head`}
              className="h-10 w-10 rounded-sm border border-zinc-700 bg-zinc-800 object-cover"
            />
            <span className="truncate font-medium">{player.username}</span>
          </div>

          <Button variant="destructive" size="sm">
            Remove
          </Button>
        </div>
      ))}
    </div>
  );
}

export default function Whitelist() {
  const [world, setWorld] = useState<World>("main");
  const [search, setSearch] = useState("");

  const onAdd = (username: string) => {
    whitelist_push(username, world);
  };

  return (
    <Card className="w-full max-w-3xl bg-zinc-950 text-zinc-100 mb-14">
      <CardHeader className="border-b flex items-center justify-between">
        <CardTitle className="text-sm font-semibold tracking-wide">
          Whitelist
        </CardTitle>

        <Select
          onValueChange={(server) => setWorld(server as World)}
          defaultValue="main"
        >
          <SelectTrigger>
            <SelectValue placeholder="Select Server" />
          </SelectTrigger>
          <SelectContent>
            {SERVERS.map((s) => (
              <SelectItem key={s} value={s}>
                {s}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </CardHeader>

      <CardContent className="space-y-4 p-4 pt-0">
        <Input
          onChange={(e) => setSearch(e.target.value)}
          placeholder="search"
          className="bg-zinc-900 text-zinc-100 placeholder:text-zinc-500"
        />

        <Suspense fallback={<WhitelistSkeleton />}>
          <WhitelistPlayers world={world} search={search} />
        </Suspense>
        <WhitelistAdd onAdd={onAdd} />
      </CardContent>
    </Card>
  );
}
