<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import type { ItemData } from "./Contents"
    import { pinnedIds } from "./Contents"

    const {
        itemData,
        index,
        selected,
    }: { 
        itemData: ItemData; 
        index: number; 
        selected: { value: number} 
    } = $props();

    const copyItem = () => invoke("copy_item", { id: itemData.id });

    let itemRef: undefined | HTMLButtonElement = $state();
    let isPinned: boolean = $derived($pinnedIds.has(itemData.id))

    const pin = (e: MouseEvent) => {
        invoke("pin_item", { id: itemData.id })
    }
    const unpin = (e: MouseEvent) => {
        invoke("unpin_item", { id: itemData.id })
    }

    $effect(() => {
        if (selected.value === index) {
            itemRef?.focus();
        }
    });

    function handleFocus(e: FocusEvent) {
        requestAnimationFrame(() => {
            itemRef?.scrollIntoView({
                behavior: "smooth",
                block: "center",
            });
        });
    }
</script>

<div>
    <button
        class="item"
        onclick={copyItem}
        tabindex={index + 1}
        bind:this={itemRef}
        onfocus={handleFocus}
    >
        {#if itemData.kind === "text"}
            <p>{itemData.content}</p>
        {:else if itemData.kind === "paths"}
            <p style="font-style:italic; color:gray">
                {itemData.content.join("\n")}
            </p>
        {:else}
            <img src={itemData.content} alt="clipboard item" />
        {/if}
    </button>
    
    {#if isPinned}
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

    .item {
        height: 100px;
        width: 100%;
        background-color: #fff;
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
        background-color: #fff;
        border-radius: 0 12px 0 0 ;
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
