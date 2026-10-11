<script lang="ts">
	import { Badge } from '$lib/components/ui';
	import { FolderCheck } from '@lucide/svelte';

	interface RuntimePathHintProps {
		installedPath: string;
		acceleration?: string | null;
	}

	let { installedPath, acceleration = null }: RuntimePathHintProps = $props();
</script>

<div
	class="flex items-center gap-2 px-2.5 py-1.5 rounded-xl bg-[var(--bg-sidebar)] border border-[var(--border-color)] text-[11px] font-mono text-[var(--text-muted)]"
>
	<FolderCheck size={13} class="text-[var(--success-text)] shrink-0" />
	<span class="truncate" title={installedPath}>{installedPath}</span>
	{#if acceleration}
		<Badge variant="success" rounded="rounded-md" class="ml-auto shrink-0">
			{acceleration}
		</Badge>
	{/if}
</div>
