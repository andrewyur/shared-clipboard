<script lang="ts">
    import { mdiClose } from '@mdi/js';
    import "@jamescoyle/svg-icon"
    import { platform } from "@tauri-apps/plugin-os"
    import { invoke } from '@tauri-apps/api/core';
    import { onMount } from 'svelte';
    import { listen } from '@tauri-apps/api/event';
    
    if (platform() === 'macos') {
        document.body.classList.add("macos")
    }

    const { children } = $props()

    const hideWindow = () => invoke("hide_window")
</script>

<div id="titleBar" data-tauri-drag-region>
    <button id="closeButton" aria-label="hides window" onclick={hideWindow}>
        <svg-icon class="cross" type="mdi" path={mdiClose} size="20"></svg-icon>
    </button>
</div>
{@render children() }

<style>
    svg-icon {
        opacity: 0.5;
    }

    .cross:hover {
        background-color: #fff;
        width: 100%;
        height: 100%;
        border-radius: 5px;
    }

    #closeButton {
        width: 45px;
        height: 35px;
        padding: 5px;
    }

    #titleBar {
        width: 100%;
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: end;
    }

    button {
        background-color: transparent;
        border: 0;
        margin: 0;
        padding: 5px;
    }
</style>