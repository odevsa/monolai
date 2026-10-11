<script lang="ts">
	import { goto } from '$app/navigation';
	import { chatsApi, getFeatureModels, settingsApi } from '$lib/api';
	import { streamChatCompletion } from '$lib/api/openai';
	import { ContextIndicator, MarkdownRenderer, ModelBadge } from '$lib/components/ds';
	import { showAlert } from '$lib/confirmStore';
	import { syncRecentChats } from '$lib/headerStore';
	import { t } from '$lib/i18n';
	import { runningModelsState } from '$lib/runningModelsStore';
	import { chatsState } from '$lib/state';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { generateUUID } from '$lib/utils/common';
	import {
		DEFAULT_SYSTEM_PROMPT,
		REASONING_PRESETS,
		getModelContextLimit,
		prepareSlidingWindowContext,
		type ReasoningPreset,
		type TokenUsageDetails
	} from '$lib/utils/context';
	import {
		ArrowUp,
		Box,
		Check,
		ChevronDown,
		ChevronRight,
		CircleAlert,
		Clock,
		Copy,
		Lightbulb,
		Plus,
		RotateCw,
		Sparkles,
		Square,
		Trash2,
		Zap
	} from '@lucide/svelte';
	import { onMount, tick } from 'svelte';

	let { chatId = '' }: { chatId?: string } = $props();

	interface Message {
		id: string;
		role: 'user' | 'assistant';
		content: string;
		reasoning?: string;
		reasoningDuration?: string;
		tokens?: number;
		duration?: string;
		speed?: string;
		modelTags?: string[];
		status?: string;
	}

	interface RegisteredModel {
		id: string;
		runtime: string;
		flags: string;
	}

	interface RuntimeManifestItem {
		id: string;
		name: string;
		description: string;
		features?: string[];
	}

	let effectiveTheme = $state<'dark' | 'light'>('dark');

	let messages = $state<Message[]>([]);
	let inputMessage = $state('');
	let isGenerating = $state(false);
	let currentAbortController: AbortController | null = null;

	let registeredChatModels = $state<RegisteredModel[]>([]);
	let selectedModel = $state<string>('');
	let isModelMenuOpen = $state(false);
	let modelWrapper: HTMLDivElement | null = $state(null);

	let messagesContainer: HTMLDivElement | null = $state(null);
	let textareaRef: HTMLTextAreaElement | null = $state(null);
	let inputSectionRef = $state<HTMLDivElement | null>(null);
	let inputSectionHeight = $state<number>(110);
	let scrollbarWidth = $state(0);

	// Mobile virtual keyboard detection state
	let isKeyboardOpen = $state(false);
	let isTextareaFocused = $state(false);
	let baseViewportHeight = 0;

	// Prompt History State (ArrowUp / ArrowDown)
	let promptHistory = $state<string[]>([]);
	let historyIndex = $state<number>(-1);
	let currentDraft = $state<string>('');

	// Reasoning Effort Selection (Default: Medium, as requested)
	let selectedReasoning = $state<ReasoningPreset['id']>('medium');
	let reasoningMenuOpen = $state(false);
	let collapsedReasoning = $state<Record<string, boolean>>({});

	function toggleReasoningCollapse(msgId: string) {
		collapsedReasoning[msgId] = !collapsedReasoning[msgId];
	}

	// Dynamic auto-scrolling state for chat and reasoning
	let autoScrollChat = $state(true);
	let autoScrollReasoning = $state<Record<string, boolean>>({});
	let reasoningContainers: Record<string, HTMLElement> = {};

	function registerReasoningContainer(node: HTMLElement, msgId: string) {
		reasoningContainers[msgId] = node;
		if (autoScrollReasoning[msgId] !== false) {
			node.scrollTop = node.scrollHeight;
		}
		return {
			destroy() {
				delete reasoningContainers[msgId];
			}
		};
	}

	function handleReasoningScroll(e: Event, msgId: string) {
		const target = e.currentTarget as HTMLElement;
		if (!target) return;
		const distanceFromBottom = target.scrollHeight - target.scrollTop - target.clientHeight;
		if (distanceFromBottom > 35) {
			autoScrollReasoning[msgId] = false;
		} else if (distanceFromBottom <= 15) {
			autoScrollReasoning[msgId] = true;
		}
	}
	let submenuRef: HTMLDivElement | null = $state(null);
	let submenuDirection = $state<'right' | 'left'>('right');

	function checkSubmenuPlacement() {
		if (submenuRef && typeof window !== 'undefined') {
			const rect = submenuRef.getBoundingClientRect();
			if (rect.right > window.innerWidth - 16) {
				submenuDirection = 'left';
			} else {
				submenuDirection = 'right';
			}
		}
	}

	let currentReasoningPreset = $derived(
		REASONING_PRESETS.find((r) => r.id === selectedReasoning) || REASONING_PRESETS[3]
	);

	// System Prompt state
	let systemPrompt = $state<string>(DEFAULT_SYSTEM_PROMPT);

	async function loadSystemPrompt() {
		try {
			const settings = await settingsApi.getAll();
			if (settings?.system_prompt && settings.system_prompt.trim()) {
				systemPrompt = settings.system_prompt.trim();
			}
		} catch {
			// Fallback to default
		}
	}

	// Dynamic context tracking using model's context window limit and real-time KV metrics
	let currentModelRecord = $derived(registeredChatModels.find((m) => m.id === selectedModel));
	let contextLimit = $derived(getModelContextLimit(currentModelRecord?.flags));
	let liveUsage = $state<Partial<TokenUsageDetails>>({});
	let contextInfo = $derived(
		prepareSlidingWindowContext(
			messages,
			systemPrompt,
			contextLimit,
			currentReasoningPreset.maxTokens || 1024,
			liveUsage
		)
	);

	function adjustTextareaHeight() {
		if (textareaRef) {
			textareaRef.style.height = 'auto';
			const maxHeight = 160;
			textareaRef.style.height = `${Math.min(textareaRef.scrollHeight, maxHeight)}px`;
		}
	}

	function updateKeyboardStatus() {
		if (typeof window === 'undefined') return;

		if (inputSectionRef) {
			inputSectionHeight = inputSectionRef.offsetHeight;
		}

		const isMobile = window.innerWidth <= 768 || window.matchMedia('(max-width: 768px)').matches;
		if (!isMobile) {
			if (isKeyboardOpen) isKeyboardOpen = false;
			return;
		}

		const vv = window.visualViewport;
		if (vv) {
			if (!isTextareaFocused && vv.height > baseViewportHeight) {
				baseViewportHeight = vv.height;
			}

			const heightShrunk = baseViewportHeight > 0 && baseViewportHeight - vv.height > 80;
			const screenShrunk = window.screen?.height ? window.screen.height - vv.height > 160 : false;
			const keyboardDetected = isTextareaFocused && (heightShrunk || screenShrunk);

			if (keyboardDetected !== isKeyboardOpen) {
				isKeyboardOpen = keyboardDetected;
				if (keyboardDetected && messages.length > 0) {
					setTimeout(() => {
						scrollToBottom();
					}, 60);
					setTimeout(() => {
						scrollToBottom();
					}, 220);
				}
			}
		} else {
			if (isTextareaFocused !== isKeyboardOpen) {
				isKeyboardOpen = isTextareaFocused;
				if (isTextareaFocused && messages.length > 0) {
					setTimeout(() => {
						scrollToBottom();
					}, 60);
					setTimeout(() => {
						scrollToBottom();
					}, 220);
				}
			}
		}
	}

	function handleTextareaFocus() {
		isTextareaFocused = true;
		updateKeyboardStatus();
		if (typeof window !== 'undefined') {
			window.scrollTo(0, 0);
			setTimeout(() => {
				window.scrollTo(0, 0);
				updateKeyboardStatus();
				if (messages.length > 0) {
					scrollToBottom();
				}
			}, 100);
			setTimeout(() => {
				updateKeyboardStatus();
				if (messages.length > 0) {
					scrollToBottom();
				}
			}, 300);
		}
	}

	function handleTextareaBlur() {
		isTextareaFocused = false;
		setTimeout(() => {
			updateKeyboardStatus();
		}, 100);
	}

	$effect(() => {
		if (!inputSectionRef) return;
		const ro = new ResizeObserver((entries) => {
			for (const entry of entries) {
				inputSectionHeight =
					entry.borderBoxSize?.[0]?.blockSize ??
					entry.contentRect?.height ??
					inputSectionRef?.offsetHeight ??
					110;
			}
		});
		ro.observe(inputSectionRef);
		return () => ro.disconnect();
	});

	$effect(() => {
		if (typeof window !== 'undefined') {
			baseViewportHeight = window.visualViewport?.height || window.innerHeight;

			const onVisualResize = () => {
				if (window.scrollY !== 0) {
					window.scrollTo(0, 0);
				}
				updateKeyboardStatus();
				if (messages.length > 0) {
					scrollToBottom();
				}
			};

			if (window.visualViewport) {
				window.visualViewport.addEventListener('resize', onVisualResize);
				window.visualViewport.addEventListener('scroll', onVisualResize);
			}
			window.addEventListener('resize', updateKeyboardStatus);

			return () => {
				window.visualViewport?.removeEventListener('resize', onVisualResize);
				window.visualViewport?.removeEventListener('scroll', onVisualResize);
				window.removeEventListener('resize', updateKeyboardStatus);
			};
		}
	});

	function loadPromptHistory() {
		if (typeof localStorage !== 'undefined') {
			try {
				const saved = localStorage.getItem('monolai:prompt_history');
				if (saved) {
					promptHistory = JSON.parse(saved);
				}
			} catch {
				promptHistory = [];
			}
		}
	}

	function savePromptToHistory(text: string) {
		if (!text) return;
		promptHistory = [text, ...promptHistory.filter((p) => p !== text)].slice(0, 50);
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem('monolai:prompt_history', JSON.stringify(promptHistory));
		}
		historyIndex = -1;
		currentDraft = '';
	}

	function clearPromptHistory() {
		promptHistory = [];
		historyIndex = -1;
		currentDraft = '';
		if (typeof localStorage !== 'undefined') {
			localStorage.removeItem('monolai:prompt_history');
		}
	}

	// Load chat models using the official OpenAI SDK and retrieve saved model from localStorage
	async function loadChatModels() {
		try {
			registeredChatModels = await getFeatureModels(['chat']);

			const savedModel =
				typeof localStorage !== 'undefined' ? localStorage.getItem('monolai:selected_model') : null;
			if (savedModel && registeredChatModels.some((m) => m.id === savedModel)) {
				selectedModel = savedModel;
			} else if (registeredChatModels.length > 0 && !selectedModel) {
				selectedModel = registeredChatModels[0].id;
				if (typeof localStorage !== 'undefined') {
					localStorage.setItem('monolai:selected_model', selectedModel);
				}
			}
		} catch (err) {
			console.error('Error fetching chat models via OpenAI SDK:', err);
		}
	}

	// Select model WITHOUT triggering immediate swap (lazy swap on proxy request)
	function selectModel(modelId: string) {
		selectedModel = modelId;
		isModelMenuOpen = false;
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem('monolai:selected_model', modelId);
		}
	}

	let loadedChatId = $state<string | undefined>(undefined);

	let sessionChatId = $state<string | undefined>(undefined);

	$effect(() => {
		sessionChatId = chatId;
	});

	// Load messages for the current active chat session from backend DB
	async function loadChatMessages(targetChatId?: string) {
		liveUsage = {};
		if (!targetChatId) {
			messages = [];
			loadedChatId = undefined;
			return;
		}

		try {
			const records = await chatsApi.getMessages(targetChatId);
			if (records) {
				messages = records.map((r: any) => {
					let tags: string[] | undefined = undefined;

					if (r.model_tags) {
						try {
							tags = JSON.parse(r.model_tags);
						} catch {
							tags = [r.model_tags];
						}
					}

					let content = r.content || '';
					let reasoning: string | undefined = undefined;
					let reasoningDuration: string | undefined = undefined;

					// Check for stored <think>...</think> tags in content (supporting optional duration attribute)
					const thinkMatch = content.match(
						/^<think(?:\s+duration=["']([^"']+)["'])?>([\s\S]*?)<\/think>([\s\S]*)$/
					);
					if (thinkMatch) {
						if (thinkMatch[1]) {
							reasoningDuration = thinkMatch[1].trim();
						} else if (r.duration) {
							reasoningDuration = r.duration;
						}
						reasoning = thinkMatch[2].trim();
						content = thinkMatch[3].trim();
						collapsedReasoning[r.id] = true;
					}

					return {
						id: r.id,
						role: r.role,
						content,
						reasoning,
						reasoningDuration,
						tokens: r.tokens || undefined,
						duration: r.duration || undefined,
						speed: r.speed || undefined,
						modelTags: tags,
						status: r.status || undefined
					};
				});

				const lastAssistant = [...messages]
					.reverse()
					.find(
						(m) => m.role === 'assistant' && m.speed && m.speed !== '0 t/s' && m.speed !== '0.0t/s'
					);
				if (lastAssistant?.speed) {
					liveUsage = { avgSpeed: lastAssistant.speed };
				}

				loadedChatId = targetChatId;
				scrollToBottom();
			} else {
				messages = [];
				loadedChatId = targetChatId;
			}
		} catch (err) {
			console.error('Error loading chat messages:', err);
			messages = [];
			loadedChatId = targetChatId;
		}
	}

	$effect(() => {
		const targetId = sessionChatId || chatId;
		if (targetId !== loadedChatId && !isGenerating) {
			loadChatMessages(targetId);
		}
	});

	// Sync top header tabs and conversations with backend chats
	async function refreshHeaderTabs(targetId?: string) {
		try {
			await chatsState.ensureLoaded(targetId || sessionChatId);
		} catch (e) {
			console.error('Error syncing header tabs:', e);
		}
	}

	$effect(() => {
		refreshHeaderTabs();
	});

	// Popup menus state and element references for click-outside
	let toolsMenuOpen = $state(false);
	let activeSubMenu = $state<string | null>(null);
	let toolsWrapper: HTMLDivElement | null = $state(null);
	let activeTooltipId = $state<string | null>(null);

	function toggleModelMenu(e?: MouseEvent) {
		if (e) e.stopPropagation();
		isModelMenuOpen = !isModelMenuOpen;
		toolsMenuOpen = false;
		if (isModelMenuOpen) {
			loadChatModels();
		}
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as Node;
		if (toolsMenuOpen && toolsWrapper && !toolsWrapper.contains(target)) {
			toolsMenuOpen = false;
			activeSubMenu = null;
			reasoningMenuOpen = false;
		}
		if (isModelMenuOpen && modelWrapper && !modelWrapper.contains(target)) {
			isModelMenuOpen = false;
		}
		if (activeTooltipId) {
			activeTooltipId = null;
		}
	}

	let wasInterruptedByUnload = $state(false);
	let wasStoppedByUser = $state(false);

	function handleModelUnloadInterruption(unloadedModelId?: string) {
		if (!isGenerating) return;
		if (!unloadedModelId || unloadedModelId === '*' || unloadedModelId === selectedModel) {
			wasInterruptedByUnload = true;
			if (currentAbortController) {
				currentAbortController.abort('INTERRUPTED');
			}
		}
	}

	$effect(() => {
		if (isGenerating && selectedModel) {
			if (runningModelsState.unloadingModelIds.has(selectedModel)) {
				handleModelUnloadInterruption(selectedModel);
			}
		}
	});

	onMount(() => {
		loadChatModels();
		loadPromptHistory();
		loadSystemPrompt();
		document.addEventListener('click', handleClickOutside);

		const onUnloadEvent = (e: Event) => {
			const customEvent = e as CustomEvent<{ modelId?: string }>;
			handleModelUnloadInterruption(customEvent.detail?.modelId);
		};
		if (typeof window !== 'undefined') {
			window.addEventListener('monolai:model-unload', onUnloadEvent);
		}

		const updateTheme = () => {
			const theme = document.documentElement.getAttribute('data-theme');
			if (theme === 'light' || theme === 'dark') {
				effectiveTheme = theme;
			}
		};
		updateTheme();
		const observer = new MutationObserver(updateTheme);
		observer.observe(document.documentElement, {
			attributes: true,
			attributeFilter: ['data-theme']
		});

		return () => {
			document.removeEventListener('click', handleClickOutside);
			if (typeof window !== 'undefined') {
				window.removeEventListener('monolai:model-unload', onUnloadEvent);
			}
			observer.disconnect();
		};
	});

	const builtInTools = [
		{ id: 'read_file', name: 'read_file', checked: true },
		{ id: 'file_glob_search', name: 'file_glob_search', checked: true },
		{ id: 'grep_search', name: 'grep_search', checked: true },
		{ id: 'exec_shell_command', name: 'exec_shell_command', checked: true },
		{ id: 'write_file', name: 'write_file', checked: true },
		{ id: 'edit_file', name: 'edit_file', checked: true },
		{ id: 'get_datetime', name: 'get_datetime', checked: true }
	];

	let showScrollButton = $state(false);

	function updateScrollbarWidth() {
		if (messagesContainer) {
			const diff = messagesContainer.offsetWidth - messagesContainer.clientWidth;
			scrollbarWidth = diff > 0 ? diff : 0;
		}
	}

	$effect(() => {
		// Track message list and generation changes
		messages.length;
		isGenerating;
		if (!messagesContainer) return;
		updateScrollbarWidth();

		const ro = new ResizeObserver(updateScrollbarWidth);
		ro.observe(messagesContainer);
		window.addEventListener('resize', updateScrollbarWidth);
		return () => {
			ro.disconnect();
			window.removeEventListener('resize', updateScrollbarWidth);
		};
	});

	function handleScroll() {
		if (messagesContainer) {
			updateScrollbarWidth();
			const { scrollTop, scrollHeight, clientHeight } = messagesContainer;
			const distanceFromBottom = scrollHeight - scrollTop - clientHeight;
			showScrollButton = distanceFromBottom >= 100;

			// If user scrolled up (> 80px from bottom), pause auto-scrolling
			// If user scrolled back near the bottom (<= 40px), resume auto-scrolling
			if (distanceFromBottom > 80) {
				autoScrollChat = false;
			} else if (distanceFromBottom <= 40) {
				autoScrollChat = true;
			}
		}
	}

	async function scrollToBottom() {
		autoScrollChat = true;
		await tick();
		if (messagesContainer) {
			messagesContainer.scrollTo({
				top: messagesContainer.scrollHeight,
				behavior: 'smooth'
			});
			showScrollButton = false;
		}
	}

	async function handleSendMessage() {
		const text = inputMessage.trim();
		if (!text || isGenerating) return;

		if (!selectedModel) {
			if (registeredChatModels.length === 0) {
				await loadChatModels();
			}
			if (registeredChatModels.length > 0 && !selectedModel) {
				selectedModel = registeredChatModels[0].id;
			}
		}

		if (!selectedModel) {
			await showAlert('Please select a model first.', 'Model Required', 'warning');
			return;
		}

		savePromptToHistory(text);

		let activeId = sessionChatId || chatId;
		isGenerating = true;
		wasInterruptedByUnload = false;
		wasStoppedByUser = false;

		if (!activeId) {
			activeId = generateUUID();
			sessionChatId = activeId;
			loadedChatId = activeId;
		} else {
			loadedChatId = activeId;
		}

		const userMsgId = generateUUID();
		const userMsg: Message = {
			id: userMsgId,
			role: 'user',
			content: text
		};

		const assistantMsgId = generateUUID();
		const assistantMsg: Message = {
			id: assistantMsgId,
			role: 'assistant',
			content: '',
			tokens: 0,
			duration: '0s',
			speed: '0 t/s',
			modelTags: [selectedModel],
			status: 'RUNNING'
		};

		// Optimistically add user & assistant messages to UI and clear input textarea immediately
		messages = [...messages, userMsg, assistantMsg];
		inputMessage = '';
		if (textareaRef) {
			textareaRef.style.height = 'auto';
		}
		tick().then(() => {
			if (textareaRef) {
				textareaRef.style.height = 'auto';
			}
		});
		autoScrollChat = true;
		scrollToBottom();
		textareaRef?.focus();

		// Create chat session in DB if starting a new chat
		if (!chatId && sessionChatId === activeId) {
			try {
				const chatTitle = text.length > 25 ? text.slice(0, 25) + '...' : text;
				await chatsApi.create({ id: activeId, title: chatTitle });
				goto(`/chat/${activeId}`, { replaceState: true, keepFocus: true });
				chatsState.notifyUpdated(activeId);
			} catch (e) {
				console.error('Error creating chat session:', e);
			}
		}

		// Save User Message & Assistant Message placeholder to Backend DB
		try {
			await fetch(`/api/chats/${encodeURIComponent(activeId)}/messages`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					id: userMsgId,
					role: 'user',
					content: text
				})
			});

			if (messages.length <= 2) {
				const newTitle = text.length > 25 ? text.slice(0, 25) + '...' : text;
				await chatsApi.update(activeId, newTitle);
				chatsState.notifyUpdated(activeId);
			}

			await fetch(`/api/chats/${encodeURIComponent(activeId)}/messages`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					id: assistantMsgId,
					role: 'assistant',
					content: '',
					model_tags: JSON.stringify([selectedModel]),
					status: 'RUNNING'
				})
			});
		} catch (err) {
			console.error('Error persisting message start to DB:', err);
		}

		currentAbortController = new AbortController();
		const startTime = performance.now();
		let tokenCount = 0;
		let accumulatedContent = '';
		let accumulatedReasoning = '';
		let isInsideThinkTag = false;
		let reasoningStartTime: number | null = null;
		let reasoningDuration = '';
		let finalDuration = '0s';
		let finalSpeed = '0 t/s';
		let finalStatus = 'COMPLETED';

		// Expand reasoning accordion for this new assistant response
		collapsedReasoning[assistantMsgId] = false;
		let hasAutoCollapsedReasoning = false;
		autoScrollReasoning[assistantMsgId] = true;

		try {
			// Industry standard Sliding Window Context:
			// 1. Anchors System Prompt at index 0
			// 2. Filters out empty / failed / cancelled messages
			// 3. Slides and preserves chronological turn pairs within token budget
			const prepared = prepareSlidingWindowContext(
				messages.slice(0, -1), // exclude currently running assistant placeholder
				systemPrompt,
				contextLimit,
				currentReasoningPreset.maxTokens || 1024
			);

			const requestPayload: any = {
				model: selectedModel,
				messages: prepared.messages,
				stream: true,
				stream_options: { include_usage: true }
			};

			if (currentReasoningPreset.maxTokens) {
				requestPayload.max_tokens = currentReasoningPreset.maxTokens;
			}
			if (selectedReasoning !== 'default' && selectedReasoning !== 'off') {
				requestPayload.reasoning_effort = selectedReasoning;
			}

			// Pre-compute turn metrics for the context inspector
			const turnPrompt = prepared.usedTokens;
			const lastMsgContent = prepared.messages[prepared.messages.length - 1]?.content || '';
			const turnFresh = Math.max(1, Math.round(lastMsgContent.length / 3.8));
			const turnCached = Math.max(0, turnPrompt - turnFresh);

			liveUsage = {
				...liveUsage,
				thisTurnPrompt: turnPrompt,
				thisTurnFresh: turnFresh,
				thisTurnCached: turnCached,
				thisTurnGenerated: 0,
				kvCacheTotal: turnPrompt,
				avgSpeed:
					liveUsage.avgSpeed && liveUsage.avgSpeed !== '0.0t/s' ? liveUsage.avgSpeed : '0.0t/s'
			};

			const stream = await streamChatCompletion({
				...requestPayload,
				signal: currentAbortController.signal
			});

			for await (const chunk of stream) {
				if (
					wasInterruptedByUnload ||
					currentAbortController?.signal.reason === 'INTERRUPTED' ||
					runningModelsState.unloadingModelIds.has(selectedModel)
				) {
					wasInterruptedByUnload = true;
					break;
				}
				if (
					wasStoppedByUser ||
					currentAbortController?.signal.reason === 'USER_CANCELLED' ||
					currentAbortController?.signal.aborted
				) {
					wasStoppedByUser = true;
					break;
				}

				const parsed = chunk as any;

				// Process llama.cpp native timings and OpenAI usage details
				if (parsed.timings) {
					const t = parsed.timings;
					if (t.predicted_n && t.predicted_n > tokenCount) {
						tokenCount = t.predicted_n;
					}
					const speedVal = t.predicted_per_second
						? `${t.predicted_per_second.toFixed(1)} t/s`
						: finalSpeed && finalSpeed !== '0 t/s'
							? finalSpeed
							: undefined;
					liveUsage = {
						...liveUsage,
						thisTurnFresh: t.prompt_n ?? liveUsage.thisTurnFresh,
						thisTurnGenerated: tokenCount,
						avgSpeed: speedVal ?? liveUsage.avgSpeed ?? '0.0t/s'
					};
				}
				if (parsed.usage) {
					const u = parsed.usage;
					if (u.completion_tokens && u.completion_tokens > tokenCount) {
						tokenCount = u.completion_tokens;
					}
					const cached = u.prompt_tokens_details?.cached_tokens ?? 0;
					const prompt = u.prompt_tokens ?? liveUsage.thisTurnPrompt ?? turnPrompt;
					const generated = tokenCount;
					liveUsage = {
						...liveUsage,
						thisTurnPrompt: prompt,
						thisTurnCached: cached > 0 ? cached : (liveUsage.thisTurnCached ?? turnCached),
						thisTurnFresh:
							cached > 0 ? Math.max(1, prompt - cached) : (liveUsage.thisTurnFresh ?? turnFresh),
						thisTurnGenerated: generated,
						kvCacheTotal: u.total_tokens ?? prompt + generated
					};
				}

				const delta = parsed.choices?.[0]?.delta;
				const deltaReasoning = delta?.reasoning_content || delta?.reasoning || delta?.thought || '';
				const deltaContentRaw = delta?.content || '';

				let hasChunkUpdate = false;

				// 1. Handle dedicated reasoning fields (DeepSeek-R1, OpenAI o1/o3, llama.cpp with reasoning flag)
				if (deltaReasoning) {
					if (!reasoningStartTime) reasoningStartTime = performance.now();
					accumulatedReasoning += deltaReasoning;
					reasoningDuration = `${((performance.now() - reasoningStartTime) / 1000).toFixed(1)}s`;
					tokenCount++;
					hasChunkUpdate = true;
				}

				// 2. Handle raw content and inline <think> tags
				if (deltaContentRaw) {
					tokenCount++;
					hasChunkUpdate = true;

					let remaining = deltaContentRaw;
					while (remaining.length > 0) {
						if (!isInsideThinkTag) {
							const thinkIdx = remaining.indexOf('<think>');
							if (thinkIdx !== -1) {
								accumulatedContent += remaining.slice(0, thinkIdx);
								remaining = remaining.slice(thinkIdx + 7);
								isInsideThinkTag = true;
								if (!reasoningStartTime) reasoningStartTime = performance.now();
							} else {
								accumulatedContent += remaining;
								remaining = '';
							}
						} else {
							const closeIdx = remaining.indexOf('</think>');
							if (closeIdx !== -1) {
								accumulatedReasoning += remaining.slice(0, closeIdx);
								remaining = remaining.slice(closeIdx + 8);
								isInsideThinkTag = false;
								if (reasoningStartTime) {
									reasoningDuration = `${((performance.now() - reasoningStartTime) / 1000).toFixed(1)}s`;
								}
							} else {
								accumulatedReasoning += remaining;
								remaining = '';
							}
						}
					}
					if (isInsideThinkTag && reasoningStartTime) {
						reasoningDuration = `${((performance.now() - reasoningStartTime) / 1000).toFixed(1)}s`;
					}
				}

				if (hasChunkUpdate) {
					// Auto-collapse reasoning accordion as soon as reasoning is complete and answer content starts streaming
					if (
						!hasAutoCollapsedReasoning &&
						accumulatedReasoning &&
						!isInsideThinkTag &&
						accumulatedContent.trim().length > 0
					) {
						collapsedReasoning[assistantMsgId] = true;
						hasAutoCollapsedReasoning = true;
					}

					const elapsedSec = (performance.now() - startTime) / 1000;
					finalSpeed = elapsedSec > 0 ? `${(tokenCount / elapsedSec).toFixed(1)} t/s` : '0 t/s';
					finalDuration = `${elapsedSec.toFixed(1)}s`;

					liveUsage = {
						...liveUsage,
						thisTurnGenerated: tokenCount,
						kvCacheTotal: (liveUsage.thisTurnPrompt || turnPrompt) + tokenCount,
						avgSpeed: finalSpeed
					};

					messages = messages.map((msg) => {
						if (msg.id === assistantMsgId) {
							return {
								...msg,
								content: accumulatedContent,
								reasoning: accumulatedReasoning,
								reasoningDuration,
								tokens: tokenCount,
								duration: finalDuration,
								speed: finalSpeed
							};
						}
						return msg;
					});

					// 1. Auto-scroll reasoning container if reasoning updated and user hasn't scrolled up inside it
					if (deltaReasoning || isInsideThinkTag) {
						const shouldAutoScroll =
							(autoScrollReasoning as Record<string, boolean>)[assistantMsgId] !== false;
						if (shouldAutoScroll) {
							await tick();
							const rContainer = reasoningContainers[assistantMsgId];
							if (
								rContainer &&
								(autoScrollReasoning as Record<string, boolean>)[assistantMsgId] !== false
							) {
								rContainer.scrollTop = rContainer.scrollHeight;
							}
						}
					}

					// 2. Auto-scroll chat viewport if user hasn't scrolled up
					if (autoScrollChat && messagesContainer) {
						await tick();
						if (autoScrollChat && messagesContainer) {
							messagesContainer.scrollTop = messagesContainer.scrollHeight;
						}
					}
				}
			}

			// Check if generation was aborted or interrupted by model unload
			if (
				wasInterruptedByUnload ||
				currentAbortController?.signal.reason === 'INTERRUPTED' ||
				runningModelsState.unloadingModelIds.has(selectedModel)
			) {
				finalStatus = 'INTERRUPTED';
				messages = messages.map((msg) => {
					if (msg.id === assistantMsgId) {
						return {
							...msg,
							content: accumulatedContent,
							status: 'INTERRUPTED'
						};
					}
					return msg;
				});
			} else if (
				wasStoppedByUser ||
				currentAbortController?.signal.reason === 'USER_CANCELLED' ||
				currentAbortController?.signal.aborted
			) {
				finalStatus = 'CANCELLED';
				messages = messages.map((msg) => {
					if (msg.id === assistantMsgId) {
						return {
							...msg,
							content: accumulatedContent,
							status: 'CANCELLED'
						};
					}
					return msg;
				});
			}

			// Finalize turn metrics across all turns
			const effectiveTotalElapsed = (performance.now() - startTime) / 1000;
			if (tokenCount > 0 && effectiveTotalElapsed > 0) {
				finalSpeed = `${(tokenCount / effectiveTotalElapsed).toFixed(1)} t/s`;
				finalDuration = `${effectiveTotalElapsed.toFixed(1)}s`;
			}

			const validSpeed =
				finalSpeed && finalSpeed !== '0 t/s'
					? finalSpeed
					: liveUsage.avgSpeed && liveUsage.avgSpeed !== '0.0t/s'
						? liveUsage.avgSpeed
						: '0.0t/s';

			liveUsage = {
				...liveUsage,
				allPromptEvaluated:
					(liveUsage.allPromptEvaluated || 0) + (liveUsage.thisTurnFresh || turnFresh),
				allPromptCached:
					(liveUsage.allPromptCached || 0) + (liveUsage.thisTurnCached || turnCached),
				allTokensGenerated: (liveUsage.allTokensGenerated || 0) + tokenCount,
				avgSpeed: validSpeed
			};
		} catch (err: any) {
			const isUnloadInterrupted =
				wasInterruptedByUnload ||
				currentAbortController?.signal.reason === 'INTERRUPTED' ||
				(selectedModel ? runningModelsState.unloadingModelIds.has(selectedModel) : false);

			const isUserCancelled =
				wasStoppedByUser ||
				currentAbortController?.signal.reason === 'USER_CANCELLED' ||
				currentAbortController?.signal.aborted ||
				err.name === 'AbortError' ||
				err.name === 'APIUserAbortError' ||
				err.message?.toLowerCase().includes('abort');

			if (isUnloadInterrupted) {
				finalStatus = 'INTERRUPTED';
				messages = messages.map((msg) => {
					if (msg.id === assistantMsgId) {
						return {
							...msg,
							content: accumulatedContent,
							status: 'INTERRUPTED'
						};
					}
					return msg;
				});
			} else if (isUserCancelled) {
				finalStatus = 'CANCELLED';
				messages = messages.map((msg) => {
					if (msg.id === assistantMsgId) {
						return {
							...msg,
							content: accumulatedContent,
							status: 'CANCELLED'
						};
					}
					return msg;
				});
			} else {
				console.error('Chat error:', err);
				finalStatus = 'ERROR';
				accumulatedContent = accumulatedContent
					? `${accumulatedContent}\n\n⚠️ **Error:** ${err.message}`
					: `⚠️ **Error generating response:** ${err.message}`;

				messages = messages.map((msg) => {
					if (msg.id === assistantMsgId) {
						return {
							...msg,
							content: accumulatedContent,
							status: 'ERROR'
						};
					}
					return msg;
				});
			}
		} finally {
			// Persist final assistant response to DB BEFORE setting isGenerating = false
			try {
				if (activeId && assistantMsgId) {
					const fullStoredContent = accumulatedReasoning
						? reasoningDuration
							? `<think duration="${reasoningDuration}">${accumulatedReasoning}</think>${accumulatedContent}`
							: `<think>${accumulatedReasoning}</think>${accumulatedContent}`
						: accumulatedContent;

					await fetch(
						`/api/chats/${encodeURIComponent(activeId)}/messages/${encodeURIComponent(assistantMsgId)}`,
						{
							method: 'PUT',
							headers: { 'Content-Type': 'application/json' },
							body: JSON.stringify({
								content: fullStoredContent,
								tokens: tokenCount,
								duration: finalDuration,
								speed:
									finalSpeed && finalSpeed !== '0 t/s'
										? finalSpeed
										: liveUsage.avgSpeed || '0.0t/s',
								model_tags: JSON.stringify([selectedModel]),
								status: finalStatus
							})
						}
					);
				}
			} catch (err) {
				console.error('Error updating assistant response in DB:', err);
			}

			if (accumulatedReasoning) {
				collapsedReasoning[assistantMsgId] = true;
			}

			isGenerating = false;
			currentAbortController = null;
			wasInterruptedByUnload = false;
			wasStoppedByUser = false;
			if (autoScrollChat) {
				scrollToBottom();
			}
			textareaRef?.focus();
		}
	}

	function handleStopGeneration() {
		if (currentAbortController) {
			wasStoppedByUser = true;
			currentAbortController.abort('USER_CANCELLED');
		}
	}

	function copyMessageContent(content: string) {
		if (content) {
			copyToClipboard(content);
		}
	}

	async function deleteMessage(id: string) {
		if (!chatId) return;
		messages = messages.filter((m) => m.id !== id);
		try {
			await fetch(`/api/chats/${encodeURIComponent(chatId)}/messages/${encodeURIComponent(id)}`, {
				method: 'DELETE'
			});
		} catch (e) {
			console.error('Error deleting message:', e);
		}
	}

	function retryAssistantMessage(assistantMsgId: string) {
		if (isGenerating) return;
		const idx = messages.findIndex((m) => m.id === assistantMsgId);
		if (idx === -1) return;

		const userMsg = messages
			.slice(0, idx)
			.reverse()
			.find((m) => m.role === 'user');
		if (!userMsg) return;

		deleteMessage(assistantMsgId);
		inputMessage = userMsg.content;
		deleteMessage(userMsg.id);
		handleSendMessage();
	}

	function isMobileDevice(): boolean {
		if (typeof window === 'undefined') return false;
		const isTouch =
			window.matchMedia('(pointer: coarse)').matches ||
			(typeof navigator !== 'undefined' && navigator.maxTouchPoints > 0);
		const isMobileWidth =
			window.innerWidth <= 768 || window.matchMedia('(max-width: 768px)').matches;
		const isMobileUA =
			typeof navigator !== 'undefined' &&
			/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent);
		return (isTouch && isMobileWidth) || isMobileUA;
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			if (isMobileDevice()) {
				// On mobile virtual keyboard, Enter inserts a newline instead of sending
				setTimeout(adjustTextareaHeight, 0);
				return;
			}
			e.preventDefault();
			handleSendMessage();
			return;
		}

		if (e.key === 'ArrowUp') {
			const isAtStart = textareaRef ? textareaRef.selectionStart === 0 : true;
			if (isAtStart && promptHistory.length > 0) {
				e.preventDefault();
				if (historyIndex === -1) {
					currentDraft = inputMessage;
				}
				if (historyIndex < promptHistory.length - 1) {
					historyIndex++;
					inputMessage = promptHistory[historyIndex];
					adjustTextareaHeight();
				}
			}
		} else if (e.key === 'ArrowDown') {
			if (historyIndex > -1) {
				e.preventDefault();
				historyIndex--;
				if (historyIndex >= 0) {
					inputMessage = promptHistory[historyIndex];
				} else {
					inputMessage = currentDraft;
				}
				adjustTextareaHeight();
			}
		}
	}

	function toggleToolsMenu() {
		toolsMenuOpen = !toolsMenuOpen;
		if (!toolsMenuOpen) activeSubMenu = null;
		isModelMenuOpen = false;
	}
</script>

<svelte:head>
	<title>Monolai - Chat</title>
</svelte:head>

<div class="chat-page" style:--keyboard-input-height="{inputSectionHeight}px">
	<!-- Messages Container with Smooth Scroll & Upward History -->
	<div
		class="page messages-viewport {isKeyboardOpen ? 'keyboard-open' : ''}"
		bind:this={messagesContainer}
		onscroll={handleScroll}
	>
		{#if messages.length === 0}
			<!-- Empty State -->
			<div class="empty-state">
				<div class="brand-icon-wrapper">
					<Sparkles size={64} class="brand-sparkle" />
				</div>
				<h1 class="hero-title">Hello there</h1>
				<p class="hero-subtitle">Select a model and type a message to start chatting</p>
			</div>
		{:else}
			<div class="messages-list">
				{#each messages as msg (msg.id)}
					<div class="message-block {msg.role}">
						{#if msg.role === 'user'}
							<div class="user-row">
								<div class="user-bubble">{msg.content}</div>
							</div>
							<div class="message-actions user-actions">
								<button
									type="button"
									class="action-btn"
									title="Copy"
									onclick={() => copyMessageContent(msg.content)}><Copy size={14} /></button
								>
								<button
									type="button"
									class="action-btn"
									title="Delete"
									onclick={() => deleteMessage(msg.id)}><Trash2 size={14} /></button
								>
							</div>
						{:else}
							<div class="assistant-content">
								<div class="assistant-text">
									<!-- Reasoning Accordion if message has thoughts -->
									{#if msg.reasoning}
										<div
											class="reasoning-container group/reasoning mb-3 rounded-xl overflow-hidden transition-colors duration-150 {!collapsedReasoning[
												msg.id
											]
												? 'bg-[var(--bg-surface)]'
												: 'bg-transparent hover:bg-[var(--bg-surface)]'}"
										>
											<button
												type="button"
												class="flex items-center justify-between w-full px-3.5 py-2 text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors cursor-pointer border-0 bg-transparent select-none"
												onclick={() => toggleReasoningCollapse(msg.id)}
												aria-expanded={!collapsedReasoning[msg.id]}
											>
												<div class="flex items-center gap-2 font-medium">
													{#if isGenerating && !msg.content}
														<Sparkles
															size={13}
															class="text-[var(--primary)] animate-pulse shrink-0"
														/>
														<span class="text-[var(--primary)] font-semibold">Thinking...</span>
													{:else}
														<Sparkles size={13} class="text-[var(--text-muted)] shrink-0" />
														<span
															>{msg.reasoningDuration
																? `Thought for ${msg.reasoningDuration}`
																: 'Thinking process'}</span
														>
													{/if}
												</div>
												<ChevronDown
													size={13}
													class="text-[var(--text-muted)] transition-all duration-150 shrink-0 {!collapsedReasoning[
														msg.id
													]
														? 'opacity-100 rotate-180'
														: 'opacity-0 group-hover/reasoning:opacity-100'}"
												/>
											</button>

											{#if !collapsedReasoning[msg.id]}
												<div
													use:registerReasoningContainer={msg.id}
													onscroll={(e) => handleReasoningScroll(e, msg.id)}
													class="px-3.5 pb-3 pt-1 text-xs text-[var(--text-secondary)] bg-transparent leading-relaxed font-sans whitespace-pre-wrap max-h-36 sm:max-h-40 overflow-y-auto"
												>
													{msg.reasoning}
												</div>
											{/if}
										</div>
									{/if}

									{#if !msg.content && !msg.reasoning && isGenerating}
										<div class="typing-dots">
											<span class="dot"></span>
											<span class="dot"></span>
											<span class="dot"></span>
										</div>
									{:else if !msg.content && (msg.status === 'CANCELLED' || msg.status === 'INTERRUPTED')}
										<span class="cancelled-text-placeholder"
											>({msg.status === 'INTERRUPTED'
												? t('chat.interrupted')
												: t('chat.cancelledByUser')})</span
										>
									{:else if msg.content}
										<MarkdownRenderer content={msg.content} />
									{/if}
								</div>

								<div class="model-meta-row">
									{#if msg.modelTags && msg.modelTags.length > 0}
										<ModelBadge model={msg.modelTags} />
									{/if}

									{#if msg.status === 'CANCELLED' || msg.status === 'INTERRUPTED'}
										<div class="tooltip-container {activeTooltipId === msg.id ? 'active' : ''}">
											<button
												type="button"
												class="cancelled-status-badge"
												onclick={(e) => {
													e.stopPropagation();
													activeTooltipId = activeTooltipId === msg.id ? null : msg.id;
												}}
												aria-label={msg.status === 'INTERRUPTED'
													? t('chat.interrupted')
													: t('chat.cancelledByUser')}
											>
												{#if msg.status === 'INTERRUPTED'}
													<CircleAlert size={12} class="cancelled-icon" />
												{:else}
													<Square size={10} fill="currentColor" class="cancelled-icon" />
												{/if}
											</button>
											<div class="tooltip-bubble" role="tooltip">
												<span
													>{msg.status === 'INTERRUPTED'
														? t('chat.interrupted')
														: t('chat.cancelledByUser')}</span
												>
												<div class="tooltip-arrow"></div>
											</div>
										</div>
									{/if}

									<div class="message-meta">
										{#if msg.tokens}<span class="meta-item"
												><Box size={12} /> {msg.tokens} tokens</span
											>{/if}
										{#if msg.duration}<span class="meta-item"
												><Clock size={12} /> {msg.duration}</span
											>{/if}
										{#if msg.speed}<span class="meta-item"><Zap size={12} /> {msg.speed}</span>{/if}
									</div>
								</div>

								<div class="message-actions">
									<button
										type="button"
										class="action-btn"
										title="Copy"
										onclick={() => copyMessageContent(msg.content)}><Copy size={14} /></button
									>
									<button
										type="button"
										class="action-btn"
										title="Retry"
										onclick={() => retryAssistantMessage(msg.id)}><RotateCw size={14} /></button
									>
									<button
										type="button"
										class="action-btn"
										title="Delete"
										onclick={() => deleteMessage(msg.id)}><Trash2 size={14} /></button
									>
								</div>
							</div>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Floating Glassmorphism Bottom Input Container -->
	<div class="input-section" bind:this={inputSectionRef} style:right="{scrollbarWidth}px">
		{#if showScrollButton}
			<button
				type="button"
				class="scroll-down-btn backdrop-blur-xl bg-[var(--bg-surface)]/65 border border-[var(--border-color)]/60"
				onclick={scrollToBottom}
				title="Scroll to bottom"
			>
				<ChevronDown size={18} />
			</button>
		{/if}
		<div
			class="input-card backdrop-blur-xl bg-[var(--bg-surface)]/65 border border-[var(--border-color)]/60"
		>
			<textarea
				bind:this={textareaRef}
				bind:value={inputMessage}
				oninput={adjustTextareaHeight}
				onkeydown={handleKeyDown}
				onfocus={handleTextareaFocus}
				onblur={handleTextareaBlur}
				placeholder="Type a message..."
				rows="1"
				class="chat-textarea"></textarea>

			<!-- Input Controls Row -->
			<div class="input-controls">
				<div class="controls-left">
					<!-- Add / Tools (+) Popup Menu -->
					<div class="popover-wrapper" bind:this={toolsWrapper}>
						<button
							type="button"
							class="plus-btn {toolsMenuOpen ? 'active' : ''}"
							onclick={toggleToolsMenu}
							aria-label="Add or select tools"
							title="Tools & Attachments"
						>
							<Plus size={18} />
						</button>

						{#if toolsMenuOpen}
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<div
								class="mobile-backdrop sm:hidden"
								onclick={() => {
									toolsMenuOpen = false;
									reasoningMenuOpen = false;
								}}
								role="presentation"
							></div>
							<div class="popover-menu main-tools-menu">
								<!-- Reasoning Selection item with auto-positioning submenu and mobile accordion -->
								<div
									class="popover-item-wrapper"
									role="none"
									onmouseenter={() => {
										if (
											typeof window !== 'undefined' &&
											window.matchMedia('(hover: hover)').matches
										) {
											reasoningMenuOpen = true;
											tick().then(checkSubmenuPlacement);
										}
									}}
									onmouseleave={() => {
										if (
											typeof window !== 'undefined' &&
											window.matchMedia('(hover: hover)').matches
										) {
											reasoningMenuOpen = false;
										}
									}}
								>
									<button
										type="button"
										class="popover-item {reasoningMenuOpen ? 'active' : ''}"
										onclick={() => {
											reasoningMenuOpen = !reasoningMenuOpen;
											if (reasoningMenuOpen) {
												tick().then(checkSubmenuPlacement);
											}
										}}
									>
										<div class="item-left">
											<Lightbulb size={16} />
											<span>Reasoning</span>
										</div>
										<div class="item-right">
											<span class="sub-label">{currentReasoningPreset.label}</span>
											<ChevronRight
												size={14}
												class="chevron-icon {reasoningMenuOpen ? 'open' : ''}"
											/>
										</div>
									</button>

									{#if reasoningMenuOpen}
										<div class="popover-submenu pos-{submenuDirection}" bind:this={submenuRef}>
											{#each REASONING_PRESETS as preset}
												<button
													type="button"
													class="submenu-item {selectedReasoning === preset.id ? 'selected' : ''}"
													onclick={() => {
														selectedReasoning = preset.id;
														reasoningMenuOpen = false;
														toolsMenuOpen = false;
													}}
												>
													<div class="submenu-left">
														<span class="check-slot">
															{#if selectedReasoning === preset.id}
																<Check size={14} />
															{/if}
														</span>
														<span class="preset-name">{preset.label}</span>
													</div>
													{#if preset.desc}
														<span class="preset-desc">{preset.desc}</span>
													{/if}
												</button>
											{/each}
										</div>
									{/if}
								</div>

								<div class="menu-divider"></div>

								<button
									type="button"
									class="popover-item {promptHistory.length === 0 ? 'disabled' : ''}"
									disabled={promptHistory.length === 0}
									onclick={() => {
										if (promptHistory.length > 0) {
											clearPromptHistory();
											toolsMenuOpen = false;
										}
									}}
								>
									<div class="item-left">
										<Trash2 size={16} />
										<span>Clear Prompt History</span>
									</div>
								</button>
							</div>
						{/if}
					</div>
				</div>

				<div class="controls-right">
					<!-- Context Indicator button & popover matching screenshot 1 -->
					<ContextIndicator {contextInfo} />

					<!-- Model Selector Badge (Restored Original Neutral Pill Style - Not Orange!) -->
					<div class="popover-wrapper" bind:this={modelWrapper}>
						<button type="button" class="model-pill-btn" onclick={toggleModelMenu}>
							{#if selectedModel}
								<ModelBadge model={selectedModel} variant="inline" />
							{:else}
								<Box size={14} class="model-box-icon" />
								<span class="model-name-text">Select a Model</span>
							{/if}
						</button>

						{#if isModelMenuOpen}
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<div
								class="mobile-backdrop sm:hidden"
								onclick={() => (isModelMenuOpen = false)}
								role="presentation"
							></div>
							<div class="popover-menu model-menu">
								<div class="menu-label">SELECT ACTIVE MODEL</div>
								{#if registeredChatModels.length === 0}
									<div class="empty-menu-text">No chat models registered</div>
								{:else}
									{#each registeredChatModels as model}
										<button
											type="button"
											class="popover-item {selectedModel === model.id ? 'selected' : ''}"
											onclick={() => selectModel(model.id)}
										>
											<div class="item-left">
												<span class="check-slot">
													{#if selectedModel === model.id}
														<Check size={14} class="check-icon" />
													{/if}
												</span>
												<ModelBadge model={model.id} variant="inline" />
											</div>
											<span class="mini-tag">{model.runtime}</span>
										</button>
									{/each}
								{/if}
							</div>
						{/if}
					</div>

					<!-- Send / Stop Message Button -->
					{#if isGenerating}
						<button
							type="button"
							class="send-btn stop-btn"
							onclick={handleStopGeneration}
							aria-label="Stop Generation"
							title="Stop Generation"
						>
							<Square size={13} fill="currentColor" />
						</button>
					{:else}
						<button
							type="button"
							class="send-btn"
							disabled={!inputMessage.trim()}
							onclick={handleSendMessage}
							aria-label="Send Message"
						>
							<ArrowUp size={18} />
						</button>
					{/if}
				</div>
			</div>
		</div>
	</div>
</div>

<style>
	.chat-page {
		display: flex;
		flex-direction: column;
		height: 100%;
		height: 100dvh;
		max-height: 100%;
		position: relative;
		box-sizing: border-box;
		background-color: var(--bg-primary);
		overflow: hidden;
	}

	.messages-viewport {
		flex: 1;
		overflow-y: auto;
		padding: 4.5rem 1.5rem 9rem 1.5rem;
		display: flex;
		flex-direction: column;
		overscroll-behavior-y: contain;
		-webkit-overflow-scrolling: touch;
	}

	.empty-state {
		margin: auto;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		text-align: center;
		gap: 1rem;
		max-width: 480px;
	}

	.brand-icon-wrapper {
		display: flex;
		align-items: center;
		justify-content: center;
		background: transparent;
		color: var(--primary);
		margin-bottom: 0.25rem;
	}

	.brand-icon-wrapper :global(.brand-sparkle) {
		filter: drop-shadow(0 0 14px color-mix(in srgb, var(--primary) 75%, transparent))
			drop-shadow(0 0 32px color-mix(in srgb, var(--primary) 40%, transparent));
		animation: sparkleGlow 1.5s ease-in-out infinite alternate;
		transition: transform 1.5s cubic-bezier(0.34, 1.56, 0.64, 1);
	}

	.brand-icon-wrapper :global(.brand-sparkle:hover) {
		transform: scale(1.1) rotate(4deg);
	}

	@keyframes sparkleGlow {
		0% {
			filter: drop-shadow(0 0 10px color-mix(in srgb, var(--primary) 60%, transparent))
				drop-shadow(0 0 20px color-mix(in srgb, var(--primary) 30%, transparent));
			transform: scale(0.98);
		}
		100% {
			filter: drop-shadow(0 0 20px color-mix(in srgb, var(--primary) 95%, transparent))
				drop-shadow(0 0 42px color-mix(in srgb, var(--primary) 55%, transparent));
			transform: scale(1.02);
		}
	}

	.hero-title {
		font-size: 2rem;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
	}

	.hero-subtitle {
		font-size: 1rem;
		color: var(--text-muted);
		margin: 0;
	}

	.messages-list {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		max-width: 840px;
		width: 100%;
		margin: 0 auto;
	}

	.chat-top-bar {
		display: flex;
		justify-content: flex-end;
		margin-bottom: 0.5rem;
	}

	.clear-history-btn {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
		background: transparent;
		border: 1px solid var(--border-color);
		color: var(--text-muted);
		padding: 0.3rem 0.6rem;
		border-radius: 0.5rem;
		font-size: 0.75rem;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.clear-history-btn:hover {
		background-color: color-mix(in srgb, #ef4444 10%, transparent);
		border-color: color-mix(in srgb, #ef4444 30%, transparent);
		color: #ef4444;
	}

	.message-block {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.message-block.user {
		align-items: flex-end;
	}

	.user-row {
		display: flex;
		justify-content: flex-end;
		max-width: 80%;
	}

	.user-bubble {
		background-color: var(--bg-surface);
		border: 1px solid var(--border-color);
		color: var(--text-primary);
		padding: 0.875rem 1.125rem;
		border-radius: 1.25rem 1.25rem 0.25rem 1.25rem;
		font-size: 1rem;
		line-height: 1.5;
		white-space: pre-wrap;
		word-break: break-word;
	}

	.assistant-content {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		max-width: 85%;
	}

	.assistant-text {
		font-size: 1.05rem;
		line-height: 1.6;
		color: var(--text-primary);
		word-break: break-word;
	}

	.model-meta-row {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 0.5rem;
		margin-top: 0.25rem;
	}

	.model-name-text {
		font-size: 0.8125rem;
		font-weight: 500;
		color: var(--text-primary);
	}

	.message-meta {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		font-size: 0.75rem;
		font-variant-numeric: tabular-nums;
		color: var(--text-muted);
	}

	.meta-item {
		display: flex;
		align-items: center;
		gap: 0.25rem;
	}

	.message-actions {
		display: flex;
		align-items: center;
		gap: 0.25rem;
		opacity: 0.7;
		transition: opacity 0.15s ease;
	}

	.message-block:hover .message-actions {
		opacity: 1;
	}

	.action-btn {
		background: transparent;
		border: none;
		color: var(--text-muted);
		padding: 0.35rem;
		border-radius: 0.375rem;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.15s ease;
	}

	.action-btn:hover {
		background-color: var(--bg-surface);
		color: var(--text-primary);
	}

	/* Typing dots animation */
	.typing-dots {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
		padding: 0.5rem 0;
	}

	.dot {
		width: 8px;
		height: 8px;
		background-color: var(--text-muted);
		border-radius: 50%;
		animation: pulse-dot 1.4s infinite ease-in-out both;
	}

	.dot:nth-child(1) {
		animation-delay: -0.32s;
	}
	.dot:nth-child(2) {
		animation-delay: -0.16s;
	}

	@keyframes pulse-dot {
		0%,
		80%,
		100% {
			transform: scale(0);
		}
		40% {
			transform: scale(1);
		}
	}

	/* Floating Glassmorphism Bottom Input Section */
	.input-section {
		position: absolute;
		bottom: 1.5rem;
		left: 0;
		right: 0;
		width: auto;
		max-width: calc(840px + 3rem);
		margin: 0 auto;
		padding: 0 1.5rem;
		box-sizing: border-box;
		z-index: 50;
		pointer-events: none;
	}

	.scroll-down-btn {
		position: absolute;
		top: -2.75rem;
		left: 50%;
		transform: translateX(-50%);
		width: 34px;
		height: 34px;
		border-radius: 50%;
		background: color-mix(in srgb, var(--bg-surface) 65%, transparent);
		-webkit-backdrop-filter: blur(16px);
		backdrop-filter: blur(16px);
		border: 1px solid color-mix(in srgb, var(--border-color) 60%, transparent);
		color: var(--text-primary);
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		transition: all 0.15s ease;
		pointer-events: auto;
	}

	.scroll-down-btn:hover {
		background-color: var(--bg-hover);
		transform: translateX(-50%) scale(1.05);
	}

	.input-card {
		background: color-mix(in srgb, var(--bg-surface) 65%, transparent);
		-webkit-backdrop-filter: blur(16px);
		backdrop-filter: blur(16px);
		border: 1px solid color-mix(in srgb, var(--border-color) 60%, transparent);
		border-radius: 1.25rem;
		padding: 0.75rem 1rem;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25);
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		width: 100%;
		max-width: 840px;
		margin: 0 auto;
		box-sizing: border-box;
		pointer-events: auto;
		transform: translateZ(0);
		isolation: isolate;
	}

	.chat-textarea {
		width: 100%;
		background: transparent;
		border: none;
		outline: none;
		color: var(--text-primary);
		font-size: 1rem;
		font-family: inherit;
		resize: none;
		line-height: 1.5;
		padding: 0;
		box-sizing: border-box;
	}

	.input-controls {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.controls-left {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		flex-shrink: 0;
	}

	.controls-right {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		min-width: 0;
		flex-shrink: 1;
	}

	.popover-wrapper {
		position: relative;
		min-width: 0;
		flex-shrink: 1;
	}

	.plus-btn {
		width: 32px;
		height: 32px;
		border-radius: 0.5rem;
		border: 1px solid var(--border-color);
		background-color: transparent;
		color: var(--text-secondary);
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		transition: all 0.15s ease;
		flex-shrink: 0;
	}

	.plus-btn:hover,
	.plus-btn.active {
		background-color: var(--bg-hover);
		color: var(--text-primary);
	}

	/* Restored Original Neutral Model Pill Button Style (Not Orange!) */
	.model-pill-btn {
		display: flex;
		align-items: center;
		gap: 0.35rem;
		padding: 0.35rem 0.65rem;
		font-size: 0.825rem;
		font-weight: 500;
		color: var(--text-secondary);
		background: var(--bg-surface);
		border: 1px solid var(--border-color);
		border-radius: 9999px;
		cursor: pointer;
		transition: all 0.15s ease;
		min-width: 0;
		max-width: 100%;
		overflow: hidden;
		flex-shrink: 1;
	}

	.model-pill-btn:hover {
		color: var(--text-primary);
		border-color: var(--border-hover);
		background: var(--bg-hover);
	}

	.model-box-icon {
		color: var(--text-muted);
	}

	.send-btn {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		background: #3f3f46;
		color: #ffffff;
		border: none;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.send-btn:hover:not(:disabled) {
		background: var(--primary);
	}

	.send-btn:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}

	.send-btn.stop-btn {
		background-color: #ef4444;
	}

	.mobile-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		-webkit-backdrop-filter: blur(2px);
		backdrop-filter: blur(2px);
		z-index: 45;
	}

	/* Popovers */
	.popover-menu {
		position: absolute;
		bottom: calc(100% + 0.5rem);
		left: 0;
		background-color: var(--bg-surface);
		border: 1px solid var(--border-color);
		border-radius: 0.75rem;
		padding: 0.375rem;
		box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
		z-index: 50;
		min-width: 200px;
		display: flex;
		flex-direction: column;
		gap: 0.125rem;
		animation: popoverFadeIn 0.15s ease-out;
	}

	.main-tools-menu {
		min-width: 210px;
	}

	.model-menu {
		right: 0;
		left: auto;
		min-width: 320px;
		width: max-content;
		max-width: min(500px, calc(100vw - 2rem));
		max-height: 65vh;
		overflow-y: auto;
		overflow-x: hidden;
	}

	@media (max-width: 639px) {
		.messages-viewport {
			padding: 4.25rem 1rem 7.5rem 1rem;
		}

		.messages-viewport.keyboard-open {
			padding-bottom: max(8rem, calc(var(--keyboard-input-height, 100px) + 1.25rem)) !important;
			transition: padding-bottom 0.2s ease-out;
		}

		.input-section {
			bottom: calc(env(safe-area-inset-bottom, 0px) + 0.75rem);
			padding: 0 1rem;
			max-width: calc(840px + 2rem);
		}

		.controls-right {
			gap: 0.375rem;
		}

		.model-pill-btn {
			padding: 0.28rem 0.5rem;
			max-width: 140px;
		}

		/* On mobile, keep model name concise and hide secondary tags so + and context indicator are never squeezed */
		.model-pill-btn :global(.model-tag:not(:first-of-type)) {
			display: none;
		}

		.model-pill-btn :global(.model-name-text) {
			max-width: 68px;
		}

		.main-tools-menu {
			position: fixed !important;
			bottom: calc(env(safe-area-inset-bottom, 0px) + 5.25rem) !important;
			left: 1rem !important;
			right: 1rem !important;
			width: auto !important;
			min-width: 0 !important;
			max-width: calc(100vw - 2rem) !important;
			border-radius: 1rem;
			box-shadow: 0 20px 50px rgba(0, 0, 0, 0.7);
			z-index: 50;
		}

		.model-menu {
			position: fixed !important;
			bottom: calc(env(safe-area-inset-bottom, 0px) + 5.25rem) !important;
			left: 1rem !important;
			right: 1rem !important;
			width: auto !important;
			min-width: 0 !important;
			max-width: calc(100vw - 2rem) !important;
			max-height: 55vh;
			overflow-y: auto;
			overflow-x: hidden;
			border-radius: 1rem;
			box-shadow: 0 20px 50px rgba(0, 0, 0, 0.7);
			z-index: 50;
		}
	}

	@media (max-width: 400px) {
		.controls-right {
			gap: 0.25rem;
		}

		.model-pill-btn {
			max-width: 105px;
			padding: 0.25rem 0.4rem;
		}

		.model-pill-btn :global(.model-name-text) {
			max-width: 48px;
		}

		.model-pill-btn :global(.model-tag) {
			font-size: 0.65rem;
			padding: 0.05rem 0.3rem;
		}
	}

	.menu-label {
		padding: 0.5rem 0.75rem 0.25rem 0.75rem;
		font-size: 0.7rem;
		font-weight: 600;
		color: var(--text-muted);
		letter-spacing: 0.05em;
	}

	.empty-menu-text {
		padding: 0.75rem;
		font-size: 0.85rem;
		color: var(--text-muted);
		text-align: center;
	}

	.popover-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1.25rem;
		padding: 0.5rem 0.75rem;
		border-radius: 0.5rem;
		background: transparent;
		border: none;
		color: var(--text-primary);
		font-size: 0.875rem;
		cursor: pointer;
		width: 100%;
		max-width: 100%;
		min-width: 0;
		text-align: left;
		transition: background-color 0.15s ease;
		box-sizing: border-box;
		white-space: nowrap;
		overflow: hidden;
	}

	.popover-item:hover:not(:disabled),
	.popover-item.active {
		background-color: var(--bg-hover);
	}

	.popover-item.disabled,
	.popover-item:disabled {
		opacity: 0.45;
		cursor: not-allowed;
		color: var(--text-muted);
	}

	.popover-item.disabled:hover,
	.popover-item:disabled:hover {
		background-color: transparent;
	}

	.popover-item.selected {
		color: var(--primary);
		font-weight: 500;
	}

	.model-menu .popover-item {
		gap: 0.75rem;
		width: 100%;
		max-width: 100%;
		min-width: 0;
	}

	.model-menu .popover-item.selected {
		background-color: var(--primary-light);
	}

	.model-menu .popover-item.selected :global(.model-name-text) {
		color: var(--primary);
		font-weight: 600;
	}

	.item-left {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		min-width: 0;
		flex: 1 1 auto;
		overflow: hidden;
	}

	.model-menu .item-left :global(.model-badge-container) {
		max-width: 100%;
		min-width: 0;
		overflow: hidden;
	}

	.model-menu .item-left :global(.model-name-text) {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.mini-tag {
		font-size: 0.7rem;
		font-weight: 500;
		color: var(--primary);
		background: var(--primary-light);
		border: 1px solid var(--primary);
		padding: 0.15rem 0.45rem;
		border-radius: 0.375rem;
		white-space: nowrap;
		flex-shrink: 0;
		line-height: 1.2;
	}

	.main-tools-menu {
		min-width: 210px;
	}

	.popover-item-wrapper {
		position: relative;
		width: 100%;
	}

	.item-right {
		display: flex;
		align-items: center;
		gap: 0.35rem;
		color: var(--text-muted);
		font-size: 0.75rem;
	}

	.sub-label {
		color: var(--text-muted);
		font-size: 0.75rem;
	}

	.popover-submenu {
		position: absolute;
		left: calc(100% - 2px);
		bottom: -4px;
		top: auto;
		min-width: 230px;
		background-color: var(--bg-surface);
		border: 1px solid var(--border-color);
		border-radius: 0.75rem;
		padding: 0.35rem;
		box-shadow: 0 16px 40px rgba(0, 0, 0, 0.55);
		display: flex;
		flex-direction: column;
		gap: 2px;
		z-index: 70;
		overflow: visible;
	}

	.popover-submenu.pos-left {
		left: auto;
		right: calc(100% - 2px);
	}

	/* Invisible hit bridge between parent item and flyout submenu */
	.popover-submenu::before {
		content: '';
		position: absolute;
		top: 0;
		bottom: 0;
		width: 16px;
		pointer-events: auto;
	}

	.popover-submenu.pos-right::before {
		left: -14px;
	}

	.popover-submenu.pos-left::before {
		right: -14px;
	}

	.chevron-icon {
		transition: transform 0.15s ease;
	}

	@media (max-width: 640px) {
		.popover-submenu {
			position: static;
			width: 100%;
			min-width: 0;
			box-shadow: none;
			border: 1px solid var(--border-color);
			background-color: var(--bg-hover);
			margin-top: 0.35rem;
			margin-bottom: 0.25rem;
			border-radius: 0.5rem;
			padding: 0.25rem;
		}

		.popover-submenu::before {
			display: none;
		}

		.chevron-icon.open {
			transform: rotate(90deg);
		}
	}

	.submenu-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.75rem;
		padding: 0.45rem 0.65rem;
		border-radius: 0.5rem;
		background: transparent;
		border: none;
		cursor: pointer;
		color: var(--text-primary);
		font-size: 0.8125rem;
		transition: background-color 0.15s ease;
		width: 100%;
		box-sizing: border-box;
		text-align: left;
	}

	.submenu-item:hover {
		background-color: var(--bg-hover);
	}

	.submenu-item.selected {
		color: var(--primary);
		font-weight: 500;
	}

	.submenu-left {
		display: flex;
		align-items: center;
		gap: 0.45rem;
	}

	.check-slot {
		width: 14px;
		height: 14px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--primary);
		flex-shrink: 0;
	}

	.preset-name {
		font-weight: 500;
	}

	.preset-desc {
		font-size: 0.7rem;
		color: var(--text-muted);
		white-space: nowrap;
	}

	.menu-divider {
		height: 1px;
		background: var(--border-color);
		margin: 0.25rem 0.5rem;
		opacity: 0.7;
	}

	.tooltip-container {
		position: relative;
		display: inline-flex;
		align-items: center;
		margin-left: 0.25rem;
	}

	.cancelled-status-badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: 9999px;
		background: rgba(239, 68, 68, 0.12);
		color: #ef4444;
		border: 1px solid rgba(239, 68, 68, 0.3);
		cursor: pointer;
		font-size: 0.7rem;
		font-weight: 500;
		line-height: 1;
		transition: all 0.15s ease;
	}

	.cancelled-status-badge:hover,
	.tooltip-container.active .cancelled-status-badge {
		background: rgba(239, 68, 68, 0.22);
		border-color: rgba(239, 68, 68, 0.5);
		color: #f87171;
	}

	.cancelled-label {
		line-height: 1;
		white-space: nowrap;
	}

	.tooltip-bubble {
		position: absolute;
		bottom: calc(100% + 7px);
		left: 50%;
		transform: translateX(-50%) translateY(4px) scale(0.95);
		background: var(--bg-surface);
		color: var(--text-primary);
		border: 1px solid rgba(239, 68, 68, 0.4);
		padding: 0.35rem 0.65rem;
		border-radius: 0.45rem;
		font-size: 0.725rem;
		font-weight: 500;
		white-space: nowrap;
		box-shadow:
			0 10px 15px -3px rgba(0, 0, 0, 0.4),
			0 4px 6px -2px rgba(0, 0, 0, 0.3);
		opacity: 0;
		visibility: hidden;
		pointer-events: none;
		transition:
			opacity 0.15s ease,
			transform 0.15s ease,
			visibility 0.15s ease;
		z-index: 100;
	}

	.tooltip-arrow {
		position: absolute;
		top: 100%;
		left: 50%;
		transform: translateX(-50%);
		width: 0;
		height: 0;
		border-left: 5px solid transparent;
		border-right: 5px solid transparent;
		border-top: 5px solid rgba(239, 68, 68, 0.4);
	}

	.tooltip-container:hover .tooltip-bubble,
	.tooltip-container.active .tooltip-bubble,
	.tooltip-container:focus-within .tooltip-bubble {
		opacity: 1;
		visibility: visible;
		pointer-events: auto;
		transform: translateX(-50%) translateY(0) scale(1);
	}

	.cancelled-text-placeholder {
		color: var(--text-muted);
		font-size: 0.85rem;
		font-style: italic;
	}

	@media (max-height: 600px) {
		.messages-viewport {
			padding-top: 3.5rem;
			padding-bottom: 5.5rem;
		}

		.empty-state {
			gap: 0.5rem;
		}

		.hero-title {
			font-size: 1.5rem;
		}

		.hero-subtitle {
			font-size: 0.825rem;
		}

		.brand-icon-wrapper :global(.brand-sparkle) {
			width: 44px;
			height: 44px;
		}
	}

	@media (max-height: 480px) and (orientation: landscape) {
		.messages-viewport {
			padding-top: 3.25rem;
			padding-bottom: 4.75rem;
		}

		.empty-state {
			gap: 0.25rem;
		}

		.hero-title {
			font-size: 1.25rem;
		}

		.hero-subtitle {
			display: none;
		}

		.brand-icon-wrapper :global(.brand-sparkle) {
			width: 36px;
			height: 36px;
		}

		.input-section {
			bottom: 0.4rem;
		}
	}
</style>
