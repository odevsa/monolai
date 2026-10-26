<script lang="ts">
	type Variant = 'primary' | 'warning' | 'danger' | 'success';

	let {
		text,
		variant = 'primary'
	}: {
		text: string;
		variant?: Variant;
	} = $props();

	const variantStyles: Record<Variant, string> = {
		primary: 'bg-[var(--primary)] text-[var(--primary-text)]',
		warning: 'bg-[var(--warning)] text-[var(--primary-text)]',
		danger: 'bg-[var(--danger)] text-[var(--primary-text)]',
		success: 'bg-[var(--success)] text-[var(--primary-text)]'
	};
</script>

<div class="absolute top-0 right-0 w-32 h-32 overflow-hidden pointer-events-none">
	<div
		class="absolute top-7 -right-8 w-38 {variantStyles[
			variant
		]} text-xs font-bold text-center py-1.5 rotate-45 shadow-md uppercase"
	>
		{text}
	</div>
</div>
