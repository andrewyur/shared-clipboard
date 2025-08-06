<script lang="ts" module>
  export type ClipboardData = {
    kind: "image" | "text"
    content: string
    id: number
} | {
    kind: "paths"
    content: [string]
    id: number
  };
</script>

<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

    const { clipboardData }: { clipboardData: ClipboardData } = $props()

    const copyItem = () => invoke("copy_item", {id: clipboardData.id});
</script>

<button onclick={copyItem}>
    {#if clipboardData.kind === "text"}
        <p>{ clipboardData.content }</p>
    {:else if clipboardData.kind === "paths"}
        <p style="font-style:italic; color:gray">{ clipboardData.content.join("\n") }</p>
    {:else}
        <img src={clipboardData.content} alt="clipboard item"/>
    {/if}
</button>

<style>
    button {
        height: 100px;
        width: 100%;
        background-color: #fff;
        padding: 10px;
        margin-bottom: 10px;
        border: 0;
        border-radius: 7px;
        position: relative;
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