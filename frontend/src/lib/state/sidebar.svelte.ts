const STORAGE_KEY = 'monolai:sidebar_collapsed';

class SidebarStateManager {
	#collapsed = $state(false);
	mobileOpen = $state(false);

	constructor() {
		if (typeof window !== 'undefined') {
			this.loadFromStorage();
		}
	}

	loadFromStorage() {
		if (typeof window === 'undefined') return;
		try {
			const saved = localStorage.getItem(STORAGE_KEY);
			if (saved !== null) {
				this.#collapsed = saved === 'true';
			}
		} catch (err) {
			console.error('Failed to load sidebar state from storage:', err);
		}
	}

	get collapsed() {
		return this.#collapsed;
	}

	set collapsed(value: boolean) {
		this.#collapsed = value;
		if (typeof window !== 'undefined') {
			try {
				localStorage.setItem(STORAGE_KEY, String(value));
			} catch (err) {
				console.error('Failed to save sidebar state to storage:', err);
			}
		}
	}

	toggleCollapse() {
		this.collapsed = !this.collapsed;
	}

	setCollapsed(val: boolean) {
		this.collapsed = val;
	}

	openMobile() {
		this.mobileOpen = true;
	}

	closeMobile() {
		this.mobileOpen = false;
	}
}

export const sidebarState = new SidebarStateManager();
