<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import type { ItemData } from "./State.svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { onMount } from "svelte";
    import "@jamescoyle/svg-icon"
    import { mdiPin, mdiPinOutline } from "@mdi/js"

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

    let hovered = $state(false)

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

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
    onmouseenter={() => hovered = true}
    onmouseleave={() => hovered = false}
    class:hover={hovered}
>
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
        <svg-icon type="mdi" size="15" path={mdiPin}></svg-icon>
    </button>
    {:else}
    <button class="action" aria-label="pin the item" onclick={pin}>
        <svg-icon type="mdi" size="15" path={mdiPinOutline}></svg-icon>
    </button>
    {/if}
</div>

<style>
    div {
        position: relative;
        width: 100%;
    }

    svg-icon {
        opacity: 0.5;
    }

    .item {
        height: 100px;
        width: 100%;
        background-color: #fff;
        padding: 10px;
        margin-bottom: 10px;
        border: 0;
        border-radius: 7px;
        position: relative;
        outline: 3px solid rgba(0, 0, 0, 0);
        padding: 10px;
        transition: outline 0.2s ease-in-out;
    }

    .action {
        position: absolute;
        top: 0;
        right: 0;
        border: 0;
        padding: 7px;
        margin: 0;
        font-size: small;
        background-color: #fff;
        border-radius: 7px;
        margin: 3px;
        height: 29px;
        width: 29px;
    }

    .action:hover {
        background-color: rgb(238, 238, 238)
    }

    .item:focus {
        outline-color: rgba(0, 0, 0, 1);
    }

    .hover > .item {
        box-shadow: 0px 0px 5px 2px rgb(215, 215, 215) ;
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
