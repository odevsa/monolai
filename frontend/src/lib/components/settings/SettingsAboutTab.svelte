<script lang="ts">
	import { configApi } from '$lib/api/config';
	import { Logo } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ConfigStatus } from '$lib/types/config';
	import { APP_VERSION } from '$lib/version';
	import { Cpu, Network, Palette, ScrollText, ShieldCheck, Sparkles } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import SettingGroup from './SettingGroup.svelte';
	import SettingRow from './SettingRow.svelte';

	let configStatus = $state<ConfigStatus | null>(null);

	onMount(() => {
		configApi
			.getStatus()
			.then((data) => {
				if (data) configStatus = data;
			})
			.catch(() => {});
	});
</script>

<div class="flex-1 overflow-y-auto p-4 sm:p-6 flex flex-col gap-5 min-h-0 w-full items-center">
	<!-- Featured App Header -->
	<div class="flex flex-col items-center gap-4 py-2 text-center max-w-sm">
		<Logo size={64} />
		<div>
			<h4 class="m-0 text-base font-bold text-[var(--text-primary)]">
				{t('settings.aboutAppName')}
			</h4>
			<p class="m-0 text-xs text-[var(--text-muted)] mt-0.5 leading-relaxed">
				{t('settings.aboutDescription')}
			</p>
		</div>
	</div>

	<!-- System Specifications -->
	<SettingGroup title={t('settings.groupAboutSpecs')}>
		<SettingRow icon={Sparkles} title={t('settings.appVersion')}>
			<span class="text-xs font-semibold text-[var(--text-secondary)]">
				v{APP_VERSION}
			</span>
		</SettingRow>

		<SettingRow icon={ShieldCheck} title={t('settings.privacy')}>
			<span class="text-xs font-semibold text-[var(--text-secondary)]">
				{t('settings.privacyLocal')}
			</span>
		</SettingRow>

		<SettingRow icon={Cpu} title={t('settings.backend')}>
			<span class="text-xs font-mono font-medium text-[var(--text-secondary)]"> Rust </span>
		</SettingRow>

		<SettingRow icon={Palette} title={t('settings.frontend')}>
			<span class="text-xs font-mono font-medium text-[var(--text-secondary)]">
				Svelte 5 + Tailwind
			</span>
		</SettingRow>
	</SettingGroup>

	<!-- Resources & Links -->
	<SettingGroup title={t('settings.groupAboutResources')}>
		<SettingRow
			icon={ScrollText}
			title={t('settings.apiDocumentation')}
			subtitle="{window.location.hostname}:{window.location.port}/api/swagger"
			href="{window.location.origin}/api/swagger"
		/>

		<SettingRow
			title={t('settings.githubRepo')}
			subtitle="github.com/odevsa/monolai"
			href="https://github.com/odevsa/monolai"
		>
			{#snippet iconSnippet()}
				<svg class="w-4 h-4 fill-current text-[var(--primary)] shrink-0" viewBox="0 0 24 24">
					<path
						d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"
					/>
				</svg>
			{/snippet}
		</SettingRow>
	</SettingGroup>
</div>
