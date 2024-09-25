<script>
    import { createEventDispatcher } from 'svelte';

    /** @type {Array<{ key: string, value: string }>} */
    export let keyValuePairs = [];

    /** @type {string} */
    let keyInput = '/github/user';

    /** @type {string} */
    let valueInput = 'douganderson444';

    const dispatch = createEventDispatcher();

    /**
     * Adds a new key-value pair to the list
     */
    function addKeyValuePair() {
        if (keyInput && valueInput) {
            keyValuePairs = [...keyValuePairs, { key: keyInput, value: valueInput }];
            dispatch('update', keyValuePairs);
            keyInput = '';
            valueInput = '';
        }
    }
</script>

<div class="mt-6">
    <h2 class="text-xl font-semibold mb-2">Add Key-Value Pairs</h2>
    <div class="flex space-x-2 mb-2">
        <input
            bind:value={keyInput}
            placeholder="Key (e.g., /github/user)"
            class="flex-1 p-2 border rounded"
        />
        <input
            bind:value={valueInput}
            placeholder="Value (e.g., douganderson444)"
            class="flex-1 p-2 border rounded"
        />
        <button
            on:click={addKeyValuePair}
            class="py-2 px-4 bg-green-500 text-white rounded hover:bg-green-600"
        >
            Add
        </button>
    </div>
    {#if keyValuePairs.length > 0}
        <ul class="list-disc pl-5 mb-4">
            {#each keyValuePairs as pair}
                <li>{pair.key}: {pair.value}</li>
            {/each}
        </ul>
    {/if}
</div>