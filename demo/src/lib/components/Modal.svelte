<script>
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    /** @type {string} */
    export let title = '';

    /**
     * Closes the modal when clicked outside
     * @param {MouseEvent} event - The click event
     */
    function handleOutsideClick(event) {
        if (event.target === event.currentTarget) {
            dispatch('close');
        }
    }
</script>

<div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full" on:click={handleOutsideClick}>
    <div class="relative top-20 mx-auto p-5 border max-w-2xl shadow-lg rounded-md bg-white">
        <div class="mt-3">
            <h3 class="text-lg leading-6 font-medium text-gray-900">{title}</h3>
            <div class="mt-2 py-3">
                <slot></slot>
            </div>
        </div>
    </div>
</div>