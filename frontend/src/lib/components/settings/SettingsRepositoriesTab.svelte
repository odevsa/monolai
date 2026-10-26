<script lang="ts">
	import { hubApi } from '$lib/api/hub';
	import { Alert, Badge, Button, Input, Modal, Select, Spinner, Toggle } from '$lib/components/ui';
	import { askConfirm } from '$lib/confirmStore';
	import { t } from '$lib/i18n';
	import type { CreateRepositoryPayload, RepositoryRecord } from '$lib/types/models';
	import {
		Check,
		Database,
		ExternalLink,
		Eye,
		EyeOff,
		Globe,
		KeyRound,
		Plus,
		Save,
		ShieldCheck,
		Trash2
	} from '@lucide/svelte';
	import { onMount } from 'svelte';
	import SettingGroup from './SettingGroup.svelte';
	import SettingRow from './SettingRow.svelte';

	let repositories = $state<RepositoryRecord[]>([]);
	let isLoading = $state(true);
	let isSaving = $state(false);
	let saveSuccess = $state(false);
	let errorMessage = $state<string | null>(null);

	// Tracking state per repo
	let editingTokens = $state<Record<string, string>>({});
	let editingEnabled = $state<Record<string, boolean>>({});
	let showToken = $state<Record<string, boolean>>({});
	let showTokenGuide = $state<Record<string, boolean>>({});

	// Custom repo modal
	let showAddModal = $state(false);
	let isAdding = $state(false);
	let newRepo = $state<{
		id: string;
		name: string;
		repo_type: string;
		base_url: string;
		api_token: string;
		enabled: boolean;
	}>({
		id: '',
		name: '',
		repo_type: 'huggingface',
		base_url: '',
		api_token: '',
		enabled: true
	});

	const repoTypeOptions = [
		{ value: 'huggingface', label: 'Hugging Face Hub' },
		{ value: 'custom', label: 'Custom' }
	];

	async function loadRepositories() {
		isLoading = true;
		errorMessage = null;
		try {
			repositories = await hubApi.getRepositories();
			const tokens: Record<string, string> = {};
			const enabled: Record<string, boolean> = {};
			for (const r of repositories) {
				tokens[r.id] = r.api_token || '';
				enabled[r.id] = r.enabled;
			}
			editingTokens = tokens;
			editingEnabled = enabled;
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			errorMessage = msg;
		} finally {
			isLoading = false;
		}
	}

	async function saveAllSettings() {
		isSaving = true;
		errorMessage = null;
		saveSuccess = false;

		try {
			const updates = repositories.map(async (repo) => {
				const tokenVal = editingTokens[repo.id]?.trim() ?? '';
				const isEnabled = editingEnabled[repo.id] ?? repo.enabled;

				return hubApi.updateRepository(repo.id, {
					api_token: tokenVal,
					enabled: isEnabled
				});
			});

			const updatedList = await Promise.all(updates);
			repositories = updatedList;
			for (const r of updatedList) {
				editingTokens[r.id] = r.api_token || '';
				editingEnabled[r.id] = r.enabled;
			}

			saveSuccess = true;
			setTimeout(() => {
				saveSuccess = false;
			}, 3000);
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			errorMessage = msg;
		} finally {
			isSaving = false;
		}
	}

	async function deleteRepo(repo: RepositoryRecord) {
		const confirmed = await askConfirm(
			t('settings.repositories.deleteConfirm', { name: repo.name }),
			t('common.delete'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (!confirmed) return;

		try {
			await hubApi.deleteRepository(repo.id);
			repositories = repositories.filter((r) => r.id !== repo.id);
			delete editingTokens[repo.id];
			delete editingEnabled[repo.id];
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			errorMessage = msg;
		}
	}

	async function handleAddRepo() {
		if (!newRepo.id.trim() || !newRepo.name.trim() || !newRepo.base_url.trim()) return;

		isAdding = true;
		errorMessage = null;
		try {
			const payload: CreateRepositoryPayload = {
				id: newRepo.id.trim().toLowerCase().replace(/\s+/g, '-'),
				name: newRepo.name.trim(),
				repo_type: newRepo.repo_type,
				base_url: newRepo.base_url.trim(),
				api_token: newRepo.api_token.trim() || undefined,
				enabled: newRepo.enabled
			};
			const created = await hubApi.createRepository(payload);
			repositories = [...repositories, created];
			editingTokens[created.id] = created.api_token || '';
			editingEnabled[created.id] = created.enabled;
			showAddModal = false;
			newRepo = {
				id: '',
				name: '',
				repo_type: 'huggingface',
				base_url: '',
				api_token: '',
				enabled: true
			};
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			errorMessage = msg;
		} finally {
			isAdding = false;
		}
	}

	onMount(() => {
		loadRepositories();
	});
</script>

<div class="flex flex-col h-full min-h-0 w-full">
	<!-- Scrollable content area -->
	<div class="flex-1 overflow-y-auto p-4 sm:p-6 flex flex-col gap-5 min-h-0">
		{#if errorMessage}
			<Alert variant="error" dismissible ondismiss={() => (errorMessage = null)}>
				{errorMessage}
			</Alert>
		{/if}

		{#if isLoading}
			<div class="flex flex-col items-center justify-center p-12 text-[var(--text-muted)] gap-2">
				<Spinner size={24} />
				<span class="text-xs">{t('common.loading')}</span>
			</div>
		{:else}
			<!-- Group 1: Configured Repositories -->
			<div class="relative z-20">
				<SettingGroup title={t('settings.repositories.configuredRepositories')}>
					{#if repositories.length === 0}
						<div
							class="p-5 text-center text-xs text-[var(--text-muted)] bg-[var(--bg-hover)] rounded-2xl"
						>
							{t('settings.repositories.noRepos')}
						</div>
					{:else}
						{#each repositories as repo (repo.id)}
							<SettingRow
								icon={repo.repo_type === 'huggingface' ? Globe : Database}
								vertical={true}
							>
								{#snippet titleSnippet()}
									<div class="flex items-center gap-2 flex-wrap">
										<span
											class="text-xs sm:text-sm font-medium text-[var(--text-primary)] leading-tight"
										>
											{repo.name}
										</span>
										{#if repo.is_builtin}
											<Badge variant="muted">{t('settings.repositories.builtin')}</Badge>
										{:else}
											<Badge variant="info">{t('settings.repositories.custom')}</Badge>
										{/if}
									</div>
									<a
										href={repo.base_url}
										target="_blank"
										rel="noreferrer"
										class="text-[11px] text-[var(--text-muted)] hover:text-[var(--primary)] inline-flex items-center gap-1 no-underline mt-0.5"
									>
										<span>{repo.base_url}</span>
										<ExternalLink size={10} />
									</a>
								{/snippet}

								{#snippet actionSnippet()}
									<div class="flex items-center gap-2">
										{#if !repo.is_builtin}
											<Button
												variant="ghost"
												size="icon-sm"
												onclick={() => deleteRepo(repo)}
												title={t('common.delete')}
												ariaLabel={t('common.delete')}
											>
												<Trash2 size={14} class="text-[var(--danger-text)]" />
											</Button>
										{/if}
										<Toggle
											bind:checked={editingEnabled[repo.id]}
											ariaLabel={editingEnabled[repo.id]
												? t('common.active')
												: t('common.disabled')}
										/>
									</div>
								{/snippet}

								<!-- API Token Configuration -->
								<div class="flex flex-col gap-2 pt-2 w-full">
									<label
										for="token-{repo.id}"
										class="text-[11px] font-medium text-[var(--text-secondary)] block"
									>
										{t('settings.repositories.apiToken')}
									</label>
									<div class="w-full max-w-xl">
										<Input
											id="token-{repo.id}"
											name="monolai_hf_token_{repo.id}"
											type={showToken[repo.id] ? 'text' : 'password'}
											bind:value={editingTokens[repo.id]}
											placeholder={t('settings.repositories.apiTokenPlaceholder')}
											autocomplete="new-password"
											spellcheck={false}
											data-lpignore="true"
											data-1p-ignore="true"
											data-form-type="other"
										>
											{#snippet suffix()}
												<button
													type="button"
													tabindex="-1"
													class="p-0.5 text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors cursor-pointer bg-transparent border-0 flex items-center justify-center"
													onclick={() => (showToken[repo.id] = !showToken[repo.id])}
													aria-label={showToken[repo.id] ? t('common.hide') : t('common.show')}
													title={showToken[repo.id] ? t('common.hide') : t('common.show')}
												>
													{#if showToken[repo.id]}
														<EyeOff size={14} />
													{:else}
														<Eye size={14} />
													{/if}
												</button>
											{/snippet}
										</Input>
									</div>

									{#if repo.repo_type === 'huggingface'}
										<div class="mt-1 space-y-2">
											<button
												type="button"
												class="text-[11px] text-[var(--primary)] hover:underline inline-flex items-center gap-1 cursor-pointer bg-transparent border-none p-0 font-medium"
												onclick={() => (showTokenGuide[repo.id] = !showTokenGuide[repo.id])}
											>
												<KeyRound size={12} />
												<span>{t('settings.repositories.howToTokenTitle')}</span>
											</button>

											{#if showTokenGuide[repo.id]}
												<div
													class="bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-lg p-3 space-y-2 text-xs text-[var(--text-secondary)] max-w-xl"
												>
													<div class="flex items-center justify-between gap-2">
														<span class="font-medium text-[var(--text-primary)]">
															{t('settings.repositories.howToTokenDesc')}
														</span>
														<a
															href="https://huggingface.co/settings/tokens"
															target="_blank"
															rel="noreferrer"
															class="inline-flex items-center gap-1 text-[11px] text-[var(--primary)] hover:underline no-underline shrink-0"
														>
															<span>{t('settings.repositories.openHfTokens')}</span>
															<ExternalLink size={11} />
														</a>
													</div>
													<ol class="space-y-1 pl-4 m-0 list-decimal text-[11px]">
														<li>{t('settings.repositories.howToTokenStep1')}</li>
														<li>{t('settings.repositories.howToTokenStep2')}</li>
														<li>{t('settings.repositories.howToTokenStep3')}</li>
														<li>{t('settings.repositories.howToTokenStep4')}</li>
													</ol>
												</div>
											{/if}
										</div>
									{/if}
								</div>
							</SettingRow>
						{/each}
					{/if}
				</SettingGroup>
			</div>

			<!-- Group 2: Add Repository -->
			<div class="relative z-10">
				<SettingGroup title={t('settings.repositories.groupAdd')}>
					<SettingRow
						icon={Plus}
						title={t('settings.repositories.addCustomRepoTitle')}
						subtitle={t('settings.repositories.addCustomRepoSubtitle')}
					>
						<Button variant="primary" size="sm" onclick={() => (showAddModal = true)}>
							<Plus size={14} />
							<span>{t('common.add')}</span>
						</Button>
					</SettingRow>
				</SettingGroup>
			</div>
		{/if}
	</div>

	<!-- Dedicated Solid Dialog Footer (Completely separate from scroll area) -->
	<div
		class="shrink-0 bg-[var(--bg-surface)] border-t border-[var(--border-color)] px-4 sm:px-6 py-3.5 flex items-center justify-between gap-3 shadow-md"
	>
		<div class="flex items-center gap-2">
			{#if saveSuccess}
				<div
					class="flex items-center gap-1.5 text-xs font-semibold text-[var(--success-text)] animate-in fade-in"
				>
					<ShieldCheck size={16} />
					<span>{t('settings.saveSuccess')}</span>
				</div>
			{:else}
				<span class="text-[11px] text-[var(--text-muted)]">
					{t('settings.changesImmediate')}
				</span>
			{/if}
		</div>

		<Button variant="primary" size="md" loading={isSaving} onclick={saveAllSettings}>
			{#if saveSuccess}
				<Check size={14} />
				<span>{t('common.saved')}</span>
			{:else}
				<Save size={14} />
				<span>{t('common.save')}</span>
			{/if}
		</Button>
	</div>
</div>

<!-- Add Custom Repository Modal -->
{#if showAddModal}
	<Modal
		open={showAddModal}
		title={t('settings.repositories.addCustomModalTitle')}
		onclose={() => (showAddModal = false)}
	>
		<div class="space-y-4">
			<div>
				<label
					for="new-repo-id"
					class="text-xs font-medium text-[var(--text-secondary)] block mb-1"
				>
					{t('settings.repositories.repoId')}
				</label>
				<Input id="new-repo-id" bind:value={newRepo.id} placeholder="e.g. internal-hub" />
			</div>

			<div>
				<label
					for="new-repo-name"
					class="text-xs font-medium text-[var(--text-secondary)] block mb-1"
				>
					{t('settings.repositories.repoName')}
				</label>
				<Input
					id="new-repo-name"
					bind:value={newRepo.name}
					placeholder="e.g. Internal Enterprise Models"
				/>
			</div>

			<div>
				<label
					for="new-repo-type"
					class="text-xs font-medium text-[var(--text-secondary)] block mb-1"
				>
					{t('settings.repositories.repoType')}
				</label>
				<Select bind:value={newRepo.repo_type} options={repoTypeOptions} />
			</div>

			<div>
				<label
					for="new-repo-url"
					class="text-xs font-medium text-[var(--text-secondary)] block mb-1"
				>
					{t('settings.repositories.baseUrl')}
				</label>
				<Input
					id="new-repo-url"
					bind:value={newRepo.base_url}
					placeholder="https://huggingface.co"
				/>
			</div>

			<div>
				<label
					for="new-repo-token"
					class="text-xs font-medium text-[var(--text-secondary)] block mb-1"
				>
					{t('settings.repositories.apiToken')}
				</label>
				<Input
					id="new-repo-token"
					name="monolai_new_repo_token"
					autocomplete="new-password"
					spellcheck={false}
					data-lpignore="true"
					data-1p-ignore="true"
					data-form-type="other"
					type="password"
					bind:value={newRepo.api_token}
					placeholder="Optional API token (Bearer)"
				/>
			</div>

			<div class="flex items-center justify-end gap-2 pt-3 border-t border-[var(--border-color)]">
				<Button variant="secondary" onclick={() => (showAddModal = false)}>
					{t('common.cancel')}
				</Button>
				<Button
					variant="primary"
					onclick={handleAddRepo}
					disabled={isAdding || !newRepo.id || !newRepo.name || !newRepo.base_url}
				>
					{#if isAdding}
						<Spinner size={14} />
					{/if}
					<span>{t('common.add')}</span>
				</Button>
			</div>
		</div>
	</Modal>
{/if}
