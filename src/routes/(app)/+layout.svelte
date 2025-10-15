<script lang="ts">
    import "../../app.css";
    import '@jamescoyle/svg-icon'
    import 'overlayscrollbars/overlayscrollbars.css';
    import { mdiArrowRight, mdiArrowLeft, mdiClipboard, mdiPin, mdiMonitorMultiple, mdiCog } from "@mdi/js"
    import { goto } from "$app/navigation";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { page } from "$app/state";
    import { onMount, tick } from "svelte";
    import { listen } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";
    import { OverlayScrollbars } from "overlayscrollbars"
    
    const { children } = $props()

    const hideWindow = () => invoke("hide_window")

    listen<string>("key", ({ payload }) => {
        if(payload === "Enter") {
            (document.activeElement as HTMLButtonElement | HTMLElement)?.click()
        }
    })

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
        // {
        //     label: "other devices",
        //     path: "/devices",
        //     icon: mdiMonitorMultiple
        // }, 
        // {
        //     label: "settings",
        //     path: "/settings",
        //     icon: mdiCog
        // }
    ]

    const tabIndex = $derived(tabs.findIndex((t) => t.path === page.url.pathname));

    onMount(() => {
        tick().then(() => {
            OverlayScrollbars(document.querySelector('main') as HTMLElement, {});
        })

        const unlistenKey = listen<string>("key", ({ payload }) => {
            if(payload === "LeftArrow") {
                goto(tabs[Math.max(tabIndex - 1, 0)].path)
            }
            if(payload === "RightArrow") {
                goto(tabs[Math.min(tabIndex + 1, tabs.length - 1)].path)
            }
            if(payload === "Escape") {
                hideWindow()
            }
        });

        const unlistenWindowShown = listen("window-shown", () => {
            invoke("request_update");
            goto("/history")
        });
        
        invoke("request_update");
        
        return async () => {
            (await unlistenKey)();
            (await unlistenWindowShown)();
        }
    })
</script>

<nav id="tabs">
    <svg-icon size="20" type="mdi" path={mdiArrowLeft} tabindex="-1" style="opacity: 0.25; margin-right:20px"></svg-icon>
    {#each tabs as { path, icon, label }}
    <button class="tab" aria-label={label} onclick={() => goto(path)} class:active={page.url.pathname === path}><svg-icon type="mdi" size="20" path={icon}></svg-icon></button>
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
