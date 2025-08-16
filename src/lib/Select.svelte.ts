import { listen } from "@tauri-apps/api/event";
import { type Attachment } from "svelte/attachments";

export function useSelect(list: unknown[]) {
    let selected = $state(0)
    let update = $state(0)

    $inspect(selected)

    const refs: HTMLButtonElement[] = []

    function register(index: number, element: HTMLButtonElement) {
        refs[index] = element
    } 

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "ArrowUp") {
            if (selected == 0) {
                selected = list.length - 1;
            } else {
                selected -= 1;
            }
            e.preventDefault();
        }
        if (e.key === "ArrowDown") {
            if (selected == list.length - 1) {
                selected = 0;
            } else {
                selected += 1;
            }
            e.preventDefault();
        }
    }

    const selectAttachment: Attachment = (element) => {
        document.addEventListener("keydown", handleKeydown);
        const unlistenWindowShown = listen("window-shown", () => {
            if( selected == 0 ) {
                update += 1;
            } else {
                selected = 0
            }
        },);
        return async () => {
            (await unlistenWindowShown)()
            document.removeEventListener("keydown", handleKeydown);
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