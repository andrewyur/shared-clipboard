<script lang="ts">
    import { goto } from "$app/navigation";
    import { mdiCheck } from "@mdi/js";
    import { onMount } from "svelte";
    import { checkAccessibilityPermission, checkInputMonitoringPermission, requestAccessibilityPermission, requestInputMonitoringPermission } from "tauri-plugin-macos-permissions-api";

    let input = $state(false)
    let accessibility = $state(false)

    let permissions = $derived(input && accessibility)

    $effect(() => {
        if (input) {
            goto("/history")
        }
    })


    async function checkInput() {
        input = await checkInputMonitoringPermission() 
    }
    async function checkAccessibility() {
        accessibility = await checkAccessibilityPermission() 
    }

    function handleFocus() {
        checkInput()
        checkAccessibility()
    }
    handleFocus()

    onMount(() => {
        window.addEventListener("focus", handleFocus)
        return () => {
            window.removeEventListener("focus", handleFocus)
        }
    })
</script>

<p class="header">This app needs the following permissions to function:</p>
<div class="container">
    {#if !accessibility}
    <div class="subcontainer">
        <button onclick={requestAccessibilityPermission}>Accessibility</button>
    </div>
    {:else}
        <svg-icon type="mdi" path={mdiCheck}></svg-icon>
    {/if}

    {#if !input}
    <div class="subcontainer">
        <button onclick={requestInputMonitoringPermission}>Input Monitoring</button>
        <p class="subtitle">Toggle the switch next to Purple Clipboard Manager in the settings page that opens</p>
    </div>
    {:else}
        <svg-icon type="mdi" path={mdiCheck}></svg-icon>
    {/if}
</div>

<style>
    .container {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 20px;
        padding: 10px;
        flex-direction: column;
    }

    .subcontainer {
        display: flex; 
        align-items: center; 
        justify-items: start; 
        flex-direction: column; 
        height: min-content;
        padding: 0 10%;
        gap: 5px;
    }

    button {
        background-color: #fff;
        border: 0;
        padding: 10px;
        border-radius: 7px;
        font-size: large;
    }

    button:hover {
        opacity: 0.5;
    }
    
    .header {
        margin: 10px 30px 40px;
        width: fit-content;
        font-size: large;
    }

    .subtitle {
        font-size: x-small;
        margin: 0;
        text-align: center;
    }
</style>