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

class ConfirmStateManager {
	current = $state<ConfirmStateItem | null>(null);

	askConfirm(
		message: string,
		title: string = 'Confirmation',
		variant: 'danger' | 'warning' | 'primary' | 'info' = 'danger',
		confirmText: string = 'Confirm',
		cancelText: string = 'Cancel'
	): Promise<boolean> {
		return new Promise((resolve) => {
			this.current = {
				message,
				title,
				variant,
				confirmText,
				cancelText,
				isAlert: false,
				resolve
			};
		});
	}

	showAlert(
		message: string,
		title: string = 'Notice',
		variant: 'primary' | 'info' | 'danger' | 'warning' = 'info',
		confirmText: string = 'OK'
	): Promise<boolean> {
		return new Promise((resolve) => {
			this.current = {
				message,
				title,
				variant,
				confirmText,
				cancelText: '',
				isAlert: true,
				resolve
			};
		});
	}

	close(result: boolean) {
		if (this.current) {
			const res = this.current.resolve;
			this.current = null;
			res(result);
		}
	}
}

export const confirmState = new ConfirmStateManager();
export const askConfirm = (
	message: string,
	title: string = 'Confirmation',
	variant: 'danger' | 'warning' | 'primary' | 'info' = 'danger',
	confirmText: string = 'Confirm',
	cancelText: string = 'Cancel'
) => confirmState.askConfirm(message, title, variant, confirmText, cancelText);

export const showAlert = (
	message: string,
	title: string = 'Notice',
	variant: 'primary' | 'info' | 'danger' | 'warning' = 'info',
	confirmText: string = 'OK'
) => confirmState.showAlert(message, title, variant, confirmText);
