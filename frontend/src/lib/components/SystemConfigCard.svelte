<script lang="ts">
	import { onMount, getContext } from 'svelte';
	import {
		Settings2,
		HardDrive,
		Cpu,
		Zap,
		Copy,
		Check,
		Edit2,
		X,
		Save,
		RefreshCw,
		AlertCircle,
		FileCode,
		ShieldCheck,
		Network
	} from '@lucide/svelte';
	import { copyToClipboard as copyClipboardUtil } from '$lib/utils/clipboard';

	interface ConfigStatus {
		is_valid: boolean;
		has_models: boolean;
		has_runtimes: boolean;
		has_hardware: boolean;
		created_auto_file: boolean;
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

	const HARDWARE_TARGETS = [
		{
			id: 'auto',
			title: 'Auto (Recommended)',
			badge: 'AUTO',
			desc: 'Automatic selection based on detected GPUs'
		},
		{
			id: 'cuda',
			title: 'NVIDIA CUDA',
			badge: 'CUDA',
			desc: 'Hardware acceleration for modern NVIDIA GPUs'
		},
		{
			id: 'cpu',
			title: 'CPU Only',
			badge: 'CPU',
			desc: 'Standard CPU instructions (portable)'
		},
		{
			id: 'vulkan',
			title: 'Vulkan',
			badge: 'VULKAN',
			desc: 'Cross-vendor graphics acceleration'
		},
		{
			id: 'rocm',
			title: 'AMD ROCm',
			badge: 'ROCM',
			desc: 'Hardware acceleration for supported AMD GPUs'
		}
	];

	let configStatus = $state<ConfigStatus | null>(null);
	let hardwareReport = $state<HardwareReport | null>(null);
	let isLoading = $state(false);
	let isRefreshing = $state(false);
	let fetchError = $state<string | null>(null);

	// Edit mode state
	let isEditing = $state(false);
	let isSaving = $state(false);
	let saveSuccess = $state(false);
	let saveError = $state<string | null>(null);

	let editModels = $state('');
	let editRuntimes = $state('');
	let editHardware = $state('auto');
	let editHost = $state('0.0.0.0');
	let editPort = $state(8080);

	// Copy feedback state
	let copiedField = $state<string | null>(null);

	const checkConfigFromLayout = getContext<() => Promise<void>>('checkConfigStatus');

	function getHardwareMeta(id: string) {
		return (
			HARDWARE_TARGETS.find((h) => h.id.toLowerCase() === id.toLowerCase()) || {
				id,
				title: id.toUpperCase(),
				badge: id.toUpperCase(),
				desc: 'Configured hardware acceleration'
			}
		);
	}

	let configPath = $derived(
		configStatus ? configStatus.loaded_path || configStatus.expected_path : ''
	);
	let isLoaded = $derived(Boolean(configStatus?.loaded_path));
	let modelsPath = $derived(configStatus?.models_dir || 'Not configured');
	let runtimesPath = $derived(configStatus?.runtimes_dir || 'Not configured');
	let hardwareSetting = $derived(configStatus?.hardware || 'auto');
	let hardwareMeta = $derived(getHardwareMeta(hardwareSetting));
	let hostSetting = $derived(configStatus?.host || '0.0.0.0');
	let portSetting = $derived(configStatus?.port ?? 8080);

	async function loadConfigData(silent = false) {
		if (!silent && !configStatus) isLoading = true;
		else isRefreshing = true;
		fetchError = null;

		try {
			const [statusRes, hwRes] = await Promise.all([
				fetch('/api/config/status'),
				fetch('/api/hardware/detect')
			]);

			if (statusRes.ok) {
				const status: ConfigStatus = await statusRes.json();
				configStatus = status;
				editModels = status.models_dir || '';
				editRuntimes = status.runtimes_dir || '';
				editHardware = status.hardware || 'auto';
				editHost = status.host || '0.0.0.0';
				editPort = status.port ?? 8080;
			} else {
				fetchError = 'Failed to load configuration status from server';
			}

			if (hwRes.ok) {
				const hw: HardwareReport = await hwRes.json();
				hardwareReport = hw;
			}
		} catch (e: any) {
			fetchError = e.message || 'Error connecting to configuration API';
		} finally {
			isLoading = false;
			isRefreshing = false;
		}
	}

	function startEdit() {
		if (configStatus) {
			editModels = configStatus.models_dir || '';
			editRuntimes = configStatus.runtimes_dir || '';
			editHardware = configStatus.hardware || 'auto';
			editHost = configStatus.host || '0.0.0.0';
			editPort = configStatus.port ?? 8080;
		}
		saveError = null;
		saveSuccess = false;
		isEditing = true;
	}

	function cancelEdit() {
		if (configStatus) {
			editModels = configStatus.models_dir || '';
			editRuntimes = configStatus.runtimes_dir || '';
			editHardware = configStatus.hardware || 'auto';
			editHost = configStatus.host || '0.0.0.0';
			editPort = configStatus.port ?? 8080;
		}
		saveError = null;
		isEditing = false;
	}

	async function saveConfiguration() {
		if (!editModels.trim()) {
			saveError = 'Models directory path cannot be empty';
			return;
		}
		if (!editRuntimes.trim()) {
			saveError = 'Runtimes directory path cannot be empty';
			return;
		}
		if (!editHost.trim()) {
			saveError = 'Host address cannot be empty';
			return;
		}
		const parsedPort = Number(editPort);
		if (isNaN(parsedPort) || parsedPort < 1 || parsedPort > 65535) {
			saveError = 'Port must be a valid number between 1 and 65535';
			return;
		}

		isSaving = true;
		saveError = null;
		saveSuccess = false;

		try {
			const res = await fetch('/api/config/setup', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					models: editModels.trim(),
					runtimes: editRuntimes.trim(),
					hardware: editHardware,
					host: editHost.trim(),
					port: parsedPort
				})
			});

			if (!res.ok) {
				const errText = await res.text();
				throw new Error(errText || 'Failed to save configuration');
			}

			saveSuccess = true;
			isEditing = false;
			await loadConfigData(true);

			if (checkConfigFromLayout) {
				await checkConfigFromLayout();
			}

			setTimeout(() => {
				saveSuccess = false;
			}, 3500);
		} catch (err: any) {
			saveError = err.message || 'Error saving configuration to disk';
		} finally {
			isSaving = false;
		}
	}

	async function handleCopy(text: string, field: string) {
		const ok = await copyClipboardUtil(text);
		if (ok) {
			copiedField = field;
			setTimeout(() => {
				if (copiedField === field) copiedField = null;
			}, 2000);
		}
	}

	onMount(() => {
		loadConfigData();
	});
</script>

<div class="flex flex-col gap-4">
	<!-- Section Header -->
	<div class="flex items-center justify-between gap-3 border-b border-[var(--border-color)] pb-3">
		<div>
			<h4 class="m-0 text-sm sm:text-base font-bold text-[var(--text-primary)] flex items-center gap-2 leading-none">
				<Settings2 size={18} class="text-[var(--primary)] shrink-0" />
				<span>Active Configuration & Storage</span>
			</h4>
			<p class="m-0 text-[0.725rem] text-[var(--text-muted)] mt-1.5 leading-relaxed">
				Disk configuration file, model storage, runtimes, and chosen hardware target.
			</p>
		</div>

		<div class="flex items-center gap-2 shrink-0">
			{#if !isEditing}
				<button
					type="button"
					onclick={() => loadConfigData(true)}
					disabled={isRefreshing || isLoading}
					class="w-7 h-7 flex items-center justify-center rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] hover:bg-[var(--bg-hover)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition cursor-pointer"
					title="Refresh configuration"
					aria-label="Refresh configuration"
				>
					<RefreshCw size={13} class={isRefreshing ? 'animate-spin text-[var(--primary)]' : ''} />
				</button>

				<button
					type="button"
					onclick={startEdit}
					disabled={isLoading || !configStatus}
					class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] hover:bg-[var(--bg-hover)] hover:border-[var(--primary)] text-xs text-[var(--text-primary)] font-medium transition cursor-pointer shadow-2xs"
				>
					<Edit2 size={13} class="text-[var(--primary)]" />
					<span>Edit</span>
				</button>
			{:else}
				<button
					type="button"
					onclick={cancelEdit}
					disabled={isSaving}
					class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg border border-[var(--border-color)] bg-[var(--bg-primary)] hover:bg-[var(--bg-hover)] text-xs text-[var(--text-muted)] hover:text-[var(--text-primary)] transition cursor-pointer"
				>
					<X size={13} />
					<span>Cancel</span>
				</button>
			{/if}
		</div>
	</div>

	<!-- Alert Messages -->
	{#if fetchError}
		<div
			class="flex items-center gap-2.5 bg-red-500/10 border border-red-500/30 text-red-500 px-3.5 py-2.5 rounded-xl text-xs"
		>
			<AlertCircle size={15} class="shrink-0" />
			<span>{fetchError}</span>
		</div>
	{/if}

	{#if saveError}
		<div
			class="flex items-center gap-2.5 bg-red-500/10 border border-red-500/30 text-red-500 px-3.5 py-2.5 rounded-xl text-xs"
		>
			<AlertCircle size={15} class="shrink-0" />
			<span>{saveError}</span>
		</div>
	{/if}

	{#if saveSuccess}
		<div
			class="flex items-center gap-2 bg-emerald-500/10 border border-emerald-500/30 text-emerald-500 px-3.5 py-2.5 rounded-xl text-xs font-semibold animate-in fade-in duration-200"
		>
			<ShieldCheck size={15} />
			<span>Configuration updated and saved to disk successfully!</span>
		</div>
	{/if}

	{#if isLoading && !configStatus}
		<div
			class="flex flex-col items-center justify-center gap-2.5 py-10 text-[var(--text-muted)] text-xs"
		>
			<RefreshCw size={20} class="animate-spin text-[var(--primary)]" />
			<span>Loading system configuration...</span>
		</div>
	{:else if configStatus}
		{#if !isEditing}
			<!-- View Mode: Elegant, Clean Rows Without Icon Frames -->
			<div class="grid grid-cols-1 gap-2.5">
				<!-- 1. Config File Item -->
				<div
					class="p-3 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl flex items-center justify-between gap-3 group hover:border-[var(--border-hover)] transition-all shadow-2xs"
				>
					<div class="flex items-start gap-2.5 min-w-0">
						<FileCode size={16} class="text-[var(--primary)] shrink-0 mt-0.5" />
						<div class="flex flex-col gap-0.5 min-w-0">
							<div class="flex items-center gap-2">
								<span class="text-xs font-semibold text-[var(--text-primary)]">Config File</span>
								<span
									class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-md text-[0.65rem] font-semibold border {isLoaded
										? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-500'
										: 'bg-amber-500/10 border-amber-500/30 text-amber-500'}"
								>
									<span
										class="w-1.5 h-1.5 rounded-full {isLoaded
											? 'bg-emerald-500 animate-pulse'
											: 'bg-amber-500'}"
									></span>
									{isLoaded ? 'Loaded' : 'Default Path'}
								</span>
							</div>
							<span
								class="text-[0.725rem] font-mono text-[var(--text-secondary)] truncate max-w-full"
								title={configPath}
							>
								{configPath}
							</span>
						</div>
					</div>

					<button
						type="button"
						onclick={() => handleCopy(configPath, 'config')}
						class="w-8 h-8 flex items-center justify-center rounded-lg border border-[var(--border-color)] bg-[var(--bg-surface)] hover:bg-[var(--bg-hover)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition cursor-pointer shrink-0 shadow-2xs"
						title="Copy config path"
						aria-label="Copy config path"
					>
						{#if copiedField === 'config'}
							<Check size={14} class="text-emerald-500" />
						{:else}
							<Copy size={14} />
						{/if}
					</button>
				</div>

				<!-- 2. Network & Port Item -->
				<div
					class="p-3 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl flex items-center justify-between gap-3 group hover:border-[var(--border-hover)] transition-all shadow-2xs"
				>
					<div class="flex items-start gap-2.5 min-w-0">
						<Network size={16} class="text-[var(--primary)] shrink-0 mt-0.5" />
						<div class="flex flex-col gap-0.5 min-w-0">
							<div class="flex items-center gap-2">
								<span class="text-xs font-semibold text-[var(--text-primary)]">Host & Port</span>
								<span
									class="px-1.5 py-0.5 rounded-md text-[0.65rem] font-medium bg-[var(--bg-surface)] text-[var(--text-muted)] border border-[var(--border-color)]"
								>
									HTTP Endpoint
								</span>
							</div>
							<span
								class="text-[0.725rem] font-mono text-[var(--text-secondary)] truncate max-w-full"
								title={`${hostSetting}:${portSetting}`}
							>
								{hostSetting}:{portSetting}
							</span>
						</div>
					</div>

					<button
						type="button"
						onclick={() => handleCopy(`${hostSetting}:${portSetting}`, 'network')}
						class="w-8 h-8 flex items-center justify-center rounded-lg border border-[var(--border-color)] bg-[var(--bg-surface)] hover:bg-[var(--bg-hover)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition cursor-pointer shrink-0 shadow-2xs"
						title="Copy host and port"
						aria-label="Copy host and port"
					>
						{#if copiedField === 'network'}
							<Check size={14} class="text-emerald-500" />
						{:else}
							<Copy size={14} />
						{/if}
					</button>
				</div>

				<!-- 3. Models Directory Item -->
				<div
					class="p-3 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl flex items-center justify-between gap-3 group hover:border-[var(--border-hover)] transition-all shadow-2xs"
				>
					<div class="flex items-start gap-2.5 min-w-0">
						<HardDrive size={16} class="text-[var(--primary)] shrink-0 mt-0.5" />
						<div class="flex flex-col gap-0.5 min-w-0">
							<div class="flex items-center gap-2">
								<span class="text-xs font-semibold text-[var(--text-primary)]"
									>Models Directory</span
								>
								<span
									class="px-1.5 py-0.5 rounded-md text-[0.65rem] font-medium bg-[var(--bg-surface)] text-[var(--text-muted)] border border-[var(--border-color)]"
								>
									Weights (.gguf, .safetensors)
								</span>
							</div>
							<span
								class="text-[0.725rem] font-mono text-[var(--text-secondary)] truncate max-w-full"
								title={modelsPath}
							>
								{modelsPath}
							</span>
						</div>
					</div>

					<button
						type="button"
						onclick={() => handleCopy(modelsPath, 'models')}
						class="w-8 h-8 flex items-center justify-center rounded-lg border border-[var(--border-color)] bg-[var(--bg-surface)] hover:bg-[var(--bg-hover)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition cursor-pointer shrink-0 shadow-2xs"
						title="Copy models directory path"
						aria-label="Copy models directory path"
					>
						{#if copiedField === 'models'}
							<Check size={14} class="text-emerald-500" />
						{:else}
							<Copy size={14} />
						{/if}
					</button>
				</div>

				<!-- 3. Runtimes Directory Item -->
				<div
					class="p-3 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl flex items-center justify-between gap-3 group hover:border-[var(--border-hover)] transition-all shadow-2xs"
				>
					<div class="flex items-start gap-2.5 min-w-0">
						<Cpu size={16} class="text-[var(--primary)] shrink-0 mt-0.5" />
						<div class="flex flex-col gap-0.5 min-w-0">
							<div class="flex items-center gap-2">
								<span class="text-xs font-semibold text-[var(--text-primary)]"
									>Runtimes Directory</span
								>
								<span
									class="px-1.5 py-0.5 rounded-md text-[0.65rem] font-medium bg-[var(--bg-surface)] text-[var(--text-muted)] border border-[var(--border-color)]"
								>
									Inference Engines
								</span>
							</div>
							<span
								class="text-[0.725rem] font-mono text-[var(--text-secondary)] truncate max-w-full"
								title={runtimesPath}
							>
								{runtimesPath}
							</span>
						</div>
					</div>

					<button
						type="button"
						onclick={() => handleCopy(runtimesPath, 'runtimes')}
						class="w-8 h-8 flex items-center justify-center rounded-lg border border-[var(--border-color)] bg-[var(--bg-surface)] hover:bg-[var(--bg-hover)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition cursor-pointer shrink-0 shadow-2xs"
						title="Copy runtimes directory path"
						aria-label="Copy runtimes directory path"
					>
						{#if copiedField === 'runtimes'}
							<Check size={14} class="text-emerald-500" />
						{:else}
							<Copy size={14} />
						{/if}
					</button>
				</div>

				<!-- 4. Chosen Hardware Acceleration Item -->
				<div
					class="p-3 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl flex items-center justify-between gap-3 group hover:border-[var(--border-hover)] transition-all shadow-2xs"
				>
					<div class="flex items-start gap-2.5 min-w-0">
						<Zap size={16} class="text-[var(--primary)] shrink-0 mt-0.5" />
						<div class="flex flex-col gap-0.5 min-w-0">
							<div class="flex items-center gap-2">
								<span class="text-xs font-semibold text-[var(--text-primary)]"
									>Hardware Acceleration</span
								>
								<span
									class="px-2 py-0.5 rounded-md text-[0.65rem] font-bold uppercase tracking-wider bg-[var(--primary-light)] text-[var(--primary)] border border-[var(--primary)]/30"
								>
									{hardwareMeta.badge}
								</span>
							</div>
							<div class="flex items-center gap-2 text-[0.725rem] text-[var(--text-secondary)]">
								<span class="font-medium text-[var(--text-primary)]">{hardwareMeta.title}</span>
								{#if hardwareReport && hardwareReport.detected_gpus.length > 0}
									<span class="text-[var(--text-muted)]">•</span>
									<span class="text-emerald-500 font-medium truncate" title={hardwareReport.detected_gpus.join(', ')}>
										{hardwareReport.detected_gpus[0]}
									</span>
								{/if}
							</div>
						</div>
					</div>

					<button
						type="button"
						onclick={() => handleCopy(hardwareSetting, 'hardware')}
						class="w-8 h-8 flex items-center justify-center rounded-lg border border-[var(--border-color)] bg-[var(--bg-surface)] hover:bg-[var(--bg-hover)] text-[var(--text-muted)] hover:text-[var(--text-primary)] transition cursor-pointer shrink-0 shadow-2xs"
						title="Copy hardware setting"
						aria-label="Copy hardware setting"
					>
						{#if copiedField === 'hardware'}
							<Check size={14} class="text-emerald-500" />
						{:else}
							<Copy size={14} />
						{/if}
					</button>
				</div>
			</div>
		{:else}
			<!-- Edit Mode: Form for Editing Paths & Hardware Acceleration -->
			<div
				class="p-4 bg-[var(--bg-primary)] border border-[var(--primary)]/40 rounded-xl flex flex-col gap-4 shadow-sm"
			>
				<div class="flex items-center justify-between border-b border-[var(--border-color)]/60 pb-2.5">
					<span class="text-xs font-bold text-[var(--text-primary)]">
						Modify Configuration & Network
					</span>
					<span class="text-[0.7rem] text-[var(--text-muted)]">
						Writes to <code class="font-mono text-[var(--primary)]">config.yaml</code>
					</span>
				</div>

				<!-- Host and Port Inputs -->
				<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
					<div class="flex flex-col gap-1.5">
						<label for="cfg-host-input" class="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
							<Network size={14} class="text-emerald-500" />
							<span>Host Address</span>
						</label>
						<input
							id="cfg-host-input"
							type="text"
							bind:value={editHost}
							placeholder="0.0.0.0"
							class="w-full px-3 py-2 bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg text-xs font-mono text-[var(--text-primary)] focus:outline-none focus:border-[var(--primary)] focus:ring-1 focus:ring-[var(--primary)]"
						/>
					</div>

					<div class="flex flex-col gap-1.5">
						<label for="cfg-port-input" class="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
							<span class="text-emerald-500 text-xs font-mono font-bold">#</span>
							<span>HTTP Port</span>
						</label>
						<input
							id="cfg-port-input"
							type="number"
							min="1"
							max="65535"
							bind:value={editPort}
							placeholder="8080"
							class="w-full px-3 py-2 bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg text-xs font-mono text-[var(--text-primary)] focus:outline-none focus:border-[var(--primary)] focus:ring-1 focus:ring-[var(--primary)]"
						/>
					</div>
				</div>

				<!-- Models Input -->
				<div class="flex flex-col gap-1.5">
					<label for="cfg-models-input" class="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
						<HardDrive size={14} class="text-blue-500" />
						<span>Models Directory</span>
					</label>
					<input
						id="cfg-models-input"
						type="text"
						bind:value={editModels}
						placeholder="e.g. ~/models"
						class="w-full px-3 py-2 bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg text-xs font-mono text-[var(--text-primary)] focus:outline-none focus:border-[var(--primary)] focus:ring-1 focus:ring-[var(--primary)]"
					/>
				</div>

				<!-- Runtimes Input -->
				<div class="flex flex-col gap-1.5">
					<label for="cfg-runtimes-input" class="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
						<Cpu size={14} class="text-purple-500" />
						<span>Runtimes Directory</span>
					</label>
					<input
						id="cfg-runtimes-input"
						type="text"
						bind:value={editRuntimes}
						placeholder="e.g. ~/.local/share/monolai/runtimes"
						class="w-full px-3 py-2 bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg text-xs font-mono text-[var(--text-primary)] focus:outline-none focus:border-[var(--primary)] focus:ring-1 focus:ring-[var(--primary)]"
					/>
				</div>

				<!-- Hardware Selector -->
				<div class="flex flex-col gap-1.5">
					<label for="cfg-hardware-select" class="text-xs font-semibold text-[var(--text-primary)] flex items-center gap-1.5">
						<Zap size={14} class="text-amber-500" />
						<span>Hardware Acceleration</span>
					</label>
					<div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
						{#each HARDWARE_TARGETS as hw}
							{@const isSelected = editHardware.toLowerCase() === hw.id}
							<button
								type="button"
								onclick={() => (editHardware = hw.id)}
								class="flex flex-col p-2 rounded-lg border text-left cursor-pointer transition-all duration-150 {isSelected
									? 'bg-[var(--primary-light)] border-[var(--primary)] text-[var(--primary)] shadow-xs'
									: 'bg-[var(--bg-surface)] border-[var(--border-color)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:border-[var(--border-hover)]'}"
							>
								<div class="flex items-center justify-between gap-1 w-full">
									<span class="text-xs font-bold leading-tight">{hw.title}</span>
									<span class="text-[0.6rem] font-mono px-1 py-0.2 rounded border uppercase {isSelected ? 'bg-[var(--primary)] text-white border-transparent' : 'bg-[var(--bg-primary)] border-[var(--border-color)] text-[var(--text-muted)]'}">
										{hw.badge}
									</span>
								</div>
								<span class="text-[0.675rem] text-[var(--text-muted)] line-clamp-1 mt-0.5">
									{hw.desc}
								</span>
							</button>
						{/each}
					</div>
				</div>

				<!-- Save Action Bar -->
				<div class="flex items-center justify-end gap-2 pt-2 border-t border-[var(--border-color)]/60">
					<button
						type="button"
						onclick={cancelEdit}
						disabled={isSaving}
						class="px-3 py-1.5 bg-transparent border border-[var(--border-color)] text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] rounded-lg transition cursor-pointer"
					>
						Cancel
					</button>

					<button
						type="button"
						onclick={saveConfiguration}
						disabled={isSaving}
						class="flex items-center gap-1.5 px-4 py-1.5 bg-[var(--primary)] hover:bg-[var(--primary-hover)] text-white text-xs font-bold rounded-lg transition shadow-md shadow-[var(--primary)]/15 cursor-pointer disabled:opacity-50"
					>
						{#if isSaving}
							<RefreshCw size={13} class="animate-spin" />
							<span>Saving...</span>
						{:else}
							<Save size={13} />
							<span>Save Configuration</span>
						{/if}
					</button>
				</div>
			</div>
		{/if}
	{/if}
</div>
