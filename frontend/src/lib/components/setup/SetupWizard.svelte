<script lang="ts">
	import { Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ConfigStatus } from '$lib/types/config';
	import { Check } from '@lucide/svelte';
	import SetupConfigPathInfo from './SetupConfigPathInfo.svelte';
	import SetupNetworkSection from './SetupNetworkSection.svelte';
	import SetupStorageSection from './SetupStorageSection.svelte';

	let {
		configStatus,
		isStorageConfigured = false,
		saving = false,
		modelsDir = $bindable(''),
		runtimesDir = $bindable(''),
		host = $bindable('0.0.0.0'),
		port = $bindable(8080),
		onsubmit,
		onResetModels,
		onResetRuntimes
	}: {
		configStatus: ConfigStatus;
		isStorageConfigured: boolean;
		saving?: boolean;
		modelsDir: string;
		runtimesDir: string;
		host: string;
		port: number;
		onsubmit: () => void;
		onResetModels: () => void;
		onResetRuntimes: () => void;
	} = $props();
</script>

<form
	onsubmit={(e) => {
		e.preventDefault();
		onsubmit();
	}}
	class="flex flex-col gap-5"
>
	<SetupStorageSection
		{configStatus}
		{isStorageConfigured}
		bind:modelsDir
		bind:runtimesDir
		{onResetModels}
		{onResetRuntimes}
	/>

	<SetupNetworkSection
		isDocker={!!configStatus.is_docker}
		{isStorageConfigured}
		bind:host
		bind:port
	/>

	{#if configStatus.expected_path}
		<SetupConfigPathInfo expectedPath={configStatus.expected_path} />
	{/if}

	<div class="pt-1">
		<Button
			type="submit"
			variant="primary"
			size="lg"
			loading={saving}
			disabled={saving}
			class="w-full shadow-md"
		>
			<Check size={16} />
			<span>
				{t(isStorageConfigured ? 'setup.saveAndContinue' : 'setup.saveAndStart')}
			</span>
		</Button>
	</div>
</form>
