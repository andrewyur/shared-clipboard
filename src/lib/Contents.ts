import { listen } from "@tauri-apps/api/event";
import { derived, readonly, writable, type Writable } from "svelte/store";

export type ItemData =
| {
    kind: "image" | "text";
    content: string;
    id: number;
}| {
    kind: "paths";
    content: [string];
    id: number;
};

const _history: Writable<ItemData[]> = writable([])
const _pinned: Writable<ItemData[]> = writable([])

export const pinnedIds = derived(_pinned, (p) => new Set(p.map(item => item.id)))
export const pinned = readonly(_pinned)
export const history = readonly(_history)

listen<ItemData[]>("history", (e) => { _history.set(e.payload) });
listen<ItemData[]>("pinned", (e) => { _pinned.set(e.payload) });
