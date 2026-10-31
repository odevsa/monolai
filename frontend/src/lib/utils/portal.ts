/**
 * Svelte action to portal an element to document.body or a target container.
 * Useful for dropdowns, tooltips, and popovers to escape overflow:hidden and scroll clipping.
 */
export function portal(node: HTMLElement, target: HTMLElement | string = 'body') {
	let targetEl: HTMLElement | null =
		typeof target === 'string' ? document.querySelector(target) : target;

	if (targetEl) {
		targetEl.appendChild(node);
	}

	return {
		update(newTarget: HTMLElement | string) {
			let newTargetEl: HTMLElement | null =
				typeof newTarget === 'string' ? document.querySelector(newTarget) : newTarget;
			if (newTargetEl) {
				newTargetEl.appendChild(node);
			}
		},
		destroy() {
			if (node.parentNode) {
				node.parentNode.removeChild(node);
			}
		}
	};
}
