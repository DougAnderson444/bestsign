<script>
	import { default as wasm, WasmWallet } from 'multiwallet-bindings';
	import { onMount } from 'svelte';

	onMount(async () => {
		await wasm();

		let credentials = {
			username: 'username',
			password: 'password',
			encrypted_seed: null
		};

		let wallet = new WasmWallet(credentials);

		let keyArgs = {
			key: '/pubkey',
			codec: 'ed25519-priv',
			threshold: 1,
			limit: 1
		};

		let mk = wallet.get_mk(keyArgs);

		console.log({ mk });

		// create some data to sign, example
		let data = new Uint8Array([0x69, 0x42]);

		// sign the data
		let sig = wallet.prove(mk, data);

		console.log({ sig });
	});
</script>

<h1 class="text-3xl font-bold underline">Wallet</h1>
<p>Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation</p>
