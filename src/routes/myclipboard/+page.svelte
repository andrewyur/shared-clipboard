<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";  
  import ClipboardItem, { type ClipboardData } from "$lib/ClipboardItem.svelte";

  let clipboardContents: ClipboardData[] = $state([]);
  let clipboardContentsDiv: HTMLDivElement | undefined = $state()

  onMount(async () => {
    listen<ClipboardData[]>('clipboard-changed', (e) => {
      console.log(e.payload)
      clipboardContents = e.payload;
      if (clipboardContentsDiv?.firstChild) {
        (clipboardContentsDiv.firstChild as HTMLButtonElement).focus()
      }
    })
    await invoke('get_clipboard_contents');
  })
</script>

<div class="items" bind:this={clipboardContentsDiv}>
  {#each clipboardContents as clipboardData (clipboardData.content) }
    <ClipboardItem {clipboardData}/>
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