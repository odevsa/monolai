import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import pkg from './package.json' with { type: 'json' };

const currentDir = path.dirname(fileURLToPath(import.meta.url));

function resolveBackendTarget(): string {
	if (process.env.BACKEND_URL) {
		return process.env.BACKEND_URL;
	}

	let host = process.env.BACKEND_HOST || process.env.HOST;
	let port = process.env.BACKEND_PORT || process.env.PORT;

	if (!host || !port) {
		const candidatePaths = [
			process.env.MONOLAI_CONFIG_PATH,
			process.env.MONOLAI_DATA_DIR ? path.join(process.env.MONOLAI_DATA_DIR, 'config.yaml') : null,
			process.env.DATA_DIR ? path.join(process.env.DATA_DIR, 'config.yaml') : null,
			path.join(os.homedir(), '.config', 'monolai', 'config.yaml'),
			path.join(os.homedir(), 'Library', 'Application Support', 'monolai', 'config.yaml'),
			process.env.APPDATA ? path.join(process.env.APPDATA, 'monolai', 'config.yaml') : null,
			path.resolve(currentDir, '../config.yaml'),
			path.resolve(currentDir, 'config.yaml')
		].filter((p): p is string => Boolean(p));

		for (const configPath of candidatePaths) {
			try {
				if (fs.existsSync(configPath)) {
					const content = fs.readFileSync(configPath, 'utf8');
					if (!port) {
						const portMatch = content.match(/^[ \t]*port:[ \t]*([0-9]+)/m);
						if (portMatch) {
							port = portMatch[1];
						}
					}
					if (!host) {
						const hostMatch = content.match(/^[ \t]*host:[ \t]*([^\s#]+)/m);
						if (hostMatch) {
							host = hostMatch[1];
						}
					}
					if (port && host) break;
				}
			} catch {
				// Fallback to defaults
			}
		}
	}

	const resolvedPort = port || '8080';
	const resolvedHost = !host || host === '0.0.0.0' ? '127.0.0.1' : host;
	return `http://${resolvedHost}:${resolvedPort}`;
}

const backendTarget = resolveBackendTarget();

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	define: {
		'import.meta.env.PACKAGE_VERSION': JSON.stringify(pkg.version)
	},
	server: {
		fs: {
			allow: ['..']
		},
		proxy: {
			'/health': {
				target: backendTarget,
				changeOrigin: true
			},
			'/api': {
				target: backendTarget,
				changeOrigin: true
			},
			'/v1': {
				target: backendTarget,
				changeOrigin: true
			}
		}
	}
});
