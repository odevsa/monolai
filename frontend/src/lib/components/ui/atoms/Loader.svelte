<div class="wrapper">
	<div class="circle"></div>
	<div class="circle"></div>
	<div class="circle"></div>
	<div class="shadow"></div>
	<div class="shadow"></div>
	<div class="shadow"></div>
</div>

<style>
	.wrapper {
		width: 40px;
		height: 26px;
		position: relative;
		z-index: 1;
	}

	.circle {
		width: 7px;
		height: 7px;
		position: absolute;
		border-radius: 50%;
		background-color: var(--primary);
		transform-origin: 50%;
		animation: circle7124 0.5s alternate infinite ease;
		opacity: 0;
	}

	@keyframes circle7124 {
		0% {
			top: 10px;
			height: 7px;
			border-radius: 50px 50px 25px 25px;
			opacity: 1;
		}

		40% {
			height: 8px;
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
		transform-origin: 50;
		animation-delay: 0.2s;
	}

	.circle:nth-child(3) {
		left: auto;
		right: 0;
		animation-delay: 0.3s;
	}

	.shadow {
		width: 5px;
		height: 1px;
		border-radius: 50%;
		background-color: rgba(0, 0, 0, 0.9);
		position: absolute;
		top: 15px;
		z-index: -1;
		left: 0;
		filter: blur(1px);
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
		right: 0%;
		animation-delay: 0.3s;
	}
</style>
