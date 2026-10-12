<script lang="ts">
	let {
		variant = 'success',
		size = 'md',
		pulse = false,
		title = '',
		class: className = ''
	}: {
		variant?: 'success' | 'warning' | 'danger' | 'primary' | 'muted';
		size?: 'sm' | 'md' | 'lg';
		pulse?: boolean;
		title?: string;
		class?: string;
	} = $props();

	const variantClasses = {
		success: 'bg-[var(--success)] shadow-[0_0_6px_var(--success)]',
		warning: 'bg-[var(--warning)] shadow-[0_0_6px_var(--warning)]',
		danger: 'bg-[var(--danger)] shadow-[0_0_6px_var(--danger)]',
		primary: 'bg-[var(--primary)] shadow-[0_0_6px_var(--primary-focus)]',
		muted: 'bg-[var(--text-muted)]'
	};

	const sizeClasses = {
		sm: 'w-1.5 h-1.5',
		md: 'w-2 h-2',
		lg: 'w-2.5 h-2.5'
	};
</script>

<span
	class="rounded-full shrink-0 inline-block {sizeClasses[size]} {variantClasses[variant]} {pulse
		? 'animate-pulse'
		: ''} {className}"
	{title}
></span>
