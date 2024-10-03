import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	optimizeDeps: {
		exclude: ['@peerpiper/peerpiper-browser']
	},
	// no strict fs server
	server: {
		fs: {
			strict: false
		}
	},
	hot: {
		preserveLocalState: true
	}
});
