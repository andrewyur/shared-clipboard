<script lang="ts">
    import "../app.css";
    import '@mdi/font/css/materialdesignicons.css'
    import { goto } from "$app/navigation";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { page } from "$app/state";
    import { platform } from "@tauri-apps/plugin-os"
    
    function hideWindow() {
        getCurrentWindow().hide();
    }

    if (platform() === 'macos') {
        document.body.classList.add("macos")
    }
</script>

<div id="titleBar" data-tauri-drag-region>
    <button id="closeButton" aria-label="hides window" onclick={hideWindow}>
        <span class="mdi mdi-close"></span>
    </button>
</div>
<nav id="tabs">
    <button class="tab" onclick={() => goto("/myclipboard")} class:active={page.route.id === '/myclipboard'}>My Clipboard</button>
    <button class="tab" onclick={() => goto("/otherdevices")} class:active={page.route.id === '/otherdevices'}>Other Devices</button>
    <button class="tab" aria-label="settings" onclick={() => goto("/settings")}><span class="mdi mdi-cog"></span></button>
</nav>

<main>
    <slot />
</main>

<style>
    .mdi-close {
        font-size: 1.5em;
    }

    #closeButton:hover {
        background-color: #9292927e;
        height: 100%;
        width: 30px;
        border: 0;
        margin: 0;
    }

    #titleBar {
        width: 100%;
        height: 28px;
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: end;
    }

    nav {
        display: flex;
        justify-content: center;
        gap: 5px;
    }

    .tab {
        font-size: small;
        border-radius: 5px;
    }
    
    button {
        background-color: transparent;
        border: 0;
    }

    .active {
        background-color: #fff;
    }

    main {
        box-sizing: border-box;
        width: 100%;
        height: 100%;
        margin: 0;
        padding: 20px;
        gap: 20px;
        display: flex;
        flex-direction: column;
        overflow-y: scroll;
        scrollbar-width: thin;
    }
</style>
