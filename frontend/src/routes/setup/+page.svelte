<script lang="ts">
	import { onMount, getContext } from 'svelte';
	import { goto } from '$app/navigation';
	import {
		Wand2,
		FileCode,
		Folder,
		Cpu,
		Zap,
		Copy,
		Check,
		RefreshCw,
		CheckCircle2,
		AlertCircle,
		HardDrive,
		Terminal,
		Sparkles,
		Network
	} from '@lucide/svelte';
	import { copyToClipboard as copyClipboardUtil } from '$lib/utils/clipboard';
	import CodeBlock from '$lib/components/CodeBlock.svelte';

	interface ConfigStatus {
		is_valid: boolean;
		has_models: boolean;
		has_runtimes: boolean;
		has_hardware: boolean;
		created_auto_file: boolean;
		is_docker?: boolean;
		loaded_path: string | null;
		expected_path: string;
		models_dir: string | null;
		runtimes_dir: string | null;
		hardware: string;
		host: string;
		port: number;
		error_message: string | null;
		example_yaml: string;
		cli_command_example: string;
	}

	interface HardwareReport {
		os: string;
		arch: string;
		available_accelerations: string[];
		recommended_acceleration: string;
		detected_gpus: string[];
	}

	let activeTab = $state<'wizard' | 'manual'>('wizard');
	let configStatus = $state<ConfigStatus | null>(null);
	let hardwareReport = $state<HardwareReport | null>(null);
	let loading = $state(true);
	let saving = $state(false);
	let saveError = $state<string | null>(null);
	let rechecking = $state(false);

	// Wizard Form State
	let modelsDir = $state('');
	let runtimesDir = $state('');
	let selectedHardware = $state('auto');
	let host = $state('0.0.0.0');
	let port = $state(8080);

	let isStorageConfigured = $derived(
		!!configStatus?.is_docker || (!!configStatus?.has_models && !!configStatus?.has_runtimes)
	);

	// Copy feedback states
	let copiedPath = $state(false);
	let copiedCli = $state(false);

	const checkConfigFromLayout = getContext<() => Promise<void>>('checkConfigStatus');

	async function fetchConfigData() {
		try {
			rechecking = true;
			const [statusRes, hwRes] = await Promise.all([
				fetch('/api/config/status'),
				fetch('/api/hardware/detect')
			]);

			if (statusRes.ok) {
				const status: ConfigStatus = await statusRes.json();
				configStatus = status;

				if (status.is_valid) {
					goto('/chat');
					return;
				}

				if (!modelsDir && status.models_dir) {
					modelsDir = status.models_dir;
				}
				if (!runtimesDir && status.runtimes_dir) {
					runtimesDir = status.runtimes_dir;
				}
				if (status.hardware && status.hardware !== 'auto') {
					selectedHardware = status.hardware;
				}
				if (status.host) {
					host = status.host;
				}
				if (status.port) {
					port = status.port;
				}
			}

			if (hwRes.ok) {
				const hw: HardwareReport = await hwRes.json();
				hardwareReport = hw;
			}
		} catch (err) {
			console.error('Failed to load setup configuration:', err);
		} finally {
			loading = false;
			rechecking = false;
		}
	}

	async function saveSetup() {
		const targetModels = configStatus?.is_docker
			? configStatus.models_dir || '/app/models'
			: modelsDir.trim() || configStatus?.models_dir || '';
		const targetRuntimes = configStatus?.is_docker
			? configStatus.runtimes_dir || '/app/runtimes'
			: runtimesDir.trim() || configStatus?.runtimes_dir || '';
		const targetHost = configStatus?.is_docker ? '0.0.0.0' : host.trim();
		const parsedPort = configStatus?.is_docker ? configStatus.port || 8080 : Number(port);

		if (!targetModels) {
			saveError = 'Please specify a valid Models directory.';
			return;
		}
		if (!targetRuntimes) {
			saveError = 'Please specify a valid Runtimes directory.';
			return;
		}
		if (!targetHost) {
			saveError = 'Please specify a valid Host address.';
			return;
		}
		if (isNaN(parsedPort) || parsedPort < 1 || parsedPort > 65535) {
			saveError = 'Please specify a valid Port number (1-65535).';
			return;
		}

		try {
			saving = true;
			saveError = null;

			const res = await fetch('/api/config/setup', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					models: targetModels,
					runtimes: targetRuntimes,
					hardware: selectedHardware,
					host: targetHost,
					port: parsedPort
				})
			});

			if (!res.ok) {
				const msg = await res.text();
				throw new Error(msg || 'Failed to save configuration');
			}

			if (checkConfigFromLayout) {
				await checkConfigFromLayout();
			}

			goto('/chat');
		} catch (err: any) {
			saveError = err?.message || 'An error occurred while saving configuration.';
		} finally {
			saving = false;
		}
	}

	async function copyToClipboard(text: string, kind: 'path' | 'cli') {
		const ok = await copyClipboardUtil(text);
		if (ok) {
			if (kind === 'path') {
				copiedPath = true;
				setTimeout(() => (copiedPath = false), 2000);
			} else {
				copiedCli = true;
				setTimeout(() => (copiedCli = false), 2000);
			}
		}
	}

	function getHardwareName(id?: string | null): string {
		switch (id?.toLowerCase()) {
			case 'cuda':
				return 'NVIDIA CUDA';
			case 'rocm':
				return 'AMD ROCm';
			case 'vulkan':
				return 'Vulkan';
			case 'cpu':
				return 'CPU (AVX2)';
			case 'auto':
				return 'Auto Detect';
			default:
				return id ? id.toUpperCase() : 'Unknown';
		}
	}

	const hardwareOptions = [
		{
			id: 'auto',
			title: 'Auto Detect',
			desc: 'Best available engine for your detected hardware'
		},
		{
			id: 'cpu',
			title: 'CPU Only',
			desc: 'Universal inference with AVX2 optimization'
		},
		{
			id: 'cuda',
			title: 'NVIDIA GPU (CUDA)',
			desc: 'Hardware acceleration for GeForce, RTX and Tesla'
		},
		{
			id: 'vulkan',
			title: 'Vulkan',
			desc: 'Cross-vendor acceleration for modern GPUs'
		},
		{
			id: 'rocm',
			title: 'AMD GPU (ROCm)',
			desc: 'Hardware acceleration for supported AMD GPUs'
		}
	];

	onMount(() => {
		fetchConfigData();
	});
</script>

<svelte:head>
	<title>Monolai - Setup</title>
</svelte:head>

<!-- Full page wrapper with natural page scroll (no scroll inside the card) -->
<div
	class="min-h-screen w-full flex items-center justify-center p-4 sm:p-6 md:p-8 box-border bg-[var(--bg-primary)]"
>
	{#if loading && !configStatus}
		<div
			class="flex flex-col items-center justify-center gap-3 p-8 text-[var(--text-muted)] text-sm"
		>
			<div
				class="w-8 h-8 rounded-full border-2 border-[var(--border-color)] border-t-[var(--primary)] animate-spin"
			></div>
			<p class="m-0 text-xs">Checking system configuration...</p>
		</div>
	{:else if configStatus}
		<!-- Generous, card with natural height (no inner scrollbar) -->
		<div
			class="bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-2xl max-w-[700px] w-full shadow-2xl flex flex-col my-auto"
		>
			<!-- Header -->
			<header class="p-6 pb-5 border-b border-[var(--border-color)] flex flex-col gap-4">
				<div class="flex items-center justify-between gap-4">
					<div class="flex items-center gap-3.5">
						<div
							class="w-11 h-11 rounded-xl bg-[var(--primary)]/10 text-[var(--primary)] border border-[var(--primary)]/20 flex items-center justify-center shrink-0"
						>
							<Sparkles size={22} />
						</div>
						<div>
							<h1 class="m-0 text-lg font-bold text-[var(--text-primary)]">
								{isStorageConfigured ? 'Hardware Acceleration' : 'Monolai Setup'}
							</h1>
							<p class="m-0 text-xs text-[var(--text-muted)] mt-0.5">
								{isStorageConfigured
									? 'Select your target hardware acceleration to start running local AI models.'
									: 'Configure storage paths and acceleration to start running local AI models.'}
							</p>
						</div>
					</div>

					<button
						onclick={fetchConfigData}
						disabled={rechecking}
						class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-[var(--border-color)] bg-white/5 hover:bg-white/10 text-xs text-[var(--text-secondary)] font-medium transition cursor-pointer"
						title="Recheck configuration status"
					>
						<RefreshCw size={12} class={rechecking ? 'animate-spin' : ''} />
						<span>Refresh</span>
					</button>
				</div>

				{#if !isStorageConfigured}
					<!-- Setup Tabs -->
					<div
						class="flex items-center gap-1.5 p-1 rounded-xl bg-black/20 border border-[var(--border-color)]"
					>
						<button
							onclick={() => (activeTab = 'wizard')}
							class="flex-1 flex items-center justify-center gap-2 py-1.5 px-3 rounded-lg text-xs font-semibold transition cursor-pointer {activeTab ===
							'wizard'
								? 'bg-[var(--primary)] text-white shadow-sm'
								: 'text-[var(--text-muted)] hover:text-[var(--text-primary)]'}"
						>
							<Wand2 size={14} />
							<span>Interactive Setup</span>
						</button>
						<button
							onclick={() => (activeTab = 'manual')}
							class="flex-1 flex items-center justify-center gap-2 py-1.5 px-3 rounded-lg text-xs font-semibold transition cursor-pointer {activeTab ===
							'manual'
								? 'bg-[var(--primary)] text-white shadow-sm'
								: 'text-[var(--text-muted)] hover:text-[var(--text-primary)]'}"
						>
							<FileCode size={14} />
							<span>Manual Configuration</span>
						</button>
					</div>
				{/if}
			</header>

			<!-- Content Area -->
			<div class="p-6 flex flex-col gap-5">
				{#if saveError}
					<div
						class="flex items-center gap-2.5 p-3 rounded-xl bg-red-500/10 border border-red-500/25 text-red-400 text-xs"
					>
						<AlertCircle size={16} class="shrink-0" />
						<span>{saveError}</span>
					</div>
				{/if}

				{#if activeTab === 'wizard'}
					<form
						onsubmit={(e) => {
							e.preventDefault();
							saveSetup();
						}}
						class="flex flex-col gap-5"
					>
						{#if !isStorageConfigured}
							<!-- Step 1: Models Directory -->
							<div
								class="flex flex-col gap-2 p-4 rounded-xl border border-[var(--border-color)] bg-white/[0.02]"
							>
								<div class="flex items-center justify-between">
									<label
										for="models-dir-input"
										class="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-[var(--text-secondary)]"
									>
										<HardDrive size={15} class="text-[var(--primary)]" />
										<span>1. Models Directory</span>
									</label>
									<button
										type="button"
										onclick={() => {
											if (configStatus?.models_dir) modelsDir = configStatus.models_dir;
										}}
										class="text-[11px] text-[var(--text-muted)] hover:text-[var(--primary)] transition cursor-pointer bg-transparent border-0 p-0"
									>
										Reset to default
									</button>
								</div>
								<input
									id="models-dir-input"
									type="text"
									bind:value={modelsDir}
									placeholder="e.g. ~/.local/share/monolai/models"
									class="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-black/30 text-xs text-[var(--text-primary)] focus:outline-none focus:border-[var(--primary)] font-mono transition"
								/>
							</div>

							<!-- Step 2: Runtimes Directory -->
							<div
								class="flex flex-col gap-2 p-4 rounded-xl border border-[var(--border-color)] bg-white/[0.02]"
							>
								<div class="flex items-center justify-between">
									<label
										for="runtimes-dir-input"
										class="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-[var(--text-secondary)]"
									>
										<Folder size={15} class="text-[var(--primary)]" />
										<span>2. Runtimes Directory</span>
									</label>
									<button
										type="button"
										onclick={() => {
											if (configStatus?.runtimes_dir) runtimesDir = configStatus.runtimes_dir;
										}}
										class="text-[11px] text-[var(--text-muted)] hover:text-[var(--primary)] transition cursor-pointer bg-transparent border-0 p-0"
									>
										Reset to default
									</button>
								</div>
								<input
									id="runtimes-dir-input"
									type="text"
									bind:value={runtimesDir}
									placeholder="e.g. ~/.local/share/monolai/runtimes"
									class="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-black/30 text-xs text-[var(--text-primary)] focus:outline-none focus:border-[var(--primary)] font-mono transition"
								/>
							</div>
						{:else}
							<!-- Storage Volumes Detected Card -->
							<div
								class="flex items-center gap-3 p-3.5 rounded-xl border border-[var(--border-color)] bg-white/[0.02] text-xs"
							>
								<div
									class="w-8 h-8 rounded-lg bg-[var(--primary)]/10 text-[var(--primary)] border border-[var(--primary)]/20 flex items-center justify-center shrink-0"
								>
									<CheckCircle2 size={16} />
								</div>
								<div class="flex flex-col gap-0.5 min-w-0">
									<div class="flex items-center gap-2">
										<span class="font-semibold text-[var(--text-primary)]">
											{configStatus.is_docker
												? 'Docker Container Volumes'
												: 'Storage Volumes Configured'}
										</span>
										{#if configStatus.is_docker}
											<span
												class="px-1.5 py-0.2 rounded text-[10px] font-semibold bg-sky-500/10 text-sky-400 border border-sky-500/25"
											>
												Fixed Mounts
											</span>
										{/if}
									</div>
									<ul class="text-[11px] text-[var(--text-muted)] font-mono truncate">
										<li>Models: {configStatus.models_dir || '/app/models'}</li>
										<li>Runtimes: {configStatus.runtimes_dir || '/app/runtimes'}</li>
									</ul>
								</div>
							</div>
						{/if}

						<!-- Hardware Acceleration -->
						<div
							class="flex flex-col gap-3 p-4 rounded-xl border border-[var(--border-color)] bg-white/[0.02]"
						>
							<div class="flex items-center justify-between">
								<span
									class="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-[var(--text-secondary)]"
								>
									<Zap size={15} class="text-[var(--primary)]" />
									<span
										>{isStorageConfigured
											? 'Hardware Acceleration'
											: '3. Hardware Acceleration'}</span
									>
								</span>
								{#if hardwareReport}
									<span class="text-[11px] text-emerald-400 font-mono flex items-center gap-1.5">
										<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
										<span>{getHardwareName(hardwareReport.recommended_acceleration)} Detected</span>
									</span>
								{/if}
							</div>

							{#if hardwareReport}
								<div
									class="flex flex-col sm:flex-row sm:items-center justify-between gap-2.5 p-3 rounded-xl bg-black/25 border border-[var(--border-color)] text-xs"
								>
									<div class="flex items-center gap-2.5 min-w-0">
										<div
											class="w-8 h-8 rounded-lg bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 flex items-center justify-center shrink-0"
										>
											<CheckCircle2 size={16} />
										</div>
										<div class="flex flex-col min-w-0">
											<div class="flex items-center gap-2 flex-wrap">
												<span class="text-[var(--text-primary)] font-semibold text-xs">
													Detected Acceleration:
												</span>
												<span
													class="px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider bg-emerald-500/20 text-emerald-400 border border-emerald-500/30"
												>
													{getHardwareName(hardwareReport.recommended_acceleration)}
												</span>
												{#if hardwareReport.detected_gpus.length > 0}
													<span class="text-[11px] text-[var(--text-muted)] font-mono truncate">
														• {hardwareReport.detected_gpus.join(', ')}
													</span>
												{/if}
											</div>
											<span class="text-[11px] text-[var(--text-muted)] mt-0.5">
												Supported on host: {hardwareReport.available_accelerations
													.map(getHardwareName)
													.join(', ')} ({hardwareReport.os}
												{hardwareReport.arch})
											</span>
										</div>
									</div>
								</div>
							{/if}

							<div class="grid grid-cols-1 sm:grid-cols-2 gap-2 mt-1">
								{#each hardwareOptions as opt}
									{@const isSelected = selectedHardware === opt.id}
									{@const isSupported =
										opt.id === 'auto' ||
										(hardwareReport && hardwareReport.available_accelerations.includes(opt.id))}
									{@const isRecommended =
										opt.id === 'auto' ||
										(hardwareReport && hardwareReport.recommended_acceleration === opt.id)}
									<button
										type="button"
										onclick={() => (selectedHardware = opt.id)}
										class="p-2.5 rounded-xl border text-left flex flex-col gap-1 transition cursor-pointer {isSelected
											? 'border-[var(--primary)] bg-[var(--primary)]/10 text-[var(--text-primary)] ring-1 ring-[var(--primary)]'
											: 'border-[var(--border-color)] bg-white/[0.02] hover:bg-white/[0.05] text-[var(--text-muted)]'}"
									>
										<div class="flex items-center justify-between gap-1.5 flex-wrap">
											<span class="font-semibold text-xs text-[var(--text-primary)]">
												{opt.title}
												{#if opt.id === 'auto' && hardwareReport}
													<span class="text-[var(--text-muted)] font-normal text-[11px]">
														({getHardwareName(hardwareReport.recommended_acceleration)})
													</span>
												{/if}
											</span>
											<div class="flex items-center gap-1">
												{#if isRecommended}
													<span
														class="px-1.5 py-0.2 rounded text-[9px] font-bold uppercase bg-emerald-500/20 text-emerald-400 border border-emerald-500/30"
													>
														Recommended
													</span>
												{:else if isSupported}
													<span
														class="px-1.5 py-0.2 rounded text-[9px] font-medium uppercase bg-white/10 text-[var(--text-secondary)] border border-white/15"
													>
														Available
													</span>
												{:else if hardwareReport}
													<span
														class="px-1.5 py-0.2 rounded text-[9px] font-medium uppercase bg-red-500/10 text-red-400/80 border border-red-500/20"
													>
														Not Detected
													</span>
												{/if}
											</div>
										</div>
										<p class="m-0 text-[11px] leading-tight text-[var(--text-muted)]">
											{opt.desc}
										</p>
									</button>
								{/each}
							</div>
						</div>

						<!-- Network Configuration -->
						<div
							class="flex flex-col gap-3 p-4 rounded-xl border border-[var(--border-color)] bg-white/[0.02]"
						>
							<div class="flex items-center justify-between">
								<span
									class="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-[var(--text-secondary)]"
								>
									<Network size={15} class="text-[var(--primary)]" />
									<span>{isStorageConfigured ? 'Network & Port' : '4. Network Configuration'}</span>
								</span>
								<span class="text-[11px] text-[var(--text-muted)] font-mono">
									Default: 0.0.0.0:8080
								</span>
							</div>

							<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
								<div class="flex flex-col gap-1.5">
									<div class="flex items-center justify-between">
										<label
											for="host-input"
											class="text-xs font-semibold text-[var(--text-secondary)]"
										>
											Host Address
										</label>
										{#if configStatus?.is_docker}
											<span class="text-[10px] text-sky-400 font-medium">Locked in Docker</span>
										{/if}
									</div>
									<input
										id="host-input"
										type="text"
										bind:value={host}
										disabled={configStatus?.is_docker}
										placeholder="0.0.0.0"
										class="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-black/30 text-xs text-[var(--text-primary)] focus:outline-none focus:border-[var(--primary)] font-mono transition disabled:opacity-50 disabled:cursor-not-allowed"
									/>
								</div>

								<div class="flex flex-col gap-1.5">
									<div class="flex items-center justify-between">
										<label
											for="port-input"
											class="text-xs font-semibold text-[var(--text-secondary)]"
										>
											HTTP Port
										</label>
										{#if configStatus?.is_docker}
											<span class="text-[10px] text-sky-400 font-medium">Locked in Docker</span>
										{/if}
									</div>
									<input
										id="port-input"
										type="number"
										min="1"
										max="65535"
										bind:value={port}
										disabled={configStatus?.is_docker}
										placeholder="8080"
										class="w-full px-3 py-2 rounded-lg border border-[var(--border-color)] bg-black/30 text-xs text-[var(--text-primary)] focus:outline-none focus:border-[var(--primary)] font-mono transition disabled:opacity-50 disabled:cursor-not-allowed"
									/>
								</div>
							</div>
						</div>

						<!-- Target Configuration Path Info -->
						<div
							class="flex items-center justify-between p-3 rounded-xl border border-[var(--border-color)] bg-white/[0.02] text-xs"
						>
							<div class="flex items-center gap-2 truncate text-[var(--text-muted)]">
								<FileCode size={14} class="text-[var(--primary)] shrink-0" />
								<span class="text-[var(--text-secondary)] font-medium"
									>Config will be written to:</span
								>
								<span class="text-[var(--text-primary)] font-mono truncate"
									>{configStatus.expected_path}</span
								>
							</div>
							<button
								type="button"
								onclick={() => copyToClipboard(configStatus!.expected_path, 'path')}
								class="px-2 py-0.5 rounded bg-white/5 hover:bg-white/10 text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition cursor-pointer shrink-0 ml-2 text-[11px]"
							>
								{copiedPath ? 'Copied' : 'Copy'}
							</button>
						</div>

						<!-- Action Button -->
						<div class="pt-1">
							<button
								type="submit"
								disabled={saving}
								class="w-full py-3 px-4 rounded-xl bg-[var(--primary)] hover:opacity-90 active:scale-[0.99] text-white font-bold text-xs flex items-center justify-center gap-2 transition cursor-pointer shadow-md disabled:opacity-50"
							>
								{#if saving}
									<div
										class="w-4 h-4 rounded-full border-2 border-white border-t-transparent animate-spin"
									></div>
									<span>Saving Configuration...</span>
								{:else}
									<Check size={15} />
									<span>
										{isStorageConfigured
											? 'Save Hardware & Continue'
											: 'Save Configuration & Start'}
									</span>
								{/if}
							</button>
						</div>
					</form>
				{:else}
					<!-- Manual Configuration Tab -->
					<div class="flex flex-col gap-4 text-xs text-[var(--text-secondary)]">
						<!-- Target File Path -->
						<div
							class="p-4 rounded-xl border border-[var(--border-color)] bg-white/[0.02] flex flex-col gap-2"
						>
							<span class="font-semibold text-[var(--text-primary)]"
								>1. Default Configuration File Location</span
							>
							<div
								class="flex items-center justify-between p-2.5 rounded-lg bg-black/40 border border-[var(--border-color)] font-mono text-xs"
							>
								<span class="text-[var(--primary)] break-all">{configStatus.expected_path}</span>
								<button
									onclick={() => copyToClipboard(configStatus!.expected_path, 'path')}
									class="px-2.5 py-1 rounded bg-white/10 hover:bg-white/15 text-[var(--text-primary)] flex items-center gap-1.5 transition cursor-pointer shrink-0 ml-2 text-xs"
								>
									{#if copiedPath}
										<Check size={12} class="text-emerald-400" />
										<span>Copied</span>
									{:else}
										<Copy size={12} />
										<span>Copy</span>
									{/if}
								</button>
							</div>
						</div>

						<!-- CLI Custom Path Argument -->
						<div
							class="p-4 rounded-xl border border-[var(--border-color)] bg-white/[0.02] flex flex-col gap-2"
						>
							<div class="flex items-center gap-2 font-semibold text-[var(--text-primary)]">
								<Terminal size={14} class="text-[var(--primary)]" />
								<span>2. Custom Config via CLI Argument</span>
							</div>
							<div
								class="flex items-center justify-between p-2.5 rounded-lg bg-black/40 border border-[var(--border-color)] font-mono text-xs"
							>
								<span class="text-[var(--text-primary)] break-all"
									>{configStatus.cli_command_example}</span
								>
								<button
									onclick={() => copyToClipboard(configStatus!.cli_command_example, 'cli')}
									class="px-2.5 py-1 rounded bg-white/10 hover:bg-white/15 text-[var(--text-primary)] flex items-center gap-1.5 transition cursor-pointer shrink-0 ml-2 text-xs"
								>
									{#if copiedCli}
										<Check size={12} class="text-emerald-400" />
										<span>Copied</span>
									{:else}
										<Copy size={12} />
										<span>Copy</span>
									{/if}
								</button>
							</div>
						</div>

						<!-- Example YAML Content using CodeBlock component -->
						<div
							class="p-4 rounded-xl border border-[var(--border-color)] bg-white/[0.02] flex flex-col gap-2.5"
						>
							<div class="flex items-center justify-between">
								<span class="font-semibold text-[var(--text-primary)]"
									>3. Valid config.yaml Example</span
								>
								{#if hardwareReport}
									<span class="text-[11px] text-[var(--text-muted)] flex items-center gap-1.5">
										<span>Detected:</span>
										<strong class="text-emerald-400 font-semibold"
											>{getHardwareName(hardwareReport.recommended_acceleration)}</strong
										>
									</span>
								{/if}
							</div>
							<CodeBlock code={configStatus.example_yaml} lang="yaml" />
						</div>

						<div class="pt-1 flex justify-end">
							<button
								onclick={fetchConfigData}
								disabled={rechecking}
								class="py-2 px-3.5 rounded-xl bg-[var(--primary)] hover:opacity-90 text-white font-semibold text-xs flex items-center gap-1.5 transition cursor-pointer shadow-sm disabled:opacity-50"
							>
								<RefreshCw size={13} class={rechecking ? 'animate-spin' : ''} />
								<span>Recheck Configuration</span>
							</button>
						</div>
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>
