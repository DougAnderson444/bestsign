<script lang="js">
	import Wallet from '$lib/Wallet.svelte';
	import Log from '$lib/Log.svelte';

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
</script>

{#if !walletCreated}
	<Wallet bind:walletCreated {handleWalletCreated} />
{:else}
	<Log {get_key} {prove} />
{/if}
