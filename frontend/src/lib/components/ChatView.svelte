<script lang="ts">
	import { goto } from '$app/navigation';
	import { chatsApi, getFeatureModels, settingsApi } from '$lib/api';
	import { streamChatCompletion } from '$lib/api/openai';
	import { ChatEmptyState, ChatInputBar, ChatMessageItem } from '$lib/components/chat';
	import type { Message } from '$lib/components/chat/ChatMessageItem.svelte';
	import { showAlert } from '$lib/confirmStore';
	import { t } from '$lib/i18n';
	import { runningModelsState } from '$lib/runningModelsStore';
	import { chatsState } from '$lib/state';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { generateUUID } from '$lib/utils/common';
	import {
		DEFAULT_SYSTEM_PROMPT,
		getModelContextLimit,
		prepareSlidingWindowContext,
		REASONING_PRESETS,
		type ReasoningPreset,
		type TokenUsageDetails
	} from '$lib/utils/context';
	import { onMount, tick } from 'svelte';

	let { chatId = '' }: { chatId?: string } = $props();

	interface RegisteredModel {
		id: string;
		runtime: string;
		flags: string;
	}

	let messages = $state<Message[]>([]);
	let inputMessage = $state('');
	let isGenerating = $state(false);
	let currentAbortController: AbortController | null = null;

	let registeredChatModels = $state<RegisteredModel[]>([]);
	let selectedModel = $state<string>('');

	let messagesContainer: HTMLDivElement | null = $state(null);
	let chatInputBarRef = $state<ReturnType<typeof ChatInputBar> | null>(null);
	let inputSectionHeight = $state<number>(110);
	let scrollbarWidth = $state(0);

	// Mobile virtual keyboard detection state
	let isKeyboardOpen = $state(false);
	let isTextareaFocused = $state(false);
	let baseViewportHeight = 0;

	// Prompt History State
	let promptHistory = $state<string[]>([]);

	// Reasoning Effort Selection (Default: Medium)
	let selectedReasoning = $state<ReasoningPreset['id']>('medium');

	// Dynamic auto-scrolling state for chat
	let autoScrollChat = $state(true);
	let showScrollButton = $state(false);

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

	function updateKeyboardStatus() {
		if (typeof window === 'undefined') return;

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
	}

	function clearPromptHistory() {
		promptHistory = [];
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

	function selectModel(modelId: string) {
		selectedModel = modelId;
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
				await tick();
				scrollToBottom(false);
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

		const onUnloadEvent = (e: Event) => {
			const customEvent = e as CustomEvent<{ modelId?: string }>;
			handleModelUnloadInterruption(customEvent.detail?.modelId);
		};
		if (typeof window !== 'undefined') {
			window.addEventListener('monolai:model-unload', onUnloadEvent);
		}

		return () => {
			if (userChatInteractionTimer) {
				clearTimeout(userChatInteractionTimer);
			}
			if (typeof window !== 'undefined') {
				window.removeEventListener('monolai:model-unload', onUnloadEvent);
			}
		};
	});

	function updateScrollbarWidth() {
		if (messagesContainer) {
			const diff = messagesContainer.offsetWidth - messagesContainer.clientWidth;
			scrollbarWidth = diff > 0 ? diff : 0;
		}
	}

	$effect(() => {
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

	let userInteractingWithChat = false;
	let userChatInteractionTimer: ReturnType<typeof setTimeout> | null = null;

	function markUserChatInteraction() {
		userInteractingWithChat = true;
		if (userChatInteractionTimer) clearTimeout(userChatInteractionTimer);
		userChatInteractionTimer = setTimeout(() => {
			userInteractingWithChat = false;
		}, 300);
	}

	function handleScroll() {
		if (messagesContainer) {
			updateScrollbarWidth();
			const { scrollTop, scrollHeight, clientHeight } = messagesContainer;
			const distanceFromBottom = scrollHeight - scrollTop - clientHeight;
			showScrollButton = distanceFromBottom >= 100;

			if (distanceFromBottom <= 40) {
				autoScrollChat = true;
			} else if (userInteractingWithChat && distanceFromBottom > 80) {
				autoScrollChat = false;
			}
		}
	}

	async function scrollToBottom(smooth = true) {
		autoScrollChat = true;
		await tick();
		if (messagesContainer) {
			if (smooth) {
				messagesContainer.scrollTo({
					top: messagesContainer.scrollHeight,
					behavior: 'smooth'
				});
			} else {
				messagesContainer.scrollTop = messagesContainer.scrollHeight;
				requestAnimationFrame(() => {
					if (messagesContainer) {
						messagesContainer.scrollTop = messagesContainer.scrollHeight;
					}
				});
			}
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
			await showAlert(t('chat.selectModelPrompt'), t('models.title'), 'warning');
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

		messages = [...messages, userMsg, assistantMsg];
		inputMessage = '';
		chatInputBarRef?.resetHeight();

		autoScrollChat = true;
		scrollToBottom();
		chatInputBarRef?.focus();

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

		try {
			await chatsApi.createMessage(activeId, {
				id: userMsgId,
				role: 'user',
				content: text
			});

			if (messages.length <= 2) {
				const newTitle = text.length > 25 ? text.slice(0, 25) + '...' : text;
				await chatsApi.update(activeId, newTitle);
				chatsState.notifyUpdated(activeId);
			}

			await chatsApi.createMessage(activeId, {
				id: assistantMsgId,
				role: 'assistant',
				content: '',
				model_tags: JSON.stringify([selectedModel]),
				status: 'RUNNING'
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

		try {
			const prepared = prepareSlidingWindowContext(
				messages.slice(0, -1),
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

				if (deltaReasoning) {
					if (!reasoningStartTime) reasoningStartTime = performance.now();
					accumulatedReasoning += deltaReasoning;
					reasoningDuration = `${((performance.now() - reasoningStartTime) / 1000).toFixed(1)}s`;
					tokenCount++;
					hasChunkUpdate = true;
				}

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

					if (autoScrollChat && messagesContainer) {
						await tick();
						if (autoScrollChat && messagesContainer) {
							messagesContainer.scrollTop = messagesContainer.scrollHeight;
						}
					}
				}
			}

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
				const errText = t('chat.errorGeneratingResponse', { error: err.message || '' });
				accumulatedContent = accumulatedContent
					? `${accumulatedContent}\n\n⚠️ ${errText}`
					: `⚠️ ${errText}`;

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
			try {
				if (activeId && assistantMsgId) {
					const fullStoredContent = accumulatedReasoning
						? reasoningDuration
							? `<think duration="${reasoningDuration}">${accumulatedReasoning}</think>${accumulatedContent}`
							: `<think>${accumulatedReasoning}</think>${accumulatedContent}`
						: accumulatedContent;

					await chatsApi.updateMessage(activeId, assistantMsgId, {
						content: fullStoredContent,
						tokens: tokenCount,
						duration: finalDuration,
						speed:
							finalSpeed && finalSpeed !== '0 t/s' ? finalSpeed : liveUsage.avgSpeed || '0.0t/s',
						model_tags: JSON.stringify([selectedModel]),
						status: finalStatus
					});
				}
			} catch (err) {
				console.error('Error updating assistant response in DB:', err);
			}

			isGenerating = false;
			currentAbortController = null;
			wasInterruptedByUnload = false;
			wasStoppedByUser = false;
			if (autoScrollChat) {
				scrollToBottom();
			}
			chatInputBarRef?.focus();
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
			await chatsApi.deleteMessage(chatId, id);
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
</script>

<svelte:head>
	<title>{t('app.title')} - {t('navigation.chat')}</title>
</svelte:head>

<div
	class="flex flex-col h-full h-[100dvh] max-h-full relative box-border bg-[var(--bg-primary)] overflow-hidden"
	style:--keyboard-input-height="{inputSectionHeight}px"
>
	<!-- Messages Container with Smooth Scroll & Upward History -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="page flex-1 overflow-y-auto flex flex-col overscroll-y-contain pt-[4.5rem] px-6 pb-36 max-sm:pt-[4.25rem] max-sm:px-4 max-sm:pb-[7.5rem] {isKeyboardOpen
			? '!pb-[max(8rem,calc(var(--keyboard-input-height,100px)+1.25rem))] transition-[padding-bottom] duration-200 ease-out'
			: ''}"
		bind:this={messagesContainer}
		onscroll={handleScroll}
		onwheel={markUserChatInteraction}
		ontouchmove={markUserChatInteraction}
		onpointermove={(e) => {
			if (e.buttons > 0) markUserChatInteraction();
		}}
		onkeydown={(e) => {
			if (['ArrowUp', 'ArrowDown', 'PageUp', 'PageDown', 'Home', 'End', ' '].includes(e.key)) {
				markUserChatInteraction();
			}
		}}
	>
		{#if messages.length === 0}
			<ChatEmptyState />
		{:else}
			<div class="flex flex-col gap-6 max-w-[840px] w-full mx-auto">
				{#each messages as msg (msg.id)}
					<ChatMessageItem
						message={msg}
						{isGenerating}
						onCopy={copyMessageContent}
						onDelete={deleteMessage}
						onRetry={retryAssistantMessage}
					/>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Floating Glassmorphic Bottom Input Container -->
	<ChatInputBar
		bind:this={chatInputBarRef}
		bind:inputMessage
		bind:selectedModel
		bind:selectedReasoning
		bind:inputSectionHeight
		{isGenerating}
		{registeredChatModels}
		{promptHistory}
		{contextInfo}
		{showScrollButton}
		{scrollbarWidth}
		onSendMessage={handleSendMessage}
		onStopGeneration={handleStopGeneration}
		onScrollToBottom={scrollToBottom}
		onClearPromptHistory={clearPromptHistory}
		onSelectModel={selectModel}
		onSelectReasoning={(presetId) => (selectedReasoning = presetId)}
		onFocus={handleTextareaFocus}
		onBlur={handleTextareaBlur}
	/>
</div>
