import {
	confirmState,
	askConfirm,
	showAlert,
	type ConfirmOptions
} from '$lib/state/confirm.svelte';

export { askConfirm, showAlert, type ConfirmOptions };

const subscribers = new Set<(val: any) => void>();

export const confirmStore = {
	subscribe(fn: (val: any) => void) {
		subscribers.add(fn);
		fn(confirmState.current);
		return () => {
			subscribers.delete(fn);
		};
	},
	set(val: any) {
		if (!val && confirmState.current) {
			confirmState.close(false);
		} else {
			confirmState.current = val;
		}
		for (const fn of subscribers) {
			fn(val);
		}
	}
};
