<script lang="ts">
	import { untrack, onMount } from 'svelte';
	import {
		Info,
		Plus,
		Trash2,
		RotateCcw,
		Save,
		AlertCircle,
		Sliders,
		Terminal,
		Copy,
		Check
	} from '@lucide/svelte';
	import {
		getFilePath,
		getFileRelativePath,
		resolveFullPath,
		flagsToCustomText,
		customTextToFlags,
		extractCleanModelName
	} from '$lib/utils/model';
	import { copyToClipboard } from '$lib/utils/clipboard';

	interface RuntimeItem {
		id: string;
		name: string;
		binary_path: string;
	}

	interface ModelFileItem {
		name?: string;
		filename?: string;
		relative_path?: string;
		absolute_path?: string;
		path?: string;
		size_bytes?: number;
		size?: number;
	}

	interface ManifestFlag {
		flag: string;
		name: string;
		description: string;
		type: string;
		important: boolean;
		default_value: string;
		options?: string[];
	}

	interface RuntimeManifest {
		id: string;
		name: string;
		description: string;
		flags: ManifestFlag[];
	}

	interface ModelRecord {
		id: string;
		runtime?: string;
		flags: string;
		created_at?: string;
	}

	let {
		runtimes = [],
		availableFiles = [],
		runtimeManifests = [],
		editingModel = null,
		onSave,
		onCancel
	}: {
		runtimes: RuntimeItem[];
		availableFiles: ModelFileItem[];
		runtimeManifests: RuntimeManifest[];
		editingModel: ModelRecord | null;
		onSave: (payload: {
			id: string;
			runtime: string;
			flags: Record<string, string>;
		}) => Promise<void>;
		onCancel: () => void;
	} = $props();

	let isEditingExisting = $derived(!!editingModel);

	let formId = $state(untrack(() => editingModel?.id || ''));
	let formRuntime = $state(untrack(() => editingModel?.runtime || runtimes[0]?.id || ''));
	let hasUserEditedId = $state(false);

	let initialFlags: Record<string, string> = untrack(() => {
		if (editingModel?.flags) {
			try {
				return JSON.parse(editingModel.flags);
			} catch {
				return {};
			}
		}
		return {};
	});

	const resolve = (val: string) => resolveFullPath(val, availableFiles);

	let formFlags = $state<Record<string, string>>(initialFlags);
	let customText = $state(flagsToCustomText(initialFlags, resolve));
	let activeFormTab = $state<'flags' | 'custom'>('flags');
	let formError = $state<string | null>(null);
	let isSaving = $state(false);
	let selectedAddFlag = $state('');
	let isCopied = $state(false);

	function updateFlags(newFlags: Record<string, string>) {
		formFlags = newFlags;
		customText = flagsToCustomText(newFlags, resolve);
	}

	function handleCustomTextChange(newText: string) {
		customText = newText;
		formFlags = customTextToFlags(newText, resolve);
	}

	function getManifestForRuntime(runtimeId: string): RuntimeManifest | null {
		if (!runtimeId) return null;
		return runtimeManifests.find((m) => m.id.toLowerCase() === runtimeId.toLowerCase()) || null;
	}

	let currentManifest = $derived(getManifestForRuntime(formRuntime));

	$effect(() => {
		if (!currentManifest && activeFormTab === 'flags') {
			activeFormTab = 'custom';
		}
	});

	let currentBinaryPath = $derived(
		runtimes.find((p) => p.id === formRuntime)?.binary_path ||
			(formRuntime?.includes('sd') ? '/usr/bin/sd-server' : '/usr/bin/llama-server')
	);

	async function copyFullCommand() {
		const fullCmd = `${currentBinaryPath} ${customText.trim()}`.trim();
		const ok = await copyToClipboard(fullCmd);
		if (ok) {
			isCopied = true;
			setTimeout(() => {
				isCopied = false;
			}, 2000);
		}
	}

	let unusedFlags = $derived(() => {
		if (!currentManifest) return [];
		return currentManifest.flags.filter((f) => !(f.flag in formFlags));
	});

	function loadDefaultFlags(runtimeId: string) {
		const manifest = getManifestForRuntime(runtimeId);
		if (!manifest) return;

		const defaults: Record<string, string> = {};
		for (const flagDef of manifest.flags) {
			if (flagDef.important) {
				const val = flagDef.default_value || '';
				defaults[flagDef.flag] = flagDef.type === 'file' ? resolve(val) : val;
			}
		}

		if (formFlags['--model']) {
			defaults['--model'] = resolve(formFlags['--model']);
		} else if (availableFiles.length > 0) {
			defaults['--model'] = getFilePath(availableFiles[0]);
		}

		updateFlags(defaults);

		if (!isEditingExisting && (!hasUserEditedId || !formId.trim())) {
			const mainFile = defaults['--model'] || defaults['-m'];
			if (mainFile) {
				const clean = extractCleanModelName(mainFile);
				if (clean) {
					formId = clean;
				}
			}
		}
	}

	let lastResolvedFilesCount = -1;
	$effect(() => {
		const files = availableFiles;
		if (files.length > 0 && files.length !== lastResolvedFilesCount) {
			lastResolvedFilesCount = files.length;
			untrack(() => {
				let updated = false;
				const nextFlags = { ...formFlags };
				for (const [k, v] of Object.entries(nextFlags)) {
					const resolved = resolve(v);
					if (resolved && resolved !== v) {
						nextFlags[k] = resolved;
						updated = true;
					}
				}
				if (updated) {
					formFlags = nextFlags;
					customText = flagsToCustomText(formFlags, resolve);
				}

				if (!isEditingExisting && (!hasUserEditedId || !formId.trim())) {
					const mainFile =
						formFlags['--model'] || formFlags['-m'] || (files[0] ? getFilePath(files[0]) : '');
					if (mainFile) {
						const clean = extractCleanModelName(mainFile);
						if (clean) {
							formId = clean;
						}
					}
				}
			});
		}
	});

	// Initialize default flags for new model if empty
	onMount(() => {
		if (!editingModel && Object.keys(formFlags).length === 0) {
			loadDefaultFlags(formRuntime);
		}
	});

	function handleRuntimeChange(e: Event) {
		const target = e.target as HTMLSelectElement;
		formRuntime = target.value;
		loadDefaultFlags(formRuntime);
	}

	function resetDefaultFlags() {
		loadDefaultFlags(formRuntime);
	}

	function addFlag() {
		if (!selectedAddFlag) return;
		const manifest = currentManifest;
		const flagDef = manifest?.flags.find((f) => f.flag === selectedAddFlag);

		let initialVal = flagDef?.default_value || '';
		if (flagDef?.type === 'file' && availableFiles.length > 0) {
			initialVal = getFilePath(availableFiles[0]);
		} else {
			initialVal = resolve(initialVal);
		}

		updateFlags({ ...formFlags, [selectedAddFlag]: initialVal });
		if (!isEditingExisting && (!hasUserEditedId || !formId.trim()) && flagDef?.type === 'file' && initialVal) {
			const clean = extractCleanModelName(initialVal);
			if (clean) {
				formId = clean;
			}
		}
		selectedAddFlag = '';
	}

	function handleFileSelect(flagKey: string, value: string) {
		updateFlags({ ...formFlags, [flagKey]: value });
		if (!isEditingExisting && (!hasUserEditedId || !formId.trim())) {
			const clean = extractCleanModelName(value);
			if (clean) {
				formId = clean;
			}
		}
	}

	function removeFlag(flagKey: string) {
		const next = { ...formFlags };
		delete next[flagKey];
		updateFlags(next);
	}

	function validateId(id: string): string | null {
		const trimmed = id.trim();
		if (!trimmed) {
			return 'Model ID is required.';
		}
		if (/\s/.test(trimmed)) {
			return 'Model ID cannot contain spaces (e.g. Qwen3.8:27b-q4-mtp).';
		}
		return null;
	}

	async function handleSubmit() {
		formError = null;
		const idError = validateId(formId);
		if (idError && !isEditingExisting) {
			formError = idError;
			return;
		}

		isSaving = true;
		try {
			await onSave({
				id: formId.trim(),
				runtime: formRuntime,
				flags: formFlags
			});
		} catch (e: any) {
			formError = e.message || 'Failed to save model';
		} finally {
			isSaving = false;
		}
	}
</script>

<div class="model-form-container">
	<div class="form-header-row">
		<h4 class="form-title">
			{isEditingExisting ? 'Edit Model' : 'New Model'}
		</h4>
		<button type="button" class="btn-text" onclick={onCancel}> Back to List </button>
	</div>

	{#if formError}
		<div class="error-banner">
			<AlertCircle size={16} />
			<span>{formError}</span>
		</div>
	{/if}

	<div class="model-form-body">
		<!-- Model ID -->
		<div class="form-group">
			<label class="form-label" for="model-id">
				Model ID <span class="required-star">*</span>
			</label>
			<input
				id="model-id"
				type="text"
				bind:value={formId}
				oninput={() => {
					hasUserEditedId = true;
				}}
				disabled={isEditingExisting}
				placeholder="e.g. Qwen3.8:27b-q4-mtp"
				class="input-control {validateId(formId) && !isEditingExisting && formId
					? 'has-error'
					: ''}"
			/>
			{#if validateId(formId) && !isEditingExisting && formId}
				<span class="field-error-text">{validateId(formId)}</span>
			{:else}
				<span class="field-hint"
					>Unique identifier string without spaces (e.g. Qwen3.8:27b-q4-mtp)</span
				>
			{/if}
		</div>

		<!-- Runtime Engine Select -->
		<div class="form-group">
			<label class="form-label" for="model-runtime">
				Runtime Engine <span class="required-star">*</span>
			</label>
			<select
				id="model-runtime"
				value={formRuntime}
				onchange={handleRuntimeChange}
				class="select-control"
			>
				{#each runtimes as rt}
					<option value={rt.id}>{rt.name} ({rt.id})</option>
				{/each}
			</select>
		</div>

		<!-- Sub-Tabs: Form Flags vs Custom Text -->
		<div class="flags-section">
			{#if currentManifest}
				<div class="flags-subtabs-header">
					<div class="subtabs-buttons">
						<button
							type="button"
							class="subtab-btn {activeFormTab === 'flags' ? 'active' : ''}"
							onclick={() => (activeFormTab = 'flags')}
						>
							<Sliders size={14} />
							<span>Form</span>
						</button>

						<button
							type="button"
							class="subtab-btn {activeFormTab === 'custom' ? 'active' : ''}"
							onclick={() => (activeFormTab = 'custom')}
						>
							<Terminal size={14} />
							<span>Command</span>
						</button>
					</div>

					{#if activeFormTab === 'flags'}
						<button
							type="button"
							class="btn-secondary-sm"
							onclick={resetDefaultFlags}
							title="Reset to default flags"
						>
							<RotateCcw size={13} />
							<span>Reset Defaults</span>
						</button>
					{/if}
				</div>
			{/if}

			{#if currentManifest && activeFormTab === 'flags'}
				<!-- Structured Form Flags View (Single Elevated Card Container) -->
				{#if Object.keys(formFlags).length > 0}
					<div class="flags-active-container">
						{#each Object.keys(formFlags) as flagKey}
							{@const flagDef = currentManifest?.flags.find((f) => f.flag === flagKey)}
							<div class="flag-control-row">
								<div class="flag-card-header">
									<span class="flag-name font-mono">{flagKey}</span>
									{#if flagDef?.name}
										<span class="flag-label">{flagDef.name}</span>
									{/if}

									<!-- Info Tooltip Button -->
									{#if flagDef?.description}
										<div class="info-tooltip-container">
											<button
												type="button"
												class="info-icon-btn"
												aria-label="Flag description"
												title={flagDef.description}
											>
												<Info size={13} />
											</button>
											<div class="info-tooltip">
												{flagDef.description}
											</div>
										</div>
									{/if}

									<button
										type="button"
										class="remove-flag-btn"
										onclick={() => removeFlag(flagKey)}
										title="Remove flag"
									>
										<Trash2 size={14} />
									</button>
								</div>

								<div class="flag-input-wrapper">
									{#if flagDef?.type === 'file'}
										<select
											bind:value={formFlags[flagKey]}
											onchange={(e) => handleFileSelect(flagKey, (e.target as HTMLSelectElement).value)}
											class="select-control"
										>
											{#if availableFiles.length === 0}
												<option value="">No files found in models directory</option>
											{:else}
												{#each availableFiles as fileItem}
													{@const fullPath = getFilePath(fileItem)}
													{@const relPath = getFileRelativePath(fileItem)}
													<option value={fullPath}>
														{relPath}
													</option>
												{/each}
												{#if formFlags[flagKey] && !availableFiles.some((f) => getFilePath(f) === formFlags[flagKey])}
													<option value={formFlags[flagKey]}>
														{getFileRelativePath({
															absolute_path: formFlags[flagKey],
															relative_path: formFlags[flagKey]
														})}
													</option>
												{/if}
											{/if}
										</select>
									{:else if flagDef?.type === 'enum' && flagDef.options}
										<select
											bind:value={formFlags[flagKey]}
											onchange={() => updateFlags({ ...formFlags })}
											class="select-control"
										>
											{#each flagDef.options as opt}
												<option value={opt}>{opt}</option>
											{/each}
										</select>
									{:else if flagDef?.type === 'bool'}
										<label class="bool-checkbox-label">
											<input
												type="checkbox"
												checked={formFlags[flagKey] === 'true'}
												onchange={(e) => {
													updateFlags({
														...formFlags,
														[flagKey]: (e.target as HTMLInputElement).checked ? 'true' : 'false'
													});
												}}
												class="setting-checkbox"
											/>
											<span>Enable flag</span>
										</label>
									{:else if flagDef?.type === 'int' || flagDef?.type === 'float'}
										<input
											type="number"
											bind:value={formFlags[flagKey]}
											oninput={() => updateFlags({ ...formFlags })}
											class="input-control"
										/>
									{:else}
										<input
											type="text"
											bind:value={formFlags[flagKey]}
											oninput={() => updateFlags({ ...formFlags })}
											class="input-control"
										/>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				{/if}

				<!-- Add Optional Flag -->
				{#if unusedFlags().length > 0}
					<div class="add-flag-row">
						<select bind:value={selectedAddFlag} class="select-control">
							<option value="">Select an optional flag to add...</option>
							{#each unusedFlags() as optFlag}
								<option value={optFlag.flag}>
									{optFlag.flag} — {optFlag.name || optFlag.description}
								</option>
							{/each}
						</select>
						<button
							type="button"
							class="btn-secondary"
							disabled={!selectedAddFlag}
							onclick={addFlag}
						>
							<Plus size={15} />
							<span>Add Flag</span>
						</button>
					</div>
				{/if}
			{:else}
				<!-- Custom Raw Command Text Editor -->
				<div class="custom-command-editor font-mono">
					<div class="command-editor-header">
						<label class="form-label" for="custom-text">
							Write raw CLI arguments directly. The runtime binary path cannot be modified here.
						</label>
						<button
							type="button"
							class="copy-cmd-btn"
							onclick={copyFullCommand}
							title="Copy full command line with binary path"
						>
							{#if isCopied}
								<Check size={13} class="text-green" />
								<span>Copied!</span>
							{:else}
								<Copy size={13} />
								<span>Copy Command</span>
							{/if}
						</button>
					</div>
					<div class="command-prefix-box">
						<div class="command-prefix-header font-mono">
							<span class="binary-path font-mono">{currentBinaryPath}</span>
						</div>
						<textarea
							id="custom-text"
							rows="6"
							value={customText}
							oninput={(e) => handleCustomTextChange((e.target as HTMLTextAreaElement).value)}
							placeholder="--model /path/to/model.gguf --ctx-size 4096 --n-gpu-layers 99"
							class="textarea-control font-mono command-textarea"></textarea>
					</div>
				</div>
			{/if}
		</div>

		<!-- Actions -->
		<div class="form-actions-row">
			<button type="button" class="btn-secondary" onclick={onCancel}> Cancel </button>
			<button type="button" class="btn-primary" disabled={isSaving} onclick={handleSubmit}>
				<Save size={15} />
				<span>{isSaving ? 'Saving...' : 'Save Model'}</span>
			</button>
		</div>
	</div>
</div>

<style>
	.model-form-container {
		display: flex;
		flex-direction: column;
		gap: 0.875rem;
	}

	.form-header-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.form-title {
		margin: 0;
		font-size: 0.95rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.btn-text {
		background: transparent;
		border: none;
		color: var(--primary);
		font-size: 0.825rem;
		font-weight: 500;
		cursor: pointer;
	}

	.error-banner {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		background: rgba(239, 68, 68, 0.1);
		border: 1px solid rgba(239, 68, 68, 0.3);
		color: #ef4444;
		padding: 0.6rem 0.75rem;
		border-radius: 0.5rem;
		font-size: 0.8rem;
	}

	.model-form-body {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}

	.form-label {
		font-size: 0.8rem;
		font-weight: 600;
		color: var(--text-primary);
	}

	.required-star {
		color: #ef4444;
	}

	.input-control,
	.select-control,
	.textarea-control {
		width: 100%;
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		border-radius: 0.5rem;
		padding: 0.5rem 0.75rem;
		font-size: 0.825rem;
		color: var(--text-primary);
		outline: none;
		box-sizing: border-box;
		transition: border-color 0.15s ease;
	}

	.textarea-control {
		resize: vertical;
		line-height: 1.4;
	}

	.input-control:focus,
	.select-control:focus,
	.textarea-control:focus {
		border-color: var(--primary);
	}

	.input-control.has-error {
		border-color: #ef4444;
	}

	.field-hint {
		font-size: 0.725rem;
		color: var(--text-muted);
	}

	.field-error-text {
		font-size: 0.725rem;
		color: #ef4444;
	}

	/* Flags Section & Subtabs */
	.flags-section {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		margin-top: 0.25rem;
		padding-top: 0.75rem;
		border-top: 1px solid var(--border-color);
	}

	.flags-subtabs-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
	}

	.subtabs-buttons {
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
		background: var(--bg-primary);
		padding: 0.25rem;
		border-radius: 0.5rem;
		border: 1px solid var(--border-color);
	}

	.subtab-btn {
		display: flex;
		align-items: center;
		gap: 0.45rem;
		padding: 0.4rem 0.95rem;
		font-size: 0.8rem;
		font-weight: 500;
		color: var(--text-secondary);
		background: transparent;
		border: 1px solid transparent;
		border-radius: 0.375rem;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.subtab-btn:hover {
		color: var(--text-primary);
		background: var(--bg-hover);
	}

	.subtab-btn.active {
		color: var(--primary);
		background: var(--bg-surface);
		border-color: var(--border-color);
		font-weight: 600;
		box-shadow: 0 1px 4px rgba(0, 0, 0, 0.15);
	}

	/* Single Card Container holding all active flags */
	.flags-active-container {
		background: var(--bg-surface);
		border: 1px solid var(--border-color);
		border-radius: 0.75rem;
		padding: 0.875rem 1rem;
		display: flex;
		flex-direction: column;
		gap: 0.875rem;
		box-shadow: 0 1px 6px rgba(0, 0, 0, 0.12);
	}

	.flag-control-row {
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
		padding-bottom: 0.75rem;
		border-bottom: 1px solid var(--border-color);
	}

	.flag-control-row:last-child {
		padding-bottom: 0;
		border-bottom: none;
	}

	.flag-card-header {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.flag-name {
		font-size: 0.8rem;
		font-weight: 700;
		color: var(--primary);
	}

	.flag-label {
		font-size: 0.775rem;
		color: var(--text-secondary);
	}

	/* Info Tooltip */
	.info-tooltip-container {
		position: relative;
		display: inline-flex;
		align-items: center;
	}

	.info-icon-btn {
		background: transparent;
		border: none;
		color: var(--text-muted);
		padding: 0.15rem;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 0.25rem;
		transition: color 0.12s ease;
	}

	.info-icon-btn:hover {
		color: var(--primary);
	}

	.info-tooltip {
		display: none;
		position: absolute;
		bottom: 125%;
		left: 50%;
		transform: translateX(-50%);
		background: var(--bg-surface);
		border: 1px solid var(--border-color);
		color: var(--text-secondary);
		padding: 0.4rem 0.6rem;
		border-radius: 0.375rem;
		font-size: 0.7rem;
		white-space: normal;
		max-width: 220px;
		width: max-content;
		z-index: 50;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
		pointer-events: none;
	}

	.info-tooltip-container:hover .info-tooltip {
		display: block;
	}

	.remove-flag-btn {
		margin-left: auto;
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		padding: 0.2rem;
	}

	.remove-flag-btn:hover {
		color: #ef4444;
	}

	.flag-input-wrapper {
		margin-top: 0.2rem;
	}

	.bool-checkbox-label {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.8rem;
		color: var(--text-primary);
		cursor: pointer;
	}

	.setting-checkbox {
		accent-color: var(--primary);
		width: 16px;
		height: 16px;
	}

	.add-flag-row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-top: 0.25rem;
	}

	.add-flag-row select {
		flex: 1;
		min-width: 0;
	}

	.add-flag-row button {
		white-space: nowrap;
		flex-shrink: 0;
		min-width: max-content;
	}

	.custom-command-editor {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.command-editor-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.copy-cmd-btn {
		display: flex;
		align-items: center;
		gap: 0.35rem;
		background: transparent;
		color: var(--text-muted);
		border: 1px solid var(--border-color);
		border-radius: 0.375rem;
		padding: 0.25rem 0.55rem;
		font-size: 0.75rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.copy-cmd-btn:hover {
		color: var(--primary);
		border-color: var(--primary);
		background: var(--primary-light);
	}

	:global(.text-green) {
		color: #10b981;
	}

	.command-prefix-box {
		display: flex;
		flex-direction: column;
		border: 1px solid var(--border-color);
		border-radius: 0.5rem;
		overflow: hidden;
		background: var(--bg-primary);
	}

	.command-prefix-header {
		background: var(--bg-hover);
		padding: 0.5rem 0.75rem;
		font-size: 0.825rem;
		border-bottom: 1px solid var(--border-color);
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.binary-path {
		color: var(--primary);
		font-weight: 700;
	}

	.command-textarea {
		border: none !important;
		border-radius: 0 !important;
		background: transparent !important;
		padding: 0.625rem 0.75rem;
	}

	.command-textarea:focus {
		outline: none !important;
		border-color: transparent !important;
		box-shadow: none !important;
	}

	.form-actions-row {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 0.75rem;
		margin-top: 0.75rem;
		padding-top: 0.75rem;
		border-top: 1px solid var(--border-color);
	}

	.btn-primary {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		background: var(--primary);
		color: white;
		border: none;
		border-radius: 0.5rem;
		padding: 0.5rem 0.875rem;
		font-size: 0.825rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-primary:hover {
		opacity: 0.9;
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-secondary {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.4rem;
		white-space: nowrap;
		flex-shrink: 0;
		background: var(--bg-hover);
		color: var(--text-primary);
		border: 1px solid var(--border-color);
		border-radius: 0.5rem;
		padding: 0.45rem 0.85rem;
		font-size: 0.825rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.btn-secondary:hover {
		background: var(--bg-surface-hover);
	}

	.btn-secondary-sm {
		display: flex;
		align-items: center;
		gap: 0.3rem;
		background: transparent;
		color: var(--text-muted);
		border: 1px solid var(--border-color);
		border-radius: 0.375rem;
		padding: 0.25rem 0.5rem;
		font-size: 0.75rem;
		cursor: pointer;
		transition: all 0.12s ease;
	}

	.btn-secondary-sm:hover {
		color: var(--text-primary);
		background: var(--bg-hover);
	}

	.font-mono {
		font-family: 'JetBrains Mono', monospace;
	}
</style>
