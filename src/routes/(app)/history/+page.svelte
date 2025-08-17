<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import Item from "$lib/Item.svelte";
  import { state } from "$lib/State.svelte";
  import { useSelect } from "$lib/Select.svelte";

  const { selectAttachment, register } = useSelect(state.history);

  onMount(() => {
    invoke("request_update");
  });
</script>

<div class="items" {@attach selectAttachment}>
  {#if state.history.length === 0}
    <p style="font-style: italic; opacity: 0.5;">No Clipboard History yet...</p>
  {:else}
    {#each state.history as item, i (item.id)}
      <Item itemData={item} index={i} {register} />
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
