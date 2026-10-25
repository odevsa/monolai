<script lang="ts">
	import { hubApi } from '$lib/api/hub';
	import { Alert, Badge, Button, Input, Modal, Select, Spinner, Toggle } from '$lib/components/ui';
	import { askConfirm } from '$lib/confirmStore';
	import { t } from '$lib/i18n';
	import type { CreateRepositoryPayload, RepositoryRecord } from '$lib/types/models';
	import { ExternalLink, Plus, Save, Trash2 } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import SettingGroup from './SettingGroup.svelte';

	let repositories = $state<RepositoryRecord[]>([]);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	let successMessage = $state<string | null>(null);

	// Custom repo modal
	let showAddModal = $state(false);
	let isAdding = $state(false);
	let newRepo = $state<{
		id: string;
		name: string;
		repo_type: string;
		base_url: string;
		mirror_url: string;
		api_token: string;
		enabled: boolean;
	}>({
		id: '',
		name: '',
		repo_type: 'huggingface',
		base_url: '',
		mirror_url: '',
		api_token: '',
		enabled: true
	});

	// Token & mirror edit tracking per repo id
	let editingTokens = $state<Record<string, string>>({});
	let editingMirrors = $state<Record<string, string>>({});
	let savingRepoId = $state<string | null>(null);

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
			const mirrors: Record<string, string> = {};
			for (const r of repositories) {
				tokens[r.id] = r.api_token || '';
				mirrors[r.id] = r.mirror_url || '';
			}
			editingTokens = tokens;
			editingMirrors = mirrors;
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			errorMessage = msg;
		} finally {
			isLoading = false;
		}
	}

	async function toggleEnabled(repo: RepositoryRecord) {
		try {
			const updated = await hubApi.updateRepository(repo.id, {
				enabled: !repo.enabled
			});
			repositories = repositories.map((r) => (r.id === repo.id ? updated : r));
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			errorMessage = msg;
		}
	}

	async function saveRepoSettings(repo: RepositoryRecord) {
		savingRepoId = repo.id;
		errorMessage = null;
		successMessage = null;
		try {
			const tokenVal = editingTokens[repo.id]?.trim() || undefined;
			const mirrorVal = editingMirrors[repo.id]?.trim() || undefined;
			const updated = await hubApi.updateRepository(repo.id, {
				api_token: tokenVal,
				mirror_url: mirrorVal
			});
			repositories = repositories.map((r) => (r.id === repo.id ? updated : r));
			successMessage = t('common.saved');
			setTimeout(() => {
				if (successMessage) successMessage = null;
			}, 3000);
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			errorMessage = msg;
		} finally {
			savingRepoId = null;
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
				mirror_url: newRepo.mirror_url.trim() || undefined,
				api_token: newRepo.api_token.trim() || undefined,
				enabled: newRepo.enabled
			};
			const created = await hubApi.createRepository(payload);
			repositories = [...repositories, created];
			editingTokens[created.id] = created.api_token || '';
			editingMirrors[created.id] = created.mirror_url || '';
			showAddModal = false;
			newRepo = {
				id: '',
				name: '',
				repo_type: 'huggingface',
				base_url: '',
				mirror_url: '',
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

<div class="flex-1 overflow-y-auto p-4 sm:p-6 space-y-6">
	<!-- Header / Description -->
	<div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
		<div>
			<h4 class="text-sm font-semibold text-[var(--text-primary)] m-0">
				{t('settings.repositories.title')}
			</h4>
			<p class="text-xs text-[var(--text-muted)] mt-1 mb-0">
				{t('settings.repositories.subtitle')}
			</p>
		</div>

		<Button variant="primary" size="sm" onclick={() => (showAddModal = true)}>
			<Plus size={14} />
			<span>{t('common.add')}</span>
		</Button>
	</div>

	{#if errorMessage}
		<Alert variant="error" title={t('common.error')}>
			{errorMessage}
		</Alert>
	{/if}

	{#if successMessage}
		<Alert variant="success" title={t('common.saved')}>
			{successMessage}
		</Alert>
	{/if}

	{#if isLoading}
		<div class="flex flex-col items-center justify-center p-12 text-[var(--text-muted)] gap-2">
			<Spinner size={24} />
			<span class="text-xs">{t('common.loading')}</span>
		</div>
	{:else}
		<SettingGroup title={t('settings.repositories.configuredRepositories')}>
			<div class="space-y-4 p-4">
				{#each repositories as repo (repo.id)}
					<div
						class="bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-xl p-4 transition-all duration-150 hover:border-[var(--border-hover)] space-y-3"
					>
						<!-- Top row: Status toggle, Name, Badge, Delete -->
						<div class="flex items-center justify-between gap-3 flex-wrap">
							<div class="flex items-center gap-3">
								<Toggle
									checked={repo.enabled}
									onchange={() => toggleEnabled(repo)}
									ariaLabel={repo.enabled ? t('common.active') : t('common.disabled')}
								/>
								<div>
									<div class="flex items-center gap-2">
										<span class="text-sm font-bold text-[var(--text-primary)] leading-tight">
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
										class="text-[11px] text-[var(--text-muted)] hover:text-[var(--primary)] flex items-center gap-1 no-underline mt-0.5"
									>
										<span>{repo.base_url}</span>
										<ExternalLink size={10} />
									</a>
								</div>
							</div>

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
							</div>
						</div>

						<!-- Config fields row: Mirror URL & API Token -->
						<div class="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-1">
							<div>
								<label
									for="mirror-{repo.id}"
									class="text-[11px] font-medium text-[var(--text-secondary)] block mb-1"
								>
									{t('settings.repositories.mirrorUrl')}
								</label>
								<Input
									id="mirror-{repo.id}"
									bind:value={editingMirrors[repo.id]}
									placeholder="e.g. https://hf-mirror.com"
								/>
							</div>

							<div>
								<label
									for="token-{repo.id}"
									class="text-[11px] font-medium text-[var(--text-secondary)] block mb-1"
								>
									{t('settings.repositories.apiToken')}
								</label>
								<div class="flex items-center gap-2">
									<Input
										id="token-{repo.id}"
										type="password"
										bind:value={editingTokens[repo.id]}
										placeholder={t('settings.repositories.apiTokenPlaceholder')}
									/>
									<Button
										variant="secondary"
										size="sm"
										onclick={() => saveRepoSettings(repo)}
										disabled={savingRepoId === repo.id}
										title={t('common.save')}
									>
										{#if savingRepoId === repo.id}
											<Spinner size={13} />
										{:else}
											<Save size={13} />
										{/if}
									</Button>
								</div>
							</div>
						</div>
					</div>
				{/each}
			</div>
		</SettingGroup>
	{/if}
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
					for="new-repo-mirror"
					class="text-xs font-medium text-[var(--text-secondary)] block mb-1"
				>
					{t('settings.repositories.mirrorUrl')}
				</label>
				<Input
					id="new-repo-mirror"
					bind:value={newRepo.mirror_url}
					placeholder="Optional mirror URL"
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
