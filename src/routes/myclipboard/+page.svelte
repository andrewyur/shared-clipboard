<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import ClipboardItem, { type ClipboardData } from "$lib/ClipboardItem.svelte";

  let clipboardContents: ClipboardData[] = $state([]);
  let selected = $state(0);

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "ArrowUp") {
      if (selected == 0) {
        selected = clipboardContents.length - 1;
      } else {
        selected -= 1;
      }
      e.preventDefault();
    }
    if (e.key === "ArrowDown") {
      if (selected == clipboardContents.length - 1) {
        selected = 0;
      } else {
        selected += 1;
      }
      e.preventDefault();
    }
  }

  onMount(() => {
    const unlistenWindowShown = listen("window-shown", () => {
      invoke("get_clipboard_contents");
    });

    const unlistenClipboardChange = listen<ClipboardData[]>(
      "clipboard-changed",
      (e) => {
        clipboardContents = e.payload;
        selected = 0;
      },
    );

    document.addEventListener("keydown", handleKeydown);

    invoke("get_clipboard_contents");

    return async () => {
      (await unlistenClipboardChange)();
      (await unlistenWindowShown)();
      document.removeEventListener("keydown", handleKeydown);
    };
  });
</script>

<div class="items">
  {#each clipboardContents as clipboardData, i (clipboardData.id)}
    <ClipboardItem {clipboardData} index={i} {selected} />
  {/each}
</div>

<style>
  .items {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 15px;
  }
</style>
