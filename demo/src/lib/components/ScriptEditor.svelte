<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    /** @type {string} */
    export let lockScript = '';

    /** @type {string} */
    export let unlockScript = '';

    function updateScripts() {
        dispatch('update', { lockScript, unlockScript });
    }

    function closeModal() {
        dispatch('close');
    }
</script>

<div class="space-y-6">
    <div>
        <label for="lockScript" class="block text-sm font-medium text-gray-700 mb-2">
            Lock Script
        </label>
        <textarea
            id="lockScript"
            bind:value={lockScript}
            class="w-full p-2 font-mono text-sm bg-gray-100 border border-gray-300 rounded-md shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
            rows="3"
        ></textarea>
    </div>

    <div>
        <label for="unlockScript" class="block text-sm font-medium text-gray-700 mb-2">
            Unlock Script
        </label>
        <textarea
            id="unlockScript"
            bind:value={unlockScript}
            class="w-full p-2 font-mono text-sm bg-gray-100 border border-gray-300 rounded-md shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
            rows="3"
        ></textarea>
    </div>

    <div class="flex justify-between">
        <button
            on:click={closeModal}
            class="py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-gray-500 hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-400"
        >
            Close
        </button>
        <button
            on:click={updateScripts}
            class="py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
        >
            Update Scripts
        </button>
    </div>
</div>