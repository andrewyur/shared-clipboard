<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import type { ItemData } from "./State.svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { onMount } from "svelte";

    const {
        itemData,
        index,
        register,
        current = false,
    }: { 
        itemData: ItemData; 
        index: number; 
        current?: boolean
        register: (i: number, e: HTMLButtonElement) => void
    } = $props();

    const copyItem = () => invoke("copy_item", { id: itemData.id });
    const hideWindow = () => getCurrentWindow().hide()

    let itemRef: HTMLButtonElement;

    const pin = (e: MouseEvent) => {
        invoke("pin_item", { id: itemData.id })
    }
    const unpin = (e: MouseEvent) => {
        invoke("unpin_item", { id: itemData.id })
    }

    onMount(() => {
        $effect(() => {
            void index
            register(index, itemRef)
        })
    })

    function handleFocus(e: FocusEvent) {
        requestAnimationFrame(() => {
            itemRef?.scrollIntoView({
                behavior: "smooth",
                block: "center",
            });
        });
    }
</script>

<div style:--bg={current ? "#888" : "#fff"}>
    <button
        class="item"
        onclick={current ? hideWindow : copyItem}
        tabindex={index + 1}
        bind:this={itemRef}
        onfocus={handleFocus}
    >
        {#if itemData.contents.kind === "text"}
            <p>{itemData.contents.content}</p>
        {:else if itemData.contents.kind === "paths"}
            <p style="font-style:italic; color:gray">
                {itemData.contents.content.join("\n")}
            </p>
        {:else}
            <img src={itemData.contents.content} alt="clipboard item" />
        {/if}
    </button>
    
    {#if itemData.is_pinned}
    <button class="action" aria-label="unpin the item" onclick={unpin}>
        <span class="mdi mdi-pin-off"></span>
    </button>
    {:else}
    <button class="action" aria-label="pin the item" onclick={pin}>
        <span class="mdi mdi-pin"></span>
    </button>
    {/if}
</div>

<style>
    div {
        position: relative;
        width: 100%;
    }

    .mdi {
        opacity: 0.5;
    }

    .item {
        height: 100px;
        width: 100%;
        background-color: var(--bg);
        padding: 10px;
        margin-bottom: 10px;
        border: 0;
        border-radius: 7px;
        position: relative;
        outline: none;
        padding: 10px;
        transition: outline 0.1s ease-in-out;
    }

    .action {
        position: absolute;
        top: 0;
        right: 0;
        border: 0;
        margin: 0;
        padding: 10px;
        font-size: small;
        background-color: var(--bg);
        border-radius: 0 7px 0 0 ;
    }

    .item:focus {
        outline: 3px solid black;
    }

    p {
        margin: 0;
        font-size: small;
        height: 100%;
        width: 100%;
        text-align: start;
        overflow: hidden;
    }

    img {
        height: 100%;
    }
</style>
