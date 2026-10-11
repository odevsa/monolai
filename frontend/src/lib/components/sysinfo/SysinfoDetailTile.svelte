<script lang="ts">
	interface SysinfoDetailTileProps {
		label: string;
		value: string;
	}

	let { label, value }: SysinfoDetailTileProps = $props();
</script>

<div class="flex flex-col gap-1.5 p-3.5 bg-[var(--bg-surface-hover)] rounded-xl border-0">
	<span class="text-[0.75rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]">
		{label}
	</span>
	<span class="text-sm font-semibold text-[var(--text-primary)] truncate" title={value}>
		{value}
	</span>
</div>
