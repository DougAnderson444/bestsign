import adapter from '@sveltejs/adapter-cloudflare';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		// adapter-auto only supports some environments, see https://kit.svelte.dev/docs/adapter-auto for a list.
		// If your environment is not supported, or you settled on a specific environment, switch out the adapter.
		// See https://kit.svelte.dev/docs/adapters for more information about adapters.
		adapter: adapter({
			// See below for an explanation of these options
			routes: {
				// defines routes that will invoke a function
				include: ['/*'],
				// defines routes that will not invoke a function
				exclude: ['<all>']
			}
		})
	},
	preprocess: vitePreprocess()
};

export default config;
