<script lang="ts" module>
  export type ClipboardData = {
    type: "image" | "text"
    content: string
  } | {
    type: "filepath"
    content: [string]
  };
</script>

<script lang="ts">
    const { clipboardData }: { clipboardData: ClipboardData } = $props()
</script>

<button>
    {#if clipboardData.type === "text" || clipboardData.type === "filepath"}
        <p>{ clipboardData.content }</p>
    {:else}
        <img src={"data:image/png;base64," + clipboardData.content} alt="clipboard item"/>
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