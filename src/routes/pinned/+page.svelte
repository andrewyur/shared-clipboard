<script lang="ts">
    import Item from "$lib/Item.svelte";
    import { useSelect } from "$lib/Select.svelte";
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { state } from "$lib/State.svelte";

    const { selectAttachment, register } = useSelect(state.pinned);

    onMount(() => {
        invoke("request_update");
    });
</script>

<div class="items" {@attach selectAttachment}>
    {#if state.pinned.length === 0} 
        <p style="font-style: italic; opacity: 0.5;">No Pinned Items...</p>
    {:else}
        {#each state.pinned as item, i (item.id)}
            <Item itemData={item} index={i} {register}/>
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