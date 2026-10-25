export type SettingsTab = 'general' | 'repositories' | 'chat' | 'about';

class SettingsDialogState {
	isOpen = $state(false);
	activeTab = $state<SettingsTab>('general');

	open(tab: SettingsTab = 'general') {
		this.activeTab = tab;
		this.isOpen = true;
	}

	close() {
		this.isOpen = false;
	}
}

export const settingsDialogState = new SettingsDialogState();
