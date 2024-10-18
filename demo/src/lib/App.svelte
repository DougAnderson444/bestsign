<script>
	import { onMount } from 'svelte';
	import * as peerpiper from '@peerpiper/peerpiper-browser';
	import { default as bestSignWasm } from 'bestsign-core-bindings';

	import { ProvenanceLog } from 'bestsign-core-bindings';
	import Wallet from '$lib/Wallet.svelte';
	import CreateLog from '$lib/components/CreateLog.svelte';
	import UpdateLog from '$lib/components/UpdateLog.svelte';

	import { logStore, vladStore } from '$lib/stores.js';

	/** @type {peerpiper.PeerPiper} - The peerpiper instance */
	let piper;

	/** @type {boolean} Indicates if the wallet has been created */
	let walletCreated = false;

	/**
	 * Gets the Multikey. Must be set to a wallet that provides this function.
	 * @type {function(string): Uint8Array} - Signature is key: string, data: Uint8Array
	 */
	let get_key;

	/**
	 * Proves the Multikey over given data. Must be set to a wallet that provides this function.
	 * @type {function(string, Uint8Array): Uint8Array} - Signature is key: string, data: Uint8Array
	 */
	let prove;

	onMount(async () => {
		try {
			await bestSignWasm();
			await peerpiper.default();
			let ppPromise = new peerpiper.PeerPiper('peerpiper');
			// pp is a Promise because we await openingthe IndexedDB in the browser
			piper = await ppPromise;
		} catch (error) {
			console.error(error);
			return;
		}

		try {
			// see if there is localstorage of rootCID, if so, convert it back to Uint8Array and Get the Plog
			let maybe_storage = localStorage.getItem('rootCID');

			if (maybe_storage) {
				const rootCID = new Uint8Array(
					atob(maybe_storage)
						.split('')
						.map((c) => c.charCodeAt(0))
				);

				// use piper command "GET" to see if there are any local logs
				let cmd = { action: 'System', Get: { key: Array.from(new Uint8Array(rootCID)) } };
				let response = await piper.command(cmd);
				// response is the Log
				$logStore = response;
			}
		} catch (error) {}
	});

	/**
	 * Handles the wallet creation event
	 * @param {Object} wallet - The created wallet instance
	 * @param {Function} wallet.get_mk - Function to get the Multikey
	 * @param {Function} wallet.prove - Function to prove the Multikey over given data
	 */
	function handleWalletCreated(wallet) {
		get_key = wallet.get_mk.bind(wallet);
		prove = wallet.prove.bind(wallet);
		walletCreated = true;
	}

	/**
	 * Handles the log creation event
	 * @param {CustomEvent} event - The log creation event. It has event.detail.log in it.
	 */
	function handleLogCreated(event) {
		logStore.set(event.detail.log);
	}
</script>

{#if !walletCreated}
	<Wallet {handleWalletCreated} />
{:else if !$logStore}
	<CreateLog {get_key} {prove} on:logCreated={handleLogCreated} />
{:else}
	<UpdateLog log={$logStore} {get_key} {prove} {piper} />
{/if}
