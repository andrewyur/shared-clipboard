import { type Attachment } from "svelte/attachments";
import { fromStore, type Readable } from "svelte/store";

export function useSelect(store: Readable<unknown[]>) {
    let selected = $state({ value: 0 })
    const list = fromStore(store)

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "ArrowUp") {
            if (selected.value == 0) {
                selected.value = list.current.length - 1;
            } else {
                selected.value -= 1;
            }
            e.preventDefault();
        }
        if (e.key === "ArrowDown") {
            if (selected.value == list.current.length - 1) {
                selected.value = 0;
            } else {
                selected.value += 1;
            }
            e.preventDefault();
        }
    }


    const selectAttachment: Attachment = (element) => {
        document.addEventListener("keydown", handleKeydown);
        return () => {
            document.removeEventListener("keydown", handleKeydown);
        }
    }

    return {
        selected,
        selectAttachment
    }
}