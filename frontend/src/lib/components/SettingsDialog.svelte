<script lang="ts">
	import {
		X,
		Sliders,
		Info,
		Box,
		Settings,
		Check,
		Palette,
		Sparkles,
		ExternalLink,
		ShieldCheck,
		Globe
	} from '@lucide/svelte';
	import Logo from './Logo.svelte';
	import ModelsSettingsTab from './ModelsSettingsTab.svelte';
	import GeneralSettingsTab from './GeneralSettingsTab.svelte';
	import { THEMES, type ThemeId } from '$lib/themes';
	import { APP_VERSION } from '$lib/version';

	let {
		open = $bindable(false),
		themeMode = $bindable<ThemeId>('system'),
		effectiveTheme = 'dark'
	}: {
		open: boolean;
		themeMode: ThemeId;
		effectiveTheme: 'dark' | 'light';
	} = $props();

	let activeTab = $state<'general' | 'appearance' | 'models' | 'about'>('general');
	let modelsTabRef = $state<{ resetView: () => void } | null>(null);

	function selectTab(tab: 'general' | 'appearance' | 'models' | 'about') {
		if (tab === 'models') {
			modelsTabRef?.resetView();
		}
		activeTab = tab;
	}

	function close() {
		open = false;
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			close();
		}
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 bg-black/75 backdrop-blur-md flex items-end sm:items-center justify-center z-[200] p-0 sm:p-4 animate-in fade-in duration-200"
		onclick={handleBackdropClick}
	>
		<div
			class="w-full max-w-[760px] h-[92vh] sm:h-[82vh] sm:max-h-[680px] bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-t-2xl sm:rounded-2xl overflow-hidden flex flex-col shadow-2xl transition-all duration-200"
		>
			<!-- Dialog Header -->
			<div
				class="flex items-center justify-between px-4 sm:px-6 py-3.5 border-b border-[var(--border-color)] shrink-0 bg-[var(--bg-sidebar)]/60 backdrop-blur-md"
			>
				<div class="flex items-center gap-2.5">
					<div
						class="w-8 h-8 rounded-xl bg-[var(--primary)]/15 text-[var(--primary)] flex items-center justify-center border border-[var(--primary)]/30"
					>
						<Sliders size={18} />
					</div>
					<div>
						<h3 class="m-0 text-base sm:text-lg font-bold text-[var(--text-primary)] leading-none">
							Settings
						</h3>
						<p class="m-0 text-[0.7rem] text-[var(--text-muted)] mt-0.5">
							Workspace preferences & system controls
						</p>
					</div>
				</div>
				<button
					type="button"
					class="w-9 h-9 flex items-center justify-center bg-transparent border border-transparent text-[var(--text-muted)] cursor-pointer rounded-xl transition-all duration-150 hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] hover:border-[var(--border-color)]"
					onclick={close}
					aria-label="Close settings"
				>
					<X size={18} />
				</button>
			</div>

			<!-- Dialog Body with Responsive Sidebar / Top Navigation -->
			<div class="flex flex-col sm:flex-row flex-1 overflow-hidden min-h-0">
				<!-- Navigation Bar: Left Sidebar on Desktop (sm:), Horizontal Scroll Bar on Mobile -->
				<div
					class="w-full sm:w-[190px] bg-[var(--bg-sidebar)]/50 border-b sm:border-b-0 sm:border-r border-[var(--border-color)] p-2 sm:p-3 flex sm:flex-col gap-1.5 shrink-0 overflow-x-auto sm:overflow-x-visible no-scrollbar"
				>
					<button
						type="button"
						class="flex items-center justify-center sm:justify-start gap-2.5 px-3.5 py-2.5 text-xs font-medium border-0 rounded-xl cursor-pointer transition-all duration-150 text-left shrink-0 sm:shrink flex-1 sm:flex-none text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] {activeTab ===
						'general'
							? '!text-[var(--primary)] !bg-[var(--primary-light)] font-semibold shadow-xs'
							: ''}"
						onclick={() => selectTab('general')}
					>
						<Settings size={16} />
						<span>General</span>
					</button>

					<button
						type="button"
						class="flex items-center justify-center sm:justify-start gap-2.5 px-3.5 py-2.5 text-xs font-medium border-0 rounded-xl cursor-pointer transition-all duration-150 text-left shrink-0 sm:shrink flex-1 sm:flex-none text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] {activeTab ===
						'appearance'
							? '!text-[var(--primary)] !bg-[var(--primary-light)] font-semibold shadow-xs'
							: ''}"
						onclick={() => selectTab('appearance')}
					>
						<Palette size={16} />
						<span>Appearance</span>
					</button>

					<button
						type="button"
						class="flex items-center justify-center sm:justify-start gap-2.5 px-3.5 py-2.5 text-xs font-medium border-0 rounded-xl cursor-pointer transition-all duration-150 text-left shrink-0 sm:shrink flex-1 sm:flex-none text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] {activeTab ===
						'models'
							? '!text-[var(--primary)] !bg-[var(--primary-light)] font-semibold shadow-xs'
							: ''}"
						onclick={() => selectTab('models')}
					>
						<Box size={16} />
						<span>Models</span>
					</button>

					<button
						type="button"
						class="flex items-center justify-center sm:justify-start gap-2.5 px-3.5 py-2.5 text-xs font-medium border-0 rounded-xl cursor-pointer transition-all duration-150 text-left shrink-0 sm:shrink flex-1 sm:flex-none text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] {activeTab ===
						'about'
							? '!text-[var(--primary)] !bg-[var(--primary-light)] font-semibold shadow-xs'
							: ''}"
						onclick={() => selectTab('about')}
					>
						<Info size={16} />
						<span>About</span>
					</button>
				</div>

				<!-- Content Panel -->
				<div class="flex-1 p-4 sm:p-6 overflow-y-auto min-h-0 bg-[var(--bg-surface)]">
					{#if activeTab === 'general'}
						<GeneralSettingsTab />
					{:else if activeTab === 'appearance'}
						<div class="flex flex-col gap-4">
							<div class="border-b border-[var(--border-color)] pb-3">
								<h4
									class="m-0 text-sm sm:text-base font-bold text-[var(--text-primary)] flex items-center gap-2"
								>
									<Palette size={18} class="text-[var(--primary)]" />
									Interface Themes
								</h4>
								<p class="m-0 text-xs text-[var(--text-muted)] mt-1 leading-relaxed">
									Customize the look and feel of your Monolai workspace with curated themes.
								</p>
							</div>

							<div class="grid grid-cols-1 sm:grid-cols-2 gap-3 mt-1">
								{#each THEMES as theme (theme.id)}
									{@const isSelected = themeMode === theme.id}
									<button
										type="button"
										class="flex flex-col justify-between p-3.5 bg-[var(--bg-primary)] border-2 rounded-xl cursor-pointer text-left transition-all duration-200 relative group hover:border-[var(--primary)]/60 hover:bg-[var(--bg-hover)] {isSelected
											? 'border-[var(--primary)] bg-[var(--primary-light)]/40 shadow-md shadow-[var(--primary)]/5'
											: 'border-[var(--border-color)]'}"
										onclick={() => (themeMode = theme.id)}
									>
										<div class="flex flex-col gap-1.5 w-full">
											<div class="flex items-center justify-between w-full gap-2">
												<div class="flex items-center gap-2 min-w-0">
													<span
														class="w-4 h-4 rounded-full border border-black/20 shrink-0 inline-block shadow-xs transition-transform group-hover:scale-110"
														style="background-color: {theme.preview.primary}"
													></span>
													<span
														class="text-xs sm:text-sm font-bold text-[var(--text-primary)] truncate"
													>
														{theme.name}
													</span>
												</div>
												{#if isSelected}
													<span
														class="w-5 h-5 rounded-full bg-[var(--primary)] text-white flex items-center justify-center shrink-0 shadow-xs"
													>
														<Check size={12} />
													</span>
												{/if}
											</div>

											<p
												class="text-[0.725rem] text-[var(--text-muted)] m-0 line-clamp-2 leading-relaxed"
											>
												{theme.description}
											</p>
										</div>

										<!-- Color Palette Swatches -->
										<div
											class="flex items-center justify-between mt-3 pt-2 border-t border-[var(--border-color)]/60 w-full text-[0.7rem] text-[var(--text-muted)]"
										>
											<span class="font-medium text-[0.675rem] uppercase tracking-wider opacity-75"
												>Palette</span
											>
											<div class="flex items-center gap-1.5">
												<span
													class="w-3 h-3 rounded-full border border-white/20 shadow-2xs"
													style="background-color: {theme.preview.bg}"
													title="Background ({theme.preview.bg})"
												></span>
												<span
													class="w-3 h-3 rounded-full border border-white/20 shadow-2xs"
													style="background-color: {theme.preview.surface}"
													title="Surface ({theme.preview.surface})"
												></span>
												<span
													class="w-3 h-3 rounded-full border border-white/20 shadow-2xs"
													style="background-color: {theme.preview.primary}"
													title="Accent ({theme.preview.primary})"
												></span>
											</div>
										</div>
									</button>
								{/each}
							</div>
						</div>
					{:else if activeTab === 'models'}
						<ModelsSettingsTab bind:this={modelsTabRef} />
					{:else if activeTab === 'about'}
						<div class="flex flex-col items-center gap-5 py-4 text-center max-w-[420px] mx-auto">
							<div class="flex flex-col items-center gap-3">
								<div
									class="relative p-3 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-2xl shadow-sm"
								>
									<Logo size={56} />
								</div>
								<div>
									<h4 class="m-0 text-xl font-extrabold text-[var(--text-primary)]">
										Monolai AI Suite
									</h4>
								</div>
								<p class="m-0 text-xs text-[var(--text-muted)] leading-relaxed">
									Lightweight local AI model manager and inference server runner.
								</p>
							</div>

							<a
								href="https://github.com/odevsa/monolai"
								target="_blank"
								rel="noopener noreferrer"
								class="flex items-center justify-center gap-2.5 w-full py-2.5 px-4 bg-[var(--bg-primary)] hover:bg-[var(--bg-hover)] border border-[var(--border-color)] hover:border-[var(--primary)] rounded-xl text-xs font-bold text-[var(--text-primary)] no-underline transition-all duration-150 shadow-2xs group"
							>
								<svg
									class="w-4 h-4 fill-current text-[var(--text-primary)] group-hover:scale-110 transition-transform"
									viewBox="0 0 24 24"
								>
									<path
										d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"
									/>
								</svg>
								<span>GitHub Repository</span>
								<ExternalLink
									size={13}
									class="text-[var(--text-muted)] group-hover:text-[var(--primary)] transition-colors"
								/>
							</a>

							<div
								class="w-full flex flex-col gap-2 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl p-3.5 text-left"
							>
								<div
									class="flex items-center justify-between text-xs py-1.5 border-b border-[var(--border-color)]/60"
								>
									<span class="text-[var(--text-muted)] flex items-center gap-1.5"> Version </span>
									<span class="text-[var(--text-primary)] font-semibold">
										v{APP_VERSION}
									</span>
								</div>
								<div
									class="flex items-center justify-between text-xs py-1.5 border-b border-[var(--border-color)]/60"
								>
									<span class="text-[var(--text-muted)] flex items-center gap-1.5">
										Privacy first
									</span>
									<span class="text-[var(--text-primary)] font-semibold"> Local </span>
								</div>
								<div
									class="flex items-center justify-between text-xs py-1.5 border-b border-[var(--border-color)]/60"
								>
									<span class="text-[var(--text-muted)]">Backend</span>
									<span class="text-[var(--text-primary)] font-semibold">Rust</span>
								</div>
								<div class="flex items-center justify-between text-xs py-1.5">
									<span class="text-[var(--text-muted)]">Frontend</span>
									<span class="text-[var(--text-primary)] font-semibold">Svelte + Tailwind</span>
								</div>
							</div>
						</div>
					{/if}
				</div>
			</div>
		</div>
	</div>
{/if}
