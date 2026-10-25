<script lang="ts">
	interface Props {
		class?: string;
		size?: number | string;
		style?: string;
		[key: string]: unknown;
	}

	let { class: className = '', size, style = '', ...restProps }: Props = $props();

	let inlineStyle = $derived(
		size != null
			? `width: ${typeof size === 'number' ? `${size}px` : size}; ${style}`
			: style || undefined
	);
</script>

<div class="wrapper {className}" style={inlineStyle} {...restProps}>
	<div class="circle"></div>
	<div class="circle"></div>
	<div class="circle"></div>
	<div class="shadow"></div>
	<div class="shadow"></div>
	<div class="shadow"></div>
</div>

<style>
	.wrapper {
		width: 30px;
		aspect-ratio: 9/4;
		position: relative;
		z-index: 1;
		container-type: inline-size;
	}

	.circle {
		width: 17.5cqw;
		height: 17.5cqw;
		position: absolute;
		border-radius: 50%;
		background-color: var(--primary);
		transform-origin: center;
		animation: circle7124 0.5s alternate infinite ease;
		opacity: 0;
	}

	@keyframes circle7124 {
		0% {
			top: 25cqw;
			height: 17.5cqw;
			border-radius: 50% 50% 25% 25%;
			opacity: 1;
		}

		40% {
			height: 20cqw;
			border-radius: 50%;
			opacity: 1;
		}

		100% {
			top: 0%;
			opacity: 1;
		}
	}

	.circle:nth-child(2) {
		left: 50%;
		transform: translateX(-50%);
		animation-delay: 0.2s;
	}

	.circle:nth-child(3) {
		left: auto;
		right: 0;
		animation-delay: 0.3s;
	}

	.shadow {
		width: 12.5cqw;
		height: 2.5cqw;
		border-radius: 50%;
		background-color: rgba(0, 0, 0, 0.9);
		position: absolute;
		top: 37.5cqw;
		z-index: -1;
		left: 2.5cqw;
		filter: blur(2.5cqw);
		animation: shadow046 0.5s alternate infinite ease;
		opacity: 0;
	}

	@keyframes shadow046 {
		40% {
			opacity: 0.7;
		}

		100% {
			opacity: 0.4;
		}
	}

	.shadow:nth-child(4) {
		left: 50%;
		transform: translateX(-50%);
		animation-delay: 0.2s;
	}

	.shadow:nth-child(5) {
		left: auto;
		right: 2.5cqw;
		animation-delay: 0.3s;
	}
</style>
