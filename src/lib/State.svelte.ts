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
    console.log(e.payload)
    state.pinned = e.payload.pinned;
    state.history = e.payload.history;
});

