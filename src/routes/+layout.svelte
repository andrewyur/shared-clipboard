<script lang="ts">
    import "../app.css";
    import '@jamescoyle/svg-icon'
    import 'overlayscrollbars/overlayscrollbars.css';
    import { mdiClose, mdiArrowRight, mdiArrowLeft, mdiClipboard, mdiPin, mdiMonitorMultiple, mdiCog } from "@mdi/js"
    import { goto } from "$app/navigation";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { page } from "$app/state";
    import { platform } from "@tauri-apps/plugin-os"
    import { onMount, tick } from "svelte";
    import { listen } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";
    import { OverlayScrollbars } from "overlayscrollbars"
    
    const { children } = $props()

    function hideWindow() {
        getCurrentWindow().hide();
    }

    const mac = platform() === 'macos'
    if (mac) {
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
            icon: mdiClipboard
        },
        {
            label: "pinned",
            path: "/pinned",
            icon: mdiPin
        },
        {
            label: "other devices",
            path: "/devices",
            icon: mdiMonitorMultiple
        }, 
        {
            label: "settings",
            path: "/settings",
            icon: mdiCog
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
        if(e.key === "Escape") {
            hideWindow()
        }
    }

    function handleBlur() {
        console.log("blurred")
        setTimeout(() => {
            if(!document.hasFocus()) {
                hideWindow()
            }
        }, 100)
    }

    
    onMount(() => {
        tick().then(() => {
            OverlayScrollbars(document.querySelector('main') as HTMLElement, {});
        })

        document.addEventListener("keydown", handleKeydown)
        window.addEventListener("blur", handleBlur)

        const unlistenWindowShown = listen("window-shown", () => {
            invoke("request_update");
            goto("/history")
        });
        
        invoke("request_update");

        return async () => {
            (await unlistenWindowShown)()
            document.removeEventListener("keydown", handleKeydown)
            window.removeEventListener("blur", handleBlur)
        }
    })
</script>

<div id="titleBar" data-tauri-drag-region>
    <button id="closeButton" aria-label="hides window" onclick={hideWindow}>
        <svg-icon class="cross" type="mdi" path={mdiClose} size="20"></svg-icon>
    </button>
</div>
<nav id="tabs">
    <svg-icon size="20" type="mdi" path={mdiArrowLeft} tabindex="-1" style="opacity: 0.25; margin-right:20px"></svg-icon>
    {#each tabs as { path, icon, label }}
    <button class="tab" aria-label={label} onclick={() => goto(path)} class:active={page.route.id === path}><svg-icon type="mdi" size="20" path={icon}></svg-icon></button>
    {/each}
    <svg-icon size="20" type="mdi" path={mdiArrowRight} tabindex="-1" style="opacity: 0.25; margin-left:20px"></svg-icon>

</nav>

<main>
    {@render children()}
</main>

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
    
    nav {
        display: flex;
        justify-content: center;
        align-items: center;
        margin-top: 0;
        gap: 5px;
        padding-bottom: 10px;
    }

    .tab {
        font-size: medium;
        border-radius: 5px;
        height: 30px;
        width: 30px;
    }
    
    button {
        background-color: transparent;
        border: 0;
        margin: 0;
        padding: 5px;
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
        /* overflow-y: scroll; */
        padding-top: 10px;
        scroll-behavior: smooth;
    }
</style>
