<script>
	import { onMount } from 'svelte';
	import { default as wasm, ProvenanceLog } from 'bestsign-bindings';
	import KeyValuePairInput from './KeyValuePairInput.svelte';
	import ScriptEditor from './ScriptEditor.svelte';
	import Modal from './Modal.svelte';
	import Header from './Header.svelte';

	/**
	 * The log cbor bytes
	 * @type {Uint8Array}
	 */
	export let log;
	/**
	 * Get the Multikey from the wallet
	 * @type {(key: string) => Uint8Array}
	 */
	export let get_key;
	/**
	 * Prove the Multikey over the data, from the wallet
	 * @type {(key: string, data: Uint8Array) => Uint8Array}
	 */
	export let prove;

	/** @type {Array<{ key: string, value: string }>} */
	let keyValuePairs = [];
	let result = '';
	/** @type {any} */
	let displayData = null;

	/** @type {string} - The inital key path to be used in the lock script */
	let keyPath = '/';

	let lockScript = `check_signature("/recoverykey", "/entry/") ||
check_signature("/pubkey", "/entry/") ||
check_preimage("/hash")`;

	let unlockScript = `push("/entry/");
push("/entry/proof");`;

	let showModal = false;

	/** @type {(evt: Event) => void} */
	let updateLog = (evt) => {
		console.error('updateLog function not initialized');
	};

	/** @type {() => void} */
	let handleScriptUpdate = () => {
		console.error('handleScriptUpdate function not initialized');
	};

	onMount(async () => {
		await wasm();
		initializeLog();
	});

	function initializeLog() {
		if (!get_key || !prove || !log) {
			console.error('log, get_key and prove must be set to a wallet that provides these functions');
			return;
		}
		const logUpdater = new ProvenanceLog(log, unlockScript, get_key, prove);
		displayData = logUpdater.plog();

		updateLog = () => {
			try {
				for (const { key, value } of keyValuePairs) {
					logUpdater.add_string({ key, value });
				}

				logUpdater.update();
				displayData = logUpdater.plog();
				result = `Log updated successfully ${displayData}`;
			} catch (error) {
				console.error('Error updating log:', error);
				result = `Error updating log: ${error}`;
			}
		};

		handleScriptUpdate = (event) => {
			const {
				lock: { keyPath: key_path, script: newLockScript },
				unlockScript: newUnlockScript
			} = event.detail;

			keyPath = key_path;
			lockScript = newLockScript;
			unlockScript = newUnlockScript;

			try {
				logUpdater.add_lock_script(keyPath, lockScript);
			} catch (error) {
				console.error('Error updating lock script:', error);
				result = `Error updating lock script: ${error}`;
			}
			logUpdater.set_unlock(unlockScript);
			result = 'Scripts updated successfully';
		};
	}

	/**
	 * @param {CustomEvent<{ key: string, value: string }[]>} event
	 */
	function handleKeyValuePairsUpdate(event) {
		keyValuePairs = event.detail;
	}
</script>

<Header />

{#if displayData}
	<div class="p-6 max-w-2xl mx-auto">
		<h1 class="text-3xl font-bold mb-6">Your Provenance Log</h1>

		<button
			on:click={(_) => (showModal = !showModal)}
			class="mb-4 py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
		>
			Edit Lock Scripts
		</button>

		{#if showModal}
			<Modal title="Advanced Settings" on:close={() => (showModal = false)}>
				<ScriptEditor
					{keyPath}
					{lockScript}
					{unlockScript}
					on:update={handleScriptUpdate}
					on:close={() => (showModal = false)}
				/>
			</Modal>
		{/if}

		<div class="mb-6">
			<h2 class="text-xl font-semibold mb-2">Current Log Data:</h2>
			<div class="mb-6">
				<h2 class="text-xl font-semibold mb-2">Verifiable Long-Lived Address (VLAD):</h2>
				<pre
					class="whitespace-pre-wrap break-all text-sm bg-neutral-100 p-4 rounded-md
          ">{displayData.ReturnValue.vlad.encoded}</pre>
			</div>

			<div class="mb-6">
				<h2 class="text-xl font-semibold mb-2">Key-Value Pairs:</h2>
				<ul class="pl-5 mb-4">
					<!-- Sort the displayData.ReturnValue.kvp_data such that Multikey is shown first, then Str, then Cid -->
					{#each displayData.ReturnValue.kvp_data.sort((a, b) => {
						const getPriority = (item) => (item.Multikey ? 0 : item.Str ? 1 : item.Cid ? 2 : 3);
						return getPriority(a) - getPriority(b);
					}) as pair}
						<!-- Only if Multikey key_path is /pubkey --->
						{#if pair.Multikey && pair.Multikey.key_path === '/pubkey'}
							<li>{pair.Multikey.key_path}: {pair.Multikey.fingerprint}</li>
						{/if}
						{#if pair.Str}
							<li>{pair.Str.key_path}: {pair.Str.value}</li>
						{/if}
						<!-- Only if Cid key_path is not /vlad/cid --->
						{#if pair.Cid && pair.Cid.key_path !== '/vlad/cid'}
							<li>{pair.Cid.key_path}: {pair.Cid.encoded}</li>
						{/if}
					{/each}
				</ul>
			</div>
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
