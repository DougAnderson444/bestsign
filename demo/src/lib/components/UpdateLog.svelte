<script>
	import { onMount } from 'svelte';
	import { default as wasm, ProvenanceLog } from 'bestsign-bindings';
	import KeyValuePairInput from './KeyValuePairInput.svelte';

	/**
	 * The log cbor bytes
	 * @type {Uint8Array}
	 */
	export let log;
	export let get_key;
	export let prove;

	let keyValuePairs = [];
	let result = '';
	let displayData = null;

	onMount(async () => {
		await wasm();
		initializeLog();
	});

	function initializeLog() {
		if (!get_key || !prove || !log) {
			console.error('log, get_key and prove must be set to a wallet that provides these functions');
			return;
		}
		const provenanceLog = new ProvenanceLog(log, get_key, prove);
		displayData = provenanceLog.plog();
	}

	function handleKeyValuePairsUpdate(event) {
		keyValuePairs = event.detail;
	}

	async function updateLog() {
		// Implement log update logic here
		result = 'Log updated successfully';
	}
</script>

{#if displayData}
	<div class="p-6 max-w-2xl mx-auto">
		<h1 class="text-3xl font-bold underline mb-6">Update Provenance Log</h1>

		<div class="mb-6">
			<h2 class="text-xl font-semibold mb-2">Current Log Data:</h2>
			<pre class="whitespace-pre-wrap bg-gray-100 p-4 rounded-md">{JSON.stringify(
					displayData,
					null,
					2
				)}</pre>
		</div>

		<KeyValuePairInput {keyValuePairs} on:update={handleKeyValuePairsUpdate} />

		<div class="mt-6">
			<button
				on:click={updateLog}
				class="w-full py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
			>
				Update Log
			</button>
		</div>

		{#if result}
			<div class="mt-6 p-4 bg-gray-100 rounded-md">
				<h2 class="text-xl font-semibold mb-2">Result:</h2>
				<pre class="whitespace-pre-wrap">{result}</pre>
			</div>
		{/if}
	</div>
{:else}
	<div class="p-6 max-w-2xl mx-auto">
		<p>Loading...</p>
	</div>
{/if}

