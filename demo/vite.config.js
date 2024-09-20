import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	// no strict fs server
	server: {
		fs: {
			strict: false
		}
	}
});
