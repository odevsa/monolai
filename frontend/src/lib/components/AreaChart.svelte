<script lang="ts">
	import { AlertTriangle } from '@lucide/svelte';
	import { MAX_CHART_POINTS } from '$lib';

	let {
		data = [],
		maxVal = 100,
		maxPoints = MAX_CHART_POINTS,
		color = '#38bdf8',
		unit = '%',
		title = '',
		subtitle = '',
		currentValue = 0,
		icon: IconComponent = null,
		unavailable = false,
		unavailableMessage = 'Resource Not Available'
	}: {
		data: number[];
		maxVal?: number;
		maxPoints?: number;
		color?: string;
		unit?: string;
		title?: string;
		subtitle?: string;
		currentValue?: string | number;
		icon?: any;
		unavailable?: boolean;
		unavailableMessage?: string;
	} = $props();

	const id = Math.random().toString(36).substring(2, 9);
	const gradientId = `mountain-gradient-${id}`;

	const width = 500;
	const height = 140;
	const paddingY = 12;

	let points = $derived.by(() => {
		if (unavailable || !data || data.length === 0) return [];
		const slice = data.slice(-maxPoints);
		const offset = maxPoints - slice.length;

		return slice.map((val, idx) => {
			const slotIndex = offset + idx;
			const clamped = Math.min(maxVal, Math.max(0, val));
			const normY = clamped / maxVal;
			const x = (slotIndex / (maxPoints - 1)) * width;
			const y = height - paddingY - normY * (height - 2 * paddingY);
			return { x, y, val };
		});
	});

	let pathD = $derived.by(() => {
		if (points.length === 0) return '';
		return points.reduce((acc, p, i) => `${acc} ${i === 0 ? 'M' : 'L'} ${p.x.toFixed(1)},${p.y.toFixed(1)}`, '');
	});

	let areaD = $derived.by(() => {
		if (points.length === 0) return '';
		const first = points[0];
		const last = points[points.length - 1];
		return `${pathD} L ${last.x.toFixed(1)},${height} L ${first.x.toFixed(1)},${height} Z`;
	});

	let lastPoint = $derived(points.length > 0 ? points[points.length - 1] : null);
</script>

<div class="bg-[var(--bg-surface)] rounded-2xl p-5 flex flex-col justify-between gap-4 h-full border-0">
	<div class="flex items-start justify-between gap-4 min-w-0">
		<div class="flex items-center gap-3 min-w-0 flex-1">
			{#if IconComponent}
				<div
					class="w-9 h-9 rounded-xl flex items-center justify-center shrink-0"
					style="background-color: color-mix(in srgb, {color} 15%, transparent); color: {color}"
				>
					<IconComponent size={18} />
				</div>
			{/if}
			<div class="min-w-0 flex-1">
				<h3 class="m-0 text-sm font-semibold text-[var(--text-primary)] truncate">{title}</h3>
				{#if subtitle}
					<span class="text-xs text-[var(--text-muted)] block mt-0.5 truncate" title={subtitle}>{subtitle}</span>
				{/if}
			</div>
		</div>

		{#if !unavailable}
			<div class="flex items-baseline gap-1.5 shrink-0 ml-auto pt-0.5">
				<span class="text-2xl font-bold tracking-tight leading-none tabular-nums whitespace-nowrap" style="color: {color}">
					{typeof currentValue === 'number' ? currentValue.toFixed(1) : currentValue}
				</span>
				<span class="text-xs font-medium text-[var(--text-muted)] shrink-0">{unit}</span>
			</div>
		{/if}
	</div>

	{#if unavailable}
		<div class="w-full h-[140px] flex flex-col items-center justify-center p-4 text-center bg-[var(--bg-surface-hover)] rounded-xl border-0">
			<div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-500 text-xs font-semibold">
				<AlertTriangle size={15} />
				<span>{unavailableMessage}</span>
			</div>
			<p class="mt-3 m-0 text-xs text-[var(--text-muted)] max-w-[280px] leading-relaxed">
				No hardware activity detected.
			</p>
		</div>
	{:else}
		<div class="w-full h-[140px] relative overflow-hidden rounded-xl bg-[var(--bg-surface-hover)] border-0">
			<svg
				viewBox="0 0 {width} {height}"
				preserveAspectRatio="none"
				class="w-full h-full block"
			>
				<defs>
					<linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
						<stop offset="0%" stop-color={color} stop-opacity="0.45" />
						<stop offset="70%" stop-color={color} stop-opacity="0.1" />
						<stop offset="100%" stop-color={color} stop-opacity="0.0" />
					</linearGradient>
				</defs>

				<!-- Grid Lines -->
				<line x1="0" y1={paddingY} x2={width} y2={paddingY} class="stroke-[var(--border-color)] stroke-dasharray-[4_4] stroke-opacity-40 stroke-1" />
				<line x1="0" y1={height / 2} x2={width} y2={height / 2} class="stroke-[var(--border-color)] stroke-dasharray-[4_4] stroke-opacity-40 stroke-1" />
				<line x1="0" y1={height - paddingY} x2={width} y2={height - paddingY} class="stroke-[var(--border-color)] stroke-dasharray-[4_4] stroke-opacity-40 stroke-1" />

				{#if points.length > 0}
					<!-- Area Fill -->
					<path d={areaD} fill="url(#{gradientId})" />

					<!-- Top Line -->
					<path
						d={pathD}
						fill="none"
						stroke={color}
						stroke-width="2.5"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>

					<!-- Pulse Glow Dot on Latest Point -->
					{#if lastPoint}
						<circle
							cx={lastPoint.x}
							cy={lastPoint.y}
							r="7"
							fill={color}
							opacity="0.3"
							class="animate-ping"
						/>
						<circle
							cx={lastPoint.x}
							cy={lastPoint.y}
							r="4"
							fill={color}
							stroke="var(--bg-surface)"
							stroke-width="1.5"
						/>
					{/if}
				{/if}
			</svg>
		</div>
	{/if}
</div>
