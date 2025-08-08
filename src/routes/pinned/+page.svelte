<script lang="ts">
    import Item from "$lib/Item.svelte";
    import { pinned } from "$lib/Contents";
    import { useSelect } from "$lib/Select.svelte";
    import { onMount } from "svelte";
    import { listen } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";

    const { selected, selectAttachment } = useSelect(pinned);

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
    {#if $pinned.length === 0} 
        <p style="font-style: italic; opacity: 0.5;">No Pinned Items...</p>
    {:else}
        {#each $pinned as item, i (item.id)}
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