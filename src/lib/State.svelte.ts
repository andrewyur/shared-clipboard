import { listen } from "@tauri-apps/api/event";

export type ItemData = {
    contents: {
        kind: "image" | "text";
        content: string;
        id: number;
    } | {
        kind: "paths";
        content: [string];
        id: number;
    }
    id: number;
    is_pinned: boolean;
};

type UpdateMessage = {
    pinned: ItemData[],
    history: ItemData[]
}

export const state = $state<UpdateMessage>({
    pinned: [],
    history: []
}) 


listen<UpdateMessage>("update", (e) => { 
    state.pinned.splice(0, state.pinned.length, ...e.payload.pinned)
    state.history.splice(0, state.history.length, ...e.payload.history)
});

