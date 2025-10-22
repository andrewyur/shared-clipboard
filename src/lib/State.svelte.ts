import { listen } from "@tauri-apps/api/event";

export type ItemData = {
    kind: "image" | "text";
    content: string;
    id: number;
    is_pinned: boolean
} | {
    kind: "paths";
    content: [string];
    id: number;
    is_pinned: boolean
}

type UpdateMessage = {
    pinned: ItemData[],
    history: ItemData[]
}

export const state = $state<UpdateMessage>({
    pinned: [],
    history: []
}) 


listen<UpdateMessage>("update", (e) => { 
    e.payload.pinned.forEach(i => i.is_pinned = true);
    const pinned = new Set(e.payload.pinned.map(i => i.id))

    e.payload.history.forEach(i => {
        if (pinned.has(i.id)) {
            i.is_pinned = true
        }
    })

    state.pinned.splice(0, state.pinned.length, ...e.payload.pinned)
    state.history.splice(0, state.history.length, ...e.payload.history)

    console.log(state)
});

