<script lang="ts">
	import { Badge, Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { SystemDependencyStatus } from '$lib/types/runtimes';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { AlertTriangle, Check, Copy, Terminal } from '@lucide/svelte';

	interface RuntimeDependenciesProps {
		dependencies: SystemDependencyStatus[];
	}

	let { dependencies = [] }: RuntimeDependenciesProps = $props();

	let copiedMap = $state<Record<string, boolean>>({});

	async function handleCopy(id: string, text: string) {
		const ok = await copyToClipboard(text);
		if (ok) {
			copiedMap[id] = true;
			setTimeout(() => {
				copiedMap[id] = false;
			}, 2000);
		}
	}

	let unsatisfiedDeps = $derived(dependencies.filter((d) => !d.is_satisfied));
</script>

{#if unsatisfiedDeps.length > 0}
	<div class="flex flex-col gap-2">
		{#each unsatisfiedDeps as dep (dep.id)}
			<div
				class="rounded-lg p-3 bg-[var(--warning-light)] border border-[var(--warning-border)] flex flex-col gap-2 text-xs"
			>
				<div class="flex items-center justify-between gap-2">
					<span class="flex items-center gap-1.5 font-semibold text-[var(--warning-text)]">
						<AlertTriangle size={14} class="shrink-0" />
						<span>{dep.name}</span>
					</span>
					<Badge variant="warning" class="text-[10px] shrink-0">
						{t('common.required')}
					</Badge>
				</div>

				<p class="m-0 text-[var(--text-secondary)] text-[11px] leading-relaxed">
					{dep.description}
				</p>

				{#if dep.install_command}
					<div
						class="flex items-center justify-between gap-2 px-2.5 py-1.5 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-md font-mono text-[11px] text-[var(--text-primary)]"
					>
						<span class="flex items-center gap-1.5 min-w-0">
							<Terminal size={12} class="text-[var(--text-muted)] shrink-0" />
							<code class="truncate">{dep.install_command}</code>
						</span>
						<Button
							variant="ghost"
							size="icon-xs"
							onclick={() => handleCopy(dep.id, dep.install_command!)}
							title={t('common.copy')}
							ariaLabel={t('common.copy')}
						>
							{#if copiedMap[dep.id]}
								<Check size={12} class="text-[var(--success-text)]" />
							{:else}
								<Copy size={12} />
							{/if}
						</Button>
					</div>
				{/if}
			</div>
		{/each}
	</div>
{/if}
