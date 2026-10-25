<script lang="ts">
	interface Props {
		class?: string;
		size?: number | string;
		color?: string;
		style?: string;
		[key: string]: unknown;
	}

	let {
		class: className = '',
		size,
		color = 'var(--primary)',
		style = '',
		...restProps
	}: Props = $props();

	let width = $derived(size != null ? (typeof size === 'number' ? `${size}px` : size) : '32px');

	let inlineStyle = $derived(`width: ${width}; height: auto; aspect-ratio: 16/7; ${style}`.trim());
</script>

<svg
	viewBox="0 0 32 14"
	fill="none"
	xmlns="http://www.w3.org/2000/svg"
	class="inline-block shrink-0 {className}"
	style={inlineStyle}
	aria-hidden="true"
	role="status"
	{...restProps}
>
	<circle cx="5" cy="9.5" r="3" fill={color} opacity="0.4">
		<animate
			attributeName="cy"
			values="9.5; 4.5; 9.5; 9.5"
			keyTimes="0; 0.3; 0.6; 1"
			keySplines="0.33 0 0.67 1; 0.33 0 0.67 1; 0 0 1 1"
			calcMode="spline"
			dur="1.2s"
			repeatCount="indefinite"
			begin="0s"
		/>
		<animate
			attributeName="opacity"
			values="0.4; 1; 0.4; 0.4"
			keyTimes="0; 0.3; 0.6; 1"
			keySplines="0.33 0 0.67 1; 0.33 0 0.67 1; 0 0 1 1"
			calcMode="spline"
			dur="1.2s"
			repeatCount="indefinite"
			begin="0s"
		/>
	</circle>
	<circle cx="16" cy="9.5" r="3" fill={color} opacity="0.4">
		<animate
			attributeName="cy"
			values="9.5; 4.5; 9.5; 9.5"
			keyTimes="0; 0.3; 0.6; 1"
			keySplines="0.33 0 0.67 1; 0.33 0 0.67 1; 0 0 1 1"
			calcMode="spline"
			dur="1.2s"
			repeatCount="indefinite"
			begin="0.16s"
		/>
		<animate
			attributeName="opacity"
			values="0.4; 1; 0.4; 0.4"
			keyTimes="0; 0.3; 0.6; 1"
			keySplines="0.33 0 0.67 1; 0.33 0 0.67 1; 0 0 1 1"
			calcMode="spline"
			dur="1.2s"
			repeatCount="indefinite"
			begin="0.16s"
		/>
	</circle>
	<circle cx="27" cy="9.5" r="3" fill={color} opacity="0.4">
		<animate
			attributeName="cy"
			values="9.5; 4.5; 9.5; 9.5"
			keyTimes="0; 0.3; 0.6; 1"
			keySplines="0.33 0 0.67 1; 0.33 0 0.67 1; 0 0 1 1"
			calcMode="spline"
			dur="1.2s"
			repeatCount="indefinite"
			begin="0.32s"
		/>
		<animate
			attributeName="opacity"
			values="0.4; 1; 0.4; 0.4"
			keyTimes="0; 0.3; 0.6; 1"
			keySplines="0.33 0 0.67 1; 0.33 0 0.67 1; 0 0 1 1"
			calcMode="spline"
			dur="1.2s"
			repeatCount="indefinite"
			begin="0.32s"
		/>
	</circle>
</svg>
