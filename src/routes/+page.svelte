<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  
  let clipboardContents = $state();
  onMount(async () => {
    listen('clipboard-changed', (e) => {
      clipboardContents = e.payload;
    })
    await invoke('get_clipboard_contents');
  })
</script>

<div>
<p>{clipboardContents}</p>
</div>

<style>
  div {
    width: 100%;
    height: 100vh;
    margin: 0;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  :global(body) {
    padding: 0;
    margin: 0;
  }
</style>