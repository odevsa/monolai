<script lang="ts">
	import { AlertTriangle, Copy, Check, RefreshCw, FileText } from '@lucide/svelte';
	import { copyToClipboard as copyClipboardUtil } from '$lib/utils/clipboard';

	interface ConfigStatus {
		is_valid: boolean;
		loaded_path: string | null;
		expected_path: string;
		models_dir: string | null;
		host?: string;
		port?: number;
		error_message: string | null;
		example_yaml: string;
	}

	let {
		configStatus,
		onRecheck = () => {}
	}: {
		configStatus: ConfigStatus;
		onRecheck?: () => void;
	} = $props();

	let copiedPath = $state(false);
	let copiedYaml = $state(false);
	let rechecking = $state(false);

	async function copyToClipboard(text: string, isPath: boolean) {
		const ok = await copyClipboardUtil(text);
		if (ok) {
			if (isPath) {
				copiedPath = true;
				setTimeout(() => (copiedPath = false), 2000);
			} else {
				copiedYaml = true;
				setTimeout(() => (copiedYaml = false), 2000);
			}
		}
	}

	function handleRecheck() {
		rechecking = true;
		onRecheck();
		setTimeout(() => (rechecking = false), 600);
	}
</script>

<div
	class="fixed inset-0 bg-black/85 backdrop-blur-md z-[9999] flex items-center justify-center p-6 box-border"
>
	<div
		class="bg-[var(--bg-surface)] border border-[var(--border-hover)] rounded-2xl max-w-[680px] w-full max-h-[90vh] overflow-y-auto flex flex-col shadow-2xl"
	>
		<header class="flex items-center gap-4 p-6 border-b border-[var(--border-color)]">
			<div
				class="w-11.5 h-11.5 rounded-xl bg-amber-500/12 text-amber-500 border border-amber-500/30 flex items-center justify-center shrink-0"
			>
				<AlertTriangle size={24} />
			</div>
			<div>
				<h2 class="m-0 text-xl font-bold text-[var(--text-primary)]">
					Configuration File Required
				</h2>
				<p class="m-0 text-xs text-[var(--text-muted)] mt-1">
					Monolai could not locate your <code
						class="bg-white/10 px-1.5 py-0.5 rounded font-mono text-xs text-[var(--primary)]"
						>config.yaml</code
					> file.
				</p>
			</div>
		</header>

		<div class="p-6 flex flex-col gap-5">
			{#if configStatus.error_message}
				<div class="p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-red-500 text-xs">
					<strong>Status:</strong>
					{configStatus.error_message}
				</div>
			{/if}

			<p class="m-0 text-xs text-[var(--text-secondary)] leading-relaxed">
				To start Monolai, create a configuration file at the target path below or pass <code
					class="bg-white/10 px-1.5 py-0.5 rounded font-mono text-xs text-[var(--primary)]"
					>-c /path/to/config.yaml</code
				> when running the binary.
			</p>

			<!-- Target Path Card -->
			<div
				class="flex items-center justify-between gap-4 p-3.5 bg-black/20 border border-[var(--border-color)] rounded-xl"
			>
				<div class="flex flex-col gap-1 overflow-hidden">
					<span
						class="text-[0.7rem] uppercase tracking-wider text-[var(--text-muted)] font-semibold"
						>Expected Location</span
					>
					<code class="text-xs text-[var(--text-primary)] font-mono break-all"
						>{configStatus.expected_path}</code
					>
				</div>
				<button
					type="button"
					class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium bg-white/5 border border-[var(--border-color)] text-[var(--text-secondary)] rounded-lg cursor-pointer transition-colors shrink-0 hover:bg-white/10 hover:text-[var(--text-primary)]"
					onclick={() => copyToClipboard(configStatus.expected_path, true)}
				>
					{#if copiedPath}
						<Check size={14} class="text-emerald-400" />
						<span>Copied</span>
					{:else}
						<Copy size={14} />
						<span>Copy Path</span>
					{/if}
				</button>
			</div>

			<!-- Example YAML Card -->
			<div
				class="flex flex-col border border-[var(--border-color)] rounded-xl overflow-hidden bg-black/25"
			>
				<div
					class="flex items-center justify-between px-4 py-2.5 bg-white/5 border-b border-[var(--border-color)]"
				>
					<div class="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
						<FileText size={16} />
						<span>Example <code class="font-mono">config.yaml</code> template</span>
					</div>
					<button
						type="button"
						class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium bg-white/5 border border-[var(--border-color)] text-[var(--text-secondary)] rounded-lg cursor-pointer transition-colors shrink-0 hover:bg-white/10 hover:text-[var(--text-primary)]"
						onclick={() => copyToClipboard(configStatus.example_yaml, false)}
					>
						{#if copiedYaml}
							<Check size={14} class="text-emerald-400" />
							<span>Copied</span>
						{:else}
							<Copy size={14} />
							<span>Copy Template</span>
						{/if}
					</button>
				</div>
				<pre class="m-0 p-4 text-xs font-mono leading-relaxed text-sky-400 overflow-x-auto"><code
						>{configStatus.example_yaml}</code
					></pre>
			</div>

			<!-- Shell Quick Command -->
			<div
				class="flex flex-col gap-1.5 p-3.5 bg-black/30 border border-[var(--border-color)] rounded-xl"
			>
				<span class="text-[0.7rem] text-[var(--text-muted)]"
					>Quick Shell Command (Linux/macOS):</span
				>
				<code class="text-xs font-mono text-emerald-400">
					mkdir -p ~/.config/monolai && nano ~/.config/monolai/config.yaml
				</code>
			</div>
		</div>

		<footer class="p-6 border-t border-[var(--border-color)] flex items-center justify-end">
			<button
				type="button"
				class="inline-flex items-center gap-2 px-5 py-2.5 bg-[var(--primary)] text-white text-xs font-semibold rounded-lg border-0 cursor-pointer transition-colors hover:bg-[var(--primary-hover)] disabled:opacity-60 disabled:cursor-not-allowed"
				onclick={handleRecheck}
				disabled={rechecking}
			>
				<RefreshCw size={16} class={rechecking ? 'animate-spin' : ''} />
				<span>{rechecking ? 'Checking...' : 'Recheck Configuration'}</span>
			</button>
		</footer>
	</div>
</div>
