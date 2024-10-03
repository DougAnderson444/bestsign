<script lang="js">
	import Wallet from '$lib/Wallet.svelte';
	import CreateLog from '$lib/components/CreateLog.svelte';
	import UpdateLog from '$lib/components/UpdateLog.svelte';

	/** @type {boolean} Indicates if the wallet has been created */
	let walletCreated = false;

	/**
	 * Gets the Multikey. Must be set to a wallet that provides this function.
	 * @type {Function}
	 */
	let get_key;

	/**
	 * Proves the Multikey over given data. Must be set to a wallet that provides this function.
	 * @type {Function}
	 */
	let prove;

	/** @type {Uint8Array} The created ProvenanceLog instance */
	let log;

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
	 * @param {Object} event - The log creation event
	 * @param {ProvenanceLog} event.detail.provenanceLog - The created ProvenanceLog instance
	 * @param {string} event.detail.displayData - The display data of the created ProvenanceLog
	 */
	function handleLogCreated(event) {
		log = event.detail.log;
	}
</script>

{#if !walletCreated}
	<Wallet {handleWalletCreated} />
{:else if !log}
	<CreateLog {get_key} {prove} on:logCreated={handleLogCreated} />
{:else}
	<UpdateLog {log} {get_key} {prove} />
{/if}
