<script lang="ts">
	import { copyToClipboard } from '$lib/utils/clipboard';
	import {
		customTextToFlags,
		extractCleanModelName,
		flagsToCustomText,
		getFilePath,
		getFileRelativePath,
		parseFlags,
		resolveFullPath
	} from '$lib/utils/model';
	import {
		Check,
		CircleAlert,
		Copy,
		Info,
		Plus,
		RotateCcw,
		Save,
		SlidersVertical,
		Terminal,
		Trash
	} from '@lucide/svelte';
	import { onMount, untrack } from 'svelte';
	import { Alert, Button, Input, Select, Tabs } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ModelRecord } from '$lib/types/models';

	interface RuntimeItem {
		id: string;
		name: string;
		version?: string;
		description?: string;
		features?: string[];
		is_installed?: boolean;
		installed_path?: string | null;
		binary_path?: string | null;
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
		binary_name?: string;
		flags: ManifestFlag[];
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

	let installedRuntimes = $derived(
		runtimes.filter((r) => r.is_installed || (isEditingExisting && r.id === editingModel?.runtime))
	);

	let installedRuntimeOptions = $derived(
		installedRuntimes.map((rt) => {
			const manifest = getManifestForRuntime(rt.id);
			const displayName = manifest?.name || rt.name || rt.id;
			return {
				value: rt.id,
				label: `${displayName} (${rt.id})`
			};
		})
	);

	let formId = $state(untrack(() => editingModel?.id || ''));
	let formRuntime = $state(
		untrack(() => {
			if (editingModel?.runtime) return editingModel.runtime;
			const firstInstalled = runtimes.find((r) => r.is_installed);
			return firstInstalled?.id || runtimes[0]?.id || '';
		})
	);
	let hasUserEditedId = $state(false);

	$effect(() => {
		if (!formRuntime && installedRuntimes.length > 0) {
			formRuntime = installedRuntimes[0].id;
		}
	});

	let initialFlags: Record<string, string> = untrack(() => {
		if (editingModel?.flags) {
			return parseFlags(editingModel.flags);
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
		`./${currentManifest?.binary_name || (formRuntime?.includes('sd') ? 'sd-server' : 'llama-server')}`
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

	let unusedFlags = $derived.by(() => {
		if (!currentManifest || !formFlags || typeof formFlags !== 'object') return [];
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

	function handleRuntimeChange(val: string) {
		formRuntime = val;
		loadDefaultFlags(formRuntime);
	}

	function resetDefaultFlags() {
		loadDefaultFlags(formRuntime);
	}

	function getFileOptions(currentValue: string) {
		const opts = availableFiles.map((fileItem) => ({
			value: getFilePath(fileItem),
			label: getFileRelativePath(fileItem)
		}));
		const resolvedCurrent = resolve(currentValue);
		if (
			resolvedCurrent &&
			!availableFiles.some(
				(f) => getFilePath(f) === resolvedCurrent || getFileRelativePath(f) === currentValue
			)
		) {
			opts.unshift({
				value: resolvedCurrent,
				label: getFileRelativePath({
					absolute_path: resolvedCurrent,
					relative_path: currentValue
				})
			});
		}
		if (opts.length === 0) {
			return [{ value: '', label: t('models.noFilesFound') }];
		}
		return opts;
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
		if (
			!isEditingExisting &&
			(!hasUserEditedId || !formId.trim()) &&
			flagDef?.type === 'file' &&
			initialVal
		) {
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
			return t('models.modelIdRequired');
		}
		if (/\s/.test(trimmed)) {
			return t('models.modelIdNoSpaces');
		}
		return null;
	}

	async function handleSubmit() {
		formError = null;
		const idError = validateId(formId);
		if (idError) {
			formError = idError;
			return;
		}

		if (installedRuntimes.length === 0) {
			formError = t('models.noInstalledRuntimesFound');
			return;
		}

		if (!formRuntime || !formRuntime.trim()) {
			formError = t('models.runtimeRequired');
			return;
		}

		const isRuntimeInstalled = installedRuntimes.some((r) => r.id === formRuntime);
		if (!isRuntimeInstalled) {
			formError = t('models.selectedRuntimeNotInstalled', { runtime: formRuntime });
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
			formError = e.message || t('models.saveFailed');
		} finally {
			isSaving = false;
		}
	}
</script>

<form
	class="flex flex-col gap-4"
	onsubmit={(e) => {
		e.preventDefault();
		handleSubmit();
	}}
>
	{#if formError}
		<Alert variant="error" dismissible ondismiss={() => (formError = null)}>
			{formError}
		</Alert>
	{/if}

	<div class="flex flex-col gap-4">
		<!-- Model ID -->
		<div class="flex flex-col gap-1.5">
			<label class="text-xs font-semibold text-[var(--text-primary)]" for="model-id">
				{t('models.modelId')} <span class="text-[var(--danger)]">*</span>
			</label>
			<Input
				id="model-id"
				type="text"
				bind:value={formId}
				oninput={() => {
					hasUserEditedId = true;
				}}
				placeholder="e.g. Qwen3.8:27b-q4-mtp"
				error={!!validateId(formId) && !!formId}
			/>
			{#if validateId(formId) && formId}
				<span class="text-xs text-[var(--danger-text)]">{validateId(formId)}</span>
			{:else}
				<span class="text-xs text-[var(--text-muted)]">{t('models.modelIdHint')}</span>
			{/if}
		</div>

		<!-- Runtime Engine Select -->
		<div class="flex flex-col gap-1.5">
			<label class="text-xs font-semibold text-[var(--text-primary)]" for="model-runtime">
				{t('models.runtimeEngine')} <span class="text-[var(--danger)]">*</span>
			</label>
			{#if installedRuntimeOptions.length > 0}
				<Select
					bind:value={formRuntime}
					options={installedRuntimeOptions}
					onchange={handleRuntimeChange}
					placeholder={t('models.selectInstalledRuntime')}
					ariaLabel={t('models.runtimeEngine')}
				/>
			{:else}
				<div
					class="p-3 text-xs rounded-xl bg-[var(--bg-primary)] border border-[var(--warning-border)] text-[var(--warning-text)] flex items-center justify-between gap-3"
				>
					<span>{t('models.noInstalledRuntimesWarning')}</span>
					<a
						href="/runtimes"
						class="inline-flex items-center px-2.5 py-1 text-xs rounded-lg bg-[var(--btn-bg)] text-[var(--text-primary)] border border-[var(--border-color)] hover:bg-[var(--btn-hover)] shrink-0 whitespace-nowrap no-underline font-medium"
					>
						{t('navigation.runtimes')}
					</a>
				</div>
			{/if}
		</div>

		<!-- Sub-Tabs: Form Flags vs Custom Text -->
		{#if currentManifest}
			<Tabs
				surface="dark"
				bind:value={activeFormTab}
				title={t('models.executionFlags')}
				subtitle={t('models.flagsSubtitle')}
				items={[
					{ id: 'flags', label: t('models.formTab'), icon: SlidersVertical },
					{ id: 'custom', label: t('models.commandTab'), icon: Terminal }
				]}
				class="mt-1"
				cardClass="p-4 sm:p-5 flex flex-col gap-4"
			>
				{#snippet action()}
					{#if activeFormTab === 'flags'}
						<Button
							variant="secondary"
							size="sm"
							onclick={resetDefaultFlags}
							title={t('models.resetDefaults')}
							ariaLabel={t('models.resetDefaults')}
						>
							<RotateCcw size={13} />
							<span>{t('models.resetDefaults')}</span>
						</Button>
					{/if}
				{/snippet}

				{#if activeFormTab === 'flags'}
					<!-- Structured Form Flags View -->
					{#if Object.keys(formFlags).length > 0}
						<div class="flex flex-col gap-3">
							{#each Object.keys(formFlags) as flagKey}
								{@const flagDef = currentManifest?.flags.find((f) => f.flag === flagKey)}
								<div
									class="flex flex-col gap-1.5 pb-3 border-b border-[var(--border-color)] last:border-b-0 last:pb-0"
								>
									<div class="flex items-center gap-2">
										<span class="text-xs font-mono font-bold text-[var(--primary)]">{flagKey}</span>
										{#if flagDef?.name}
											<span class="text-xs text-[var(--text-secondary)]">{flagDef.name}</span>
										{/if}

										<!-- Info Tooltip Button -->
										{#if flagDef?.description}
											<div class="relative group/tip inline-flex items-center">
												<button
													type="button"
													class="p-0.5 text-[var(--text-muted)] hover:text-[var(--primary)] bg-transparent border-0 cursor-pointer rounded flex items-center justify-center transition-colors"
													aria-label={t('models.flagDescription')}
													title={flagDef.description}
												>
													<Info size={13} />
												</button>
												<div
													class="hidden group-hover/tip:block absolute bottom-full left-1/2 -translate-x-1/2 mb-1.5 bg-[var(--bg-surface)] border border-[var(--border-color)] text-[var(--text-secondary)] px-2 py-1 rounded-md text-xs shadow-lg max-w-[240px] w-max z-50 pointer-events-none"
												>
													{flagDef.description}
												</div>
											</div>
										{/if}

										<Button
											variant="ghost"
											size="icon-xs"
											class="ml-auto text-[var(--text-muted)] hover:text-[var(--danger-text)]"
											onclick={() => removeFlag(flagKey)}
											title={t('models.removeFlag')}
											ariaLabel={t('models.removeFlag')}
										>
											<Trash size={14} />
										</Button>
									</div>

									<div class="mt-0.5">
										{#if flagDef?.type === 'file'}
											<Select
												value={formFlags[flagKey] || ''}
												options={getFileOptions(formFlags[flagKey] || '')}
												onchange={(val: string) => handleFileSelect(flagKey, val)}
												placeholder={t('models.selectModelFile')}
												ariaLabel={flagDef.name || flagKey}
											/>
										{:else if flagDef?.type === 'enum' && flagDef.options}
											<Select
												value={formFlags[flagKey] || ''}
												options={flagDef.options.map((opt) => ({ value: opt, label: opt }))}
												onchange={(val: string) => {
													formFlags[flagKey] = val;
													updateFlags({ ...formFlags });
												}}
												placeholder={t('models.selectOption')}
												ariaLabel={flagDef.name || flagKey}
											/>
										{:else if flagDef?.type === 'bool'}
											<label
												class="flex items-center gap-2 text-xs text-[var(--text-primary)] cursor-pointer select-none"
											>
												<input
													type="checkbox"
													checked={formFlags[flagKey] === 'true'}
													onchange={(e) => {
														updateFlags({
															...formFlags,
															[flagKey]: (e.target as HTMLInputElement).checked ? 'true' : 'false'
														});
													}}
													class="accent-[var(--primary)] w-4 h-4 cursor-pointer"
												/>
												<span>{t('models.enableFlag')}</span>
											</label>
										{:else if flagDef?.type === 'int' || flagDef?.type === 'float'}
											<Input
												type="number"
												value={formFlags[flagKey] || ''}
												oninput={(e) => {
													updateFlags({
														...formFlags,
														[flagKey]: (e.target as HTMLInputElement).value
													});
												}}
												size="sm"
											/>
										{:else}
											<Input
												type="text"
												value={formFlags[flagKey] || ''}
												oninput={(e) => {
													updateFlags({
														...formFlags,
														[flagKey]: (e.target as HTMLInputElement).value
													});
												}}
												size="sm"
											/>
										{/if}
									</div>
								</div>
							{/each}
						</div>
					{/if}

					<!-- Add Optional Flag -->
					{#if unusedFlags.length > 0}
						<div class="flex items-center gap-2 mt-1">
							<Select
								class="flex-1 min-w-0"
								bind:value={selectedAddFlag}
								options={[
									{ value: '', label: t('models.selectOptionalFlag') },
									...unusedFlags.map((optFlag) => ({
										value: optFlag.flag,
										label: `${optFlag.flag} — ${optFlag.name || optFlag.description}`
									}))
								]}
								placeholder={t('models.selectOptionalFlag')}
								ariaLabel={t('models.addOptionalFlag')}
							/>
							<Button
								variant="secondary"
								size="sm"
								disabled={!selectedAddFlag}
								onclick={addFlag}
								class="shrink-0 whitespace-nowrap"
							>
								<Plus size={15} />
								<span>{t('common.add')}</span>
							</Button>
						</div>
					{/if}
				{:else}
					<!-- Custom Raw Command Text Editor -->
					<div class="flex flex-col gap-2 font-mono">
						<div class="flex items-center justify-between">
							<label class="text-xs font-semibold text-[var(--text-primary)]" for="custom-text">
								{t('models.rawCliHint')}
							</label>
							<Button
								variant="secondary"
								size="xs"
								onclick={copyFullCommand}
								title={t('models.copyFullCommand')}
								ariaLabel={t('models.copyFullCommand')}
							>
								{#if isCopied}
									<Check size={13} class="text-[var(--success-text)]" />
									<span>{t('common.copied')}</span>
								{:else}
									<Copy size={13} />
									<span>{t('common.copy')}</span>
								{/if}
							</Button>
						</div>
						<div
							class="flex flex-col border border-[var(--border-color)] rounded-xl overflow-hidden bg-[var(--bg-primary)]"
						>
							<div
								class="bg-[var(--bg-hover)] px-3 py-2 text-xs border-b border-[var(--border-color)] flex items-center gap-2"
							>
								<span class="text-[var(--primary)] font-bold">{currentBinaryPath}</span>
							</div>
							<textarea
								id="custom-text"
								rows="6"
								value={customText}
								oninput={(e) => handleCustomTextChange((e.target as HTMLTextAreaElement).value)}
								onkeydown={(e) => {
									if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
										e.preventDefault();
										handleSubmit();
									}
								}}
								placeholder="--model /path/to/model.gguf --ctx-size 4096 --n-gpu-layers 99"
								class="w-full bg-transparent p-3 text-xs text-[var(--text-primary)] placeholder-[var(--text-muted)] font-mono outline-none resize-y border-0 leading-relaxed"
							></textarea>
						</div>
					</div>
				{/if}
			</Tabs>
		{/if}

		<!-- Actions -->
		<div
			class="flex items-center justify-end gap-3 pt-3 mt-3 border-t border-[var(--border-color)]"
		>
			<Button variant="secondary" size="md" onclick={onCancel}>
				{t('common.cancel')}
			</Button>
			<Button
				type="submit"
				variant="primary"
				size="md"
				disabled={isSaving || installedRuntimes.length === 0 || !formRuntime}
				title={installedRuntimes.length === 0 ? t('models.noInstalledRuntimes') : t('common.save')}
			>
				<Save size={15} />
				<span>{isSaving ? t('models.saving') : t('common.save')}</span>
			</Button>
		</div>
	</div>
</form>
