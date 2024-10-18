<!-- Svelte component which tries to find a Plog for you on the DHT. -->
<script>
	import { decode_vlad } from 'bestsign-core-bindings';
	import * as peerpiper from '@peerpiper/peerpiper-browser';
	import { peerRequest } from '$lib/utils/bestsign.js';

	/** @type {peerpiper.PeerPiper} - The peerpiper instance */
	export let piper;

	/** The key to search for in the DHT
	 * @type {string|undefined}
	 */
	let key;

	/**
	 * The list of providers for the given key
	 * @type {Array<string>}
	 */
	let providers = [];

	// use piper to search the DHT for a Plog and make request for it from a peer.
	async function findPlog(evt) {
		if (!key || !piper) {
			console.log('No key provided');
			return;
		}

		console.log('findPlog for vlad:', key);

		// Decode the vlad string to a Uint8Array
		let vlad_slice = decode_vlad(key);

		let cmd = { action: 'GetProviders', key: Array.from(new Uint8Array(vlad_slice)) };

		console.log('sending cmd:', cmd);

		try {
			providers = await piper.command(cmd);
			// providers is an Array of PeerId strings
			console.log('providers:', providers);
		} catch (e) {
			console.error(e);
		}
	}

	/**
	 * Make a peerRequest to get a Plog from a provider
	 * @param {string} peer_id - The PeerId of the provider
	 * @returns {Promise<Uint8Array|null>} - The Plog bytes or null if error
	 */
	async function getPlogFromProvider(peer_id) {
		if (!key) {
			console.log('No key provided');
			return null;
		}

		console.log('getPlogFromProvider:', peer_id);

		let vlad_bytes = decode_vlad(key);

		try {
			let plog = await peerRequest(piper, vlad_bytes, peer_id);
			console.log('plog:', plog);
			return plog;
		} catch (e) {
			console.error(e);
			return null;
		}
	}
</script>

<h1>Find a Plog using a Vlad</h1>

<!-- Input for Vlad with button to trigger handler func -->
<div class="w-full flex-row justify-center">
	<div class="flex w-full">
		<input
			type="text"
			bind:value={key}
			class="flex-1 min-w-0 px-4 py-2 border border-gray-300 rounded-l-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
			placeholder="Enter a VLAD..."
		/>
		<button
			on:click={findPlog}
			class="px-4 py-2 bg-blue-500 text-white font-semibold rounded-r-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
		>
			Find Plog
		</button>
	</div>

	<!-- If providers, list them out -->
	<div class="flex justify-start my-2">
		{#if providers.length > 0}
			<div class="w-1/2 justify-center">
				<h2 class="text-xl font-semibold mb-2">Providers</h2>
				<ul>
					{#each providers as provider}
						<li>{provider}</li>
					{/each}
				</ul>
			</div>
		{:else}
			<p>No providers found</p>
		{/if}
	</div>
</div>
