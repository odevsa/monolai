<script lang="ts">
	import llamaIcon from '$lib/assets/runtimes/llama-cpp.svg';
	import sdIcon from '$lib/assets/runtimes/sd-cpp.svg';
	import { t } from '$lib/i18n';
	import type { Runtime } from '$lib/types/runtimes';
	import { Cpu } from '@lucide/svelte';

	interface RuntimeCardHeaderProps {
		runtime: Runtime;
	}

	let { runtime }: RuntimeCardHeaderProps = $props();

	const localIcons: Record<string, string> = {
		'llama-cpp': llamaIcon,
		'sd-cpp': sdIcon
	};

	let iconSrc = $derived(localIcons[runtime.id] || runtime.icon);
</script>

<div class="flex items-start justify-between gap-3">
	<div class="flex items-start gap-3.5 min-w-0 flex-1">
		<div
			class="w-14 h-14 rounded-2xl bg-[var(--bg-surface-hover)] border border-[var(--border-color)] flex items-center justify-center p-2.5 shrink-0 overflow-hidden shadow-xs"
		>
			{#if iconSrc}
				<img src={iconSrc} alt={runtime.name} class="w-full h-full object-contain" />
			{:else}
				<Cpu size={26} class="text-[var(--primary)]" />
			{/if}
		</div>

		<div class="min-w-0 flex-1">
			<div class="flex flex-col gap-0.5">
				<div class="flex items-center gap-2 flex-wrap">
					<h3
						class="m-0 text-base font-bold text-[var(--text-primary)] truncate"
						title={runtime.name}
					>
						{runtime.name}
					</h3>
				</div>

				{#if runtime.website}
					<a
						href={runtime.website}
						target="_blank"
						rel="noopener noreferrer"
						class="text-[var(--primary)] hover:text-[var(--primary-hover)] font-semibold text-xs transition-colors truncate"
						title={t('runtimes.visitRepo')}
					>
						{runtime.website}
					</a>
				{/if}

				<div class="flex items-center gap-1.5 text-[var(--text-muted)] text-xs flex-wrap font-mono">
					{#if runtime.is_installed && runtime.installed_version}
						<span>v{runtime.installed_version}</span>
						{#if runtime.has_update}
							<span class="text-[var(--text-muted)]">&rarr;</span>
							<span class="text-[var(--primary)]">v{runtime.version}</span>
						{/if}
					{:else}
						<span>v{runtime.version}</span>
					{/if}
				</div>
			</div>
		</div>
	</div>
</div>
