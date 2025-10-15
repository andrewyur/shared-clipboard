import { listen } from "@tauri-apps/api/event";
import { type Attachment } from "svelte/attachments";

export function useSelect(list: unknown[]) {
    let selected = $state(0)
    let update = $state(0)

    const refs: HTMLButtonElement[] = []

    function register(index: number, element: HTMLButtonElement) {
        refs[index] = element
    } 

    function handleKeydown(key: string) {
        if (key === "UpArrow") {
            if (selected == 0) {
                selected = list.length - 1;
            } else {
                selected -= 1;
            }
        }
        if (key === "DownArrow") {
            if (selected == list.length - 1) {
                selected = 0;
            } else {
                selected += 1;
            }
        }
    }

    const selectAttachment: Attachment = (element) => {
        const unlistenKey = listen<string>("key", ({ payload }) => handleKeydown(payload))
        const unlistenWindowShown = listen("window-shown", () => {
            if( selected == 0 ) {
                update += 1;
            } else {
                selected = 0
            }
        },);
        refs[selected]?.focus()
        return async () => {
            (await unlistenWindowShown)();
            (await unlistenKey)();
        }
    }

    $effect(() => {
        void update
        void selected
        requestAnimationFrame(() => {
            refs[selected]?.focus()
        })
    })

    return {
        register, 
        selectAttachment,
    }
}