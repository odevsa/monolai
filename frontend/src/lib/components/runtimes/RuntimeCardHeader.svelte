<script lang="ts">
	import audioIcon from '$lib/assets/runtimes/audio-cpp.svg';
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
		'sd-cpp': sdIcon,
		'audio-cpp': audioIcon
	};

	let iconSrc = $derived(localIcons[runtime.id] || runtime.icon);
</script>

<div class="flex items-start justify-between gap-3">
	<div class="flex items-start gap-3.5 min-w-0 flex-1">
		<div
			class="w-14 h-14 rounded-2xl bg-[var(--bg-surface-hover)] border border-[var(--border-color)] flex items-center justify-center p-2.5 shrink-0 overflow-hidden shadow-xs"
		>
			{#if runtime.id === 'llama-cpp'}
				<svg
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					xmlns="http://www.w3.org/2000/svg"
					class="w-full h-full text-[var(--text-primary)] transition-colors duration-150"
					role="img"
					aria-label={runtime.name}
				>
					<path
						fill="currentColor"
						d="M22 15.067L18.808 20.6L18.805 20.598C18.242 21.469 17.273 22 16.229 22H6L10 15.067H22Z"
					/>
					<path
						fill="currentColor"
						d="M10 15.067H2L8.653 3.534C9.201 2.585 10.214 2 11.31 2H17.538L10 15.067Z"
					/>
				</svg>
			{:else if runtime.id === 'audio-cpp'}
				<svg
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					xmlns="http://www.w3.org/2000/svg"
					class="w-full h-full text-[var(--text-primary)] transition-colors duration-150"
					role="img"
					aria-label={runtime.name}
				>
					<path
						d="M12 3V21M8 8V16M16 8V16M4 11V13M20 11V13"
						stroke="currentColor"
						stroke-width="2.2"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
				</svg>
			{:else if iconSrc}
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
						<span>{runtime.installed_version}</span>
						{#if runtime.has_update}
							<span class="text-[var(--text-muted)]">&rarr;</span>
							{#if runtime.release_notes_url}
								<a
									href={runtime.release_notes_url}
									target="_blank"
									rel="noreferrer"
									class="text-[var(--primary)] hover:underline"
									title={t('runtimes.viewReleaseNotes')}
								>
									{runtime.version}
								</a>
							{:else}
								<span class="text-[var(--primary)]">{runtime.version}</span>
							{/if}
						{/if}
					{:else}
						<span>{runtime.version || '—'}</span>
					{/if}
				</div>
			</div>
		</div>
	</div>
</div>
