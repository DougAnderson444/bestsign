<script>
	import { default as wasm, ProvenanceLogBuilder, ProvenanceLog } from 'bestsign-core-bindings';
	import { onMount, createEventDispatcher } from 'svelte';
	import ScriptEditor from './ScriptEditor.svelte';
	import Modal from './Modal.svelte';
	import KeyValuePairInput from './KeyValuePairInput.svelte';

	const dispatch = createEventDispatcher();

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

	let logBuilder = null;
	let lockScript = `check_signature("/recoverykey", "/entry/") ||
check_signature("/pubkey", "/entry/") ||
check_preimage("/hash")`;
	let unlockScript = `push("/entry/");
push("/entry/proof");`;
	let result = '';
	let showModal = false;
	let keyValuePairs = [];

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

	function handleScriptUpdate(event) {
		const { lockScript: newLockScript, unlockScript: newUnlockScript } = event.detail;
		lockScript = newLockScript;
		unlockScript = newUnlockScript;
		updateScripts();
	}

	function updateScripts() {
		if (logBuilder) {
			logBuilder.set_entry_lock_script(lockScript);
			logBuilder.set_entry_unlock_script(unlockScript);
			result = 'Scripts updated successfully';
		}
	}

	function handleKeyValuePairsUpdate(event) {
		keyValuePairs = event.detail;
	}

	async function createLog() {
		if (!logBuilder) {
			result = 'Error: ProvenanceLogBuilder not initialized';
			return;
		}

		try {
			for (const { key, value } of keyValuePairs) {
				logBuilder.add_string({ key, value });
			}

			const log = logBuilder.create();
			dispatch('logCreated', { log });
		} catch (error) {
			console.error('Error creating log:', error);
			result = `Error creating log: ${error}`;
		}
	}

	function toggleModal() {
		showModal = !showModal;
	}
</script>

<div class="p-6 max-w-2xl mx-auto">
	<h1 class="text-3xl font-bold underline mb-6">Create Provenance Log</h1>

	<button
		on:click={toggleModal}
		class="mb-4 py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
	>
		Advanced
	</button>

	{#if showModal}
		<Modal title="Advanced Settings" on:close={toggleModal}>
			<ScriptEditor
				{lockScript}
				{unlockScript}
				on:update={handleScriptUpdate}
				on:close={toggleModal}
			/>
		</Modal>
	{/if}

	<KeyValuePairInput {keyValuePairs} on:update={handleKeyValuePairsUpdate} />

	<div class="mt-6">
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
