<script>
	// Connects to a PeerPiper node
	import { onMount } from 'svelte';
	import * as peerpiper from '@peerpiper/peerpiper-browser';
	import { resolveDnsaddr } from '$lib/utils/index.js';
	import { logStore, vladStore, piperStore } from '$lib/stores.js';

	/**
	 * The dnsaddr of the peer to connect to
	 * @type {string}
	 */
	export let dialAddr = '';

	/** @type {string | null} - The error message, if any */
	let errorConnecting = null;
	/** @type {string} - The state of the connection */
	let connectingState = 'idle';

	/** @type {peerpiper.PeerPiper} - The peerpiper instance */
	export let piper;

	/** @type {Uint8Array} - The root CID bytes */
	export let rootCID;

	/** @type {string} - The peer_id of the peer we are connected to */
	export let peer_id;

	// When the user input Enters the dialAddr, we will connect to the peer using connect
	async function handleConnect(evt) {
		connectingState = 'connecting...';

		// assert the dialAddr is not empty
		if (!dialAddr) {
			errorConnecting = 'Please enter a valid Multiaddr';
			connectingState = 'idle';
			return;
		}

		// handle events
		const onEvent = async (evt) => {
			console.log('Event Happened:', evt);
			// if the event is a new connection, we will set the peer_id
			//{
			//    "tag": "new-connection",
			//    "val": {
			//        "peer": "12D3KooWSsrJqCDVunhDq3bV6LGSVE2f1i4xbE47483jJTPgbTED"
			//    }
			//}
			if (evt.tag === 'new-connection') {
				peer_id = evt.val.peer;
			}
		};

		try {
			const dialAddrs = await resolveDnsaddr(dialAddr);
			console.log('Connecting to', dialAddrs);
			piper.connect(dialAddrs, onEvent);
			connectingState = 'connected';
		} catch (error) {
			console.error(error);
			errorConnecting = error;
		}
	}
</script>

<div class="flex flex-col items-center justify-start h-full w-full p-4">
	<h1 class="text-3xl font-bold mb-4">PeerPiper Remote Connect</h1>
	<div class="flex text-lg text-left w-full break-all">
		<div class="flex flex-col">
			<div class="font-semibold mb-4">Connect to a Peer using this address:</div>
			<input
				type="text"
				class="p-2 border border-slate-500 rounded"
				bind:value={dialAddr}
				placeholder="Enter a Peer's Multiaddr"
				disabled={connectingState !== 'idle'}
			/>
			<button
				class="mt-2 p-2 text-white font-semibold rounded"
				class:disabled={connectingState === 'connecting...'}
				class:bg-slate-500={connectingState === 'connecting...'}
				class:bg-green-500={connectingState === 'connected'}
				class:bg-blue-500={connectingState === 'idle'}
				on:click={handleConnect}
			>
				<!-- Use connectingState to manage the text and disableness of this button -->
				{#if connectingState === 'connecting...'}
					Connecting...
				{:else if connectingState === 'connected'}
					Connected
				{:else}
					Connect
				{/if}
			</button>
			{#if errorConnecting}
				<div class="text-red-500 mt-2">{errorConnecting}</div>
			{/if}
		</div>
	</div>
</div>
