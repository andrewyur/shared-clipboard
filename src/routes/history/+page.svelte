<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import Item from "$lib/Item.svelte";
  import { history } from "$lib/Contents";
  import { useSelect } from "$lib/Select.svelte";

  const { selected, selectAttachment } = useSelect(history);

  onMount(() => {
    const unlistenWindowShown = listen("window-shown", () => {
      selected.value = 0;
      invoke("get_history");
      invoke("get_pinned");
    });

    const unlistenClipboardChange = listen(
      "history",
      () => (selected.value = 0),
    );

    invoke("get_history");
    invoke("get_pinned");

    return async () => {
      (await unlistenClipboardChange)();
      (await unlistenWindowShown)();
    };
  });
</script>

<div class="items" {@attach selectAttachment}>
  {#if $history.length === 0}
    <p style="font-style: italic; opacity: 0.5;">Clipboard is Empty...</p>
  {:else}
    {#each $history as item, i (item.id)}
      <Item itemData={item} index={i} {selected} />
    {/each}
  {/if}
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
