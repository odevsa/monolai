<script lang="ts">
	import { t } from '$lib/i18n';
	import { Sparkles } from '@lucide/svelte';
</script>

<div class="m-auto flex flex-col items-center justify-center text-center gap-4 max-w-[480px]">
	<div class="flex items-center justify-center bg-transparent text-[var(--primary)] mb-1">
		<Sparkles
			size={64}
			class="animate-sparkle-glow transition-transform duration-1000 ease-[cubic-bezier(0.34,1.56,0.64,1)] hover:scale-110 hover:rotate-4"
		/>
	</div>
	<h1 class="text-3xl font-semibold text-[var(--text-primary)] m-0">{t('chat.emptyGreeting')}</h1>
	<p class="text-base text-[var(--text-muted)] m-0">{t('chat.emptyHeroSubtitle')}</p>
</div>
