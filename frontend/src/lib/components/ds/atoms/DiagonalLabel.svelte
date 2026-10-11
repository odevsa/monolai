<script lang="ts">
	let {
		text
	}: {
		text: string;
	} = $props();
</script>

<div class="absolute top-0 right-0 w-32 h-32 overflow-hidden pointer-events-none">
	<div
		class="absolute top-6 -right-10 w-38 bg-[var(--primary)] text-[var(--bg-primary)] text-xs font-bold text-center py-1.5 rotate-45 shadow-md uppercase"
	>
		{text}
	</div>
</div>
