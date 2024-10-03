<script>
	import { default as wasm, WasmWallet } from 'multiwallet-bindings';
	import { onMount } from 'svelte';

	/** @type {string} Key to store encrypted seed in local storage */
	const KEY_BASE64_SEED = 'encrypted_seed_base64';

	/** @type {string | null} Base64 encoded seed */
	let b64Seed;

	/** @type {string} User's username */
	let username = '';

	/** @type {string} User's password */
	let password = '';

	/** @type {string} Optional encrypted seed */
	let encrypted_seed = '';

	/** @type {WasmWallet | null} Instance of WasmWallet */
	let wallet = null;

	/** @type {string} Error message to display */
	let error = '';

	/** @type {Function | null} Function to get the Multikey */
	let get_key = null;

	/** @type {Function | null} Function to prove the Multikey over given data */
	let prove = null;

	/** @type {boolean} Indicates if WASM is loaded */
	let isWasmLoaded = false;

	/** @type {Function} Callback function when wallet is created */
	export let handleWalletCreated;

	onMount(async () => {
		try {
			await wasm();

			// check for seed in local storage, if not null, set encrypted_seed to the value
			b64Seed = localStorage.getItem(KEY_BASE64_SEED);
			if (b64Seed) {
				encrypted_seed = b64Seed;
			}

			isWasmLoaded = true;
		} catch (e) {
			error = 'Failed to load WASM. Please refresh the page and try again.';
		}
	});

	/**
	 * Handles form submission
	 */
	async function handleSubmit() {
		if (!isWasmLoaded) {
			error = 'WASM is not initialized. Please try again.';
			return;
		}

		if (username.length < 8 || password.length < 8) {
			error = 'Username and password must be at least 8 characters long.';
			return;
		}
		error = '';

		const credentials = {
			username,
			password,
			// if encrypted_seed is not null, decode from base64 accordingly before use in creating wallet
			encrypted_seed: encrypted_seed
				? new Uint8Array(
						atob(encrypted_seed)
							.split('')
							.map((c) => c.charCodeAt(0))
					)
				: null
		};

		try {
			wallet = new WasmWallet(credentials);
			// save to local storage
			let encrSeed = wallet.encryptedSeed();
			let seed = new Uint8Array(encrSeed);
			b64Seed = btoa(String.fromCharCode(...seed));
			localStorage.setItem(KEY_BASE64_SEED, b64Seed);

			handleWalletCreated(wallet);
		} catch (e) {
			error = 'Failed to create wallet. Please check your inputs and try again.';
		}
	}
</script>

<div class="max-w-md mx-auto mt-8">
	<h1 class="text-3xl font-bold mb-6">Wallet</h1>

	<form on:submit|preventDefault={handleSubmit} class="space-y-4">
		<div>
			<label for="username" class="block text-sm font-medium text-gray-700">Username</label>
			<input
				type="text"
				id="username"
				bind:value={username}
				class="mt-1 block w-full rounded-md border border-sky-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50 bg-sky-50 p-2"
				required
				minlength="8"
			/>
		</div>

		<div>
			<label for="password" class="block text-sm font-medium text-gray-700">Password</label>
			<input
				type="password"
				id="password"
				bind:value={password}
				class="mt-1 block w-full rounded-md border border-sky-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50 bg-sky-50 p-2"
				required
				minlength="8"
			/>
		</div>

		<div>
			<label for="encrypted_seed" class="block text-sm font-medium text-gray-700"
				>Encrypted Seed (optional)</label
			>
			<input
				type="text"
				id="encrypted_seed"
				bind:value={encrypted_seed}
				class="mt-1 block w-full rounded-md border border-sky-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50 bg-sky-50 p-2"
			/>
		</div>

		<div class="flex justify-end">
			<button
				type="submit"
				class="py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white
					{isWasmLoaded
					? 'bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500'
					: 'bg-gray-400 cursor-not-allowed'}"
				disabled={!isWasmLoaded}
			>
				Unlock
			</button>
		</div>
	</form>

	{#if error}
		<p class="mt-2 text-sm text-red-600">{error}</p>
	{/if}
</div>
