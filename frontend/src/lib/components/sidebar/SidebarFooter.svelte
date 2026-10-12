<script lang="ts">
	import { APP_VERSION } from '$lib/version';
</script>

<div
	class="p-3 pb-[max(0.75rem,env(safe-area-inset-bottom,0.75rem))] flex items-center justify-between border-t border-[var(--border-color)] shrink-0"
>
	<span class="text-[0.725rem] text-[var(--text-muted)]">v{APP_VERSION}</span>
</div>
