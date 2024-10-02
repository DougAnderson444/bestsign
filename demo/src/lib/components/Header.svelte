<!-- Use lib/components/graphics/disconnected.svelte to show Modal.svelte of Connect.svelte -->
<script>
	import Connect from '$lib/components/Connect.svelte';
	import Disconnect from '$lib/components/graphics/Disconnected.svelte';
	import Modal from './Modal.svelte';

	/** @type {boolean} */
	let showModal = false;

	function toggleModal() {
		showModal = !showModal;
	}
</script>

<div
	class="absolute top-0 right-0 m-2 p-2 cursor-pointer border-neutral-100 rounded-lg bg-white"
	on:click={toggleModal}
	aria-label="Connection Settings"
>
	<Disconnect />
</div>

{#if showModal}
	<Modal title="Connection Settings" on:close={toggleModal}>
		<Connect />
	</Modal>
{/if}
