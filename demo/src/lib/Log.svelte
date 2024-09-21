<script>
	import { default as wasm, ProvenanceLogBuilder } from 'bestsign-bindings';
	import { onMount } from 'svelte';

	/**
	 * Gets the Multikey. Must be set to a wallet that provides this function.
	 * @type {Function}
	 */
	export let get_key;

	/**
	 * Proves the Multikey over given data. Must be set to a wallet that provides this function.
	 * @type {Function}
	 */
	export let prove;

	/** @type {ProvenanceLogBuilder | null} */
	let logBuilder = null;

	/** @type {string} */
	let lockScript = `check_signature("/recoverykey", "/entry/") ||
check_signature("/pubkey", "/entry/") ||
check_preimage("/hash")`;

	/** @type {string} */
	let unlockScript = `push("/entry/");
push("/entry/proof");`;

	/** @type {string} */
	let vladKey = '/vlad/key';

	/** @type {string} */
	let entryKey = '/entrykey';

	/** @type {string} */
	let pubKey = '/pubkey';

	/** @type {string} */
	let result = '';

	onMount(async () => {
		await wasm();
		initializeLogBuilder();
	});

	function initializeLogBuilder() {
		if (!get_key || !prove) {
			console.error('get_key and prove must be set to a wallet that provides these functions');
			return;
		}

		logBuilder = new ProvenanceLogBuilder(lockScript, unlockScript, get_key, prove);
	}

	function updateScripts() {
		if (logBuilder) {
			logBuilder.set_entry_lock_script(lockScript);
			logBuilder.set_entry_unlock_script(unlockScript);
			result = 'Scripts updated successfully';
		}
	}

	async function createLog() {
		if (!logBuilder) {
			result = 'Error: ProvenanceLogBuilder not initialized';
			return;
		}

		try {
			const log = logBuilder.create();
			result = `Log created successfully. First lock: ${log.first_lock}`;
			console.log('Created log:', log);
		} catch (error) {
			console.error('Error creating log:', error);
			result = `Error creating log: ${error}`;
		}
	}
</script>

<div class="p-6 max-w-2xl mx-auto">
	<h1 class="text-3xl font-bold underline mb-6">Create Provenance Log</h1>

	<div class="space-y-6">
		<div>
			<label for="lockScript" class="block text-sm font-medium text-gray-700 mb-2"
				>Lock Script</label
			>
			<textarea
				id="lockScript"
				bind:value={lockScript}
				class="w-full p-2 font-mono text-sm bg-gray-100 border border-gray-300 rounded-md shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
				rows="3"
			></textarea>
		</div>

		<div>
			<label for="unlockScript" class="block text-sm font-medium text-gray-700 mb-2"
				>Unlock Script</label
			>
			<textarea
				id="unlockScript"
				bind:value={unlockScript}
				class="w-full p-2 font-mono text-sm bg-gray-100 border border-gray-300 rounded-md shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
				rows="3"
			></textarea>
		</div>

		<button
			on:click={updateScripts}
			class="w-full py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
		>
			Update Scripts
		</button>

		<div>
			<label for="vladKey" class="block text-sm font-medium text-gray-700 mb-2">VLAD Key</label>
			<input
				type="text"
				id="vladKey"
				bind:value={vladKey}
				class="w-full p-2 font-mono text-sm bg-gray-100 border border-gray-300 rounded-md shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
				placeholder="Enter VLAD key"
			/>
		</div>

		<div>
			<label for="entryKey" class="block text-sm font-medium text-gray-700 mb-2">Entry Key</label>
			<input
				type="text"
				id="entryKey"
				bind:value={entryKey}
				class="w-full p-2 font-mono text-sm bg-gray-100 border border-gray-300 rounded-md shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
				placeholder="Enter entry key"
			/>
		</div>

		<div>
			<label for="pubKey" class="block text-sm font-medium text-gray-700 mb-2">Public Key</label>
			<input
				type="text"
				id="pubKey"
				bind:value={pubKey}
				class="w-full p-2 font-mono text-sm bg-gray-100 border border-gray-300 rounded-md shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
				placeholder="Enter public key"
			/>
		</div>

		<button
			on:click={createLog}
			class="w-full py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
		>
			Create Log
		</button>
	</div>

	{#if result}
		<div class="mt-6 p-4 bg-gray-100 rounded-md">
			<h2 class="text-xl font-semibold mb-2">Result:</h2>
			<pre class="whitespace-pre-wrap">{result}</pre>
		</div>
	{/if}
</div>
