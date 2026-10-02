import { writable } from 'svelte/store';

export interface ConfirmOptions {
	title?: string;
	message: string;
	confirmText?: string;
	cancelText?: string;
	variant?: 'danger' | 'warning' | 'primary' | 'info';
	isAlert?: boolean;
}

export interface ConfirmStateItem extends ConfirmOptions {
	resolve: (result: boolean) => void;
}

export const confirmStore = writable<ConfirmStateItem | null>(null);

export function askConfirm(
	message: string,
	title: string = 'Confirmation',
	variant: 'danger' | 'warning' | 'primary' | 'info' = 'danger',
	confirmText: string = 'Confirm',
	cancelText: string = 'Cancel'
): Promise<boolean> {
	return new Promise((resolve) => {
		confirmStore.set({
			message,
			title,
			variant,
			confirmText,
			cancelText,
			isAlert: false,
			resolve
		});
	});
}

export function showAlert(
	message: string,
	title: string = 'Notice',
	variant: 'primary' | 'info' | 'danger' | 'warning' = 'info',
	confirmText: string = 'OK'
): Promise<boolean> {
	return new Promise((resolve) => {
		confirmStore.set({
			message,
			title,
			variant,
			confirmText,
			cancelText: '',
			isAlert: true,
			resolve
		});
	});
}
