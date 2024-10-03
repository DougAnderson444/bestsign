<!-- Use lib/components/graphics/disconnected.svelte to show Modal.svelte of Connect.svelte -->
<script>
	import { onMount } from 'svelte';
	import Connect from '$lib/components/Connect.svelte';
	import Disconnect from '$lib/components/graphics/Disconnected.svelte';
	import Modal from './Modal.svelte';
	import { resolveDnsaddr } from '$lib/utils/index.js';

	/** @type {boolean} */
	let showModal = false;

	let dialAddr = '/dnsaddr/peerpiper.io';

	onMount(async () => {
		let res = await resolveDnsaddr(dialAddr);
		console.log(res);
	});

	function toggleModal() {
		showModal = !showModal;
	}
</script>

<button
	class="absolute top-0 right-0 m-2 p-2 cursor-pointer border-neutral-100 rounded-lg bg-white"
	on:click={toggleModal}
	aria-label="Connection Settings"
>
	<Disconnect />
</button>

{#if showModal}
	<Modal title="Connection Settings" on:close={toggleModal}>
		<Connect {dialAddr} />
	</Modal>
{/if}
