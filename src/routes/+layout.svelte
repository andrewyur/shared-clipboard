<script lang="ts">
    import "../app.css";
    import '@mdi/font/css/materialdesignicons.css'
    import { goto } from "$app/navigation";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { page } from "$app/state";
    import { platform } from "@tauri-apps/plugin-os"
    import { onMount } from "svelte";
    import { listen } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";
    
    const { children } = $props()

    function hideWindow() {
        getCurrentWindow().hide();
    }

    if (platform() === 'macos') {
        document.body.classList.add("macos")
    }

    type Tab = {
        label: string,
        path: string,
        icon: string
    }

    const tabs: Tab[] = [
        {
            label: "clipboard history",
            path: "/history",
            icon: "mdi-clipboard"
        },
        {
            label: "pinned",
            path: "/pinned",
            icon: "mdi-pin"
        },
        {
            label: "other devices",
            path: "/devices",
            icon: "mdi-monitor-multiple"
        }, 
        {
            label: "settings",
            path: "/settings",
            icon: "mdi-cog"
        }
    ]

    const tabIndex = $derived(tabs.findIndex((t) => t.path === page.route.id));

    function handleKeydown(e: KeyboardEvent) {
        if(e.key === "ArrowLeft") {
            goto(tabs[Math.max(tabIndex - 1, 0)].path)
        }
        if(e.key === "ArrowRight") {
            goto(tabs[Math.min(tabIndex + 1, tabs.length - 1)].path)
        }
    }

    onMount(() => {
        document.addEventListener("keydown", handleKeydown)

        const unlistenWindowShown = listen("window-shown", () => {
            invoke("request_update");
            goto("/history")
        });
        
        invoke("request_update");

        return async () => {
            (await unlistenWindowShown)()
            document.removeEventListener("keydown", handleKeydown)
        }
    })
</script>

<div id="titleBar" data-tauri-drag-region>
    <button id="closeButton" aria-label="hides window" onclick={hideWindow}>
        <span class="mdi mdi-close"></span>
    </button>
</div>
<nav id="tabs">
    <span class="mdi mdi-arrow-left tab" tabindex="-1" style="opacity: 0.25; margin-right:20px"></span>
    {#each tabs as { path, icon, label }}
    <button class="tab" aria-label={label} onclick={() => goto(path)} class:active={page.route.id === path}><span class="mdi {icon}"></span></button>
    {/each}
    <span class="mdi mdi-arrow-right tab" tabindex="-1" style="opacity: 0.25; margin-left:20px"></span>
</nav>

<main>
    {@render children()}
</main>

<style>
    .mdi-close {
        font-size: 1.5em;
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
        align-items: center;
        margin-top: -10px;
        gap: 5px;
        padding-bottom: 10px;
    }

    .tab {
        font-size: medium;
        border-radius: 5px;
        color: #00000080
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
        padding-top: 10px;
        scroll-behavior: smooth;
    }
</style>
