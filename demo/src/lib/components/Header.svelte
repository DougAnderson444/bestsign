<!-- Use lib/components/graphics/disconnected.svelte to show Modal.svelte of Connect.svelte -->
<script>
	import { onMount } from 'svelte';
	import Connect from '$lib/components/Connect.svelte';
	import Disconnect from '$lib/components/graphics/Disconnected.svelte';
	import Modal from './Modal.svelte';
	import FindPlog from './FindPlog.svelte';

	import { logStore, vladStore } from '$lib/stores.js';

	/** @type {boolean} */
	let showModal = false;

	let dialAddr = '/dnsaddr/peerpiper.io';

	export let piper;

	/**
	 * The Root CID to persist
	 * @type {Uint8Array}
	 */
	export let rootCID;

	// string of undef
	/** @type {string|undefined} - The peer_id of the peer we are connected to */
	export let peer_id;

	function toggleModal() {
		showModal = !showModal;
	}
	onMount(async () => {});
</script>

<button
	class="absolute top-0 right-0 m-2 p-2 cursor-pointer border-neutral-100 rounded-lg bg-white"
	on:click={toggleModal}
	aria-label="Connection Settings"
>
	<Disconnect />
</button>

{#if showModal && piper}
	<Modal title="Connection Settings" on:close={toggleModal}>
		<Connect {dialAddr} {piper} {rootCID} bind:peer_id>
			<p>Connected to {peer_id}</p>
			<FindPlog {piper} />
		</Connect>
	</Modal>
{/if}
