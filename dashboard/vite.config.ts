import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		proxy: {
			'/api': 'http://localhost:3200',
			'/ws': {
				target: 'ws://localhost:3200',
				ws: true,
			},
		},
	},
});
