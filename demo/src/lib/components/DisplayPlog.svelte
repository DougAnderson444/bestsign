<script>
	export let displayData;
</script>

{#if displayData}
	<div class="p-6 max-w-2xl mx-auto">
		<h1 class="text-3xl font-bold mb-6">Provenance Log</h1>

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
		<slot></slot>
	</div>
{:else}
	<div class="p-6 max-w-2xl mx-auto">
		<p>Loading...</p>
	</div>
{/if}
