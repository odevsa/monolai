export type ThemeId =
	| 'system'
	| 'dark'
	| 'light'
	| 'catppuccin'
	| 'code'
	| 'dracula'
	| 'nord'
	| 'tokyonight'
	| 'monokai'
	| 'cyberpunk'
	| 'emerald'
	| 'sunset';

export interface ThemeInfo {
	id: ThemeId;
	name: string;
	description: string;
	type: 'dark' | 'light';
	preview: {
		bg: string;
		surface: string;
		primary: string;
	};
}

export const THEMES: ThemeInfo[] = [
	{
		id: 'system',
		name: 'System',
		description: 'Follows operating system dark/light mode preference',
		type: 'dark',
		preview: { bg: '#0a0a0c', surface: '#17171a', primary: '#6366f1' }
	},
	{
		id: 'dark',
		name: 'Dark (Classic)',
		description: 'Sleek obsidian theme with indigo accents',
		type: 'dark',
		preview: { bg: '#0a0a0c', surface: '#17171a', primary: '#6366f1' }
	},
	{
		id: 'light',
		name: 'Light (Classic)',
		description: 'Clean bright interface with crisp borders',
		type: 'light',
		preview: { bg: '#f4f4f6', surface: '#ffffff', primary: '#6366f1' }
	},
	{
		id: 'catppuccin',
		name: 'Catppuccin Mocha',
		description: 'Soothing pastel dark theme with mauve & lavender',
		type: 'dark',
		preview: { bg: '#181825', surface: '#1e1e2e', primary: '#cba6f7' }
	},
	{
		id: 'code',
		name: 'Code Dark',
		description: 'Classic Code dark developer palette',
		type: 'dark',
		preview: { bg: '#1e1e1e', surface: '#252526', primary: '#007acc' }
	},
	{
		id: 'dracula',
		name: 'Dracula',
		description: 'Vibrant purple & pink gothic dark theme',
		type: 'dark',
		preview: { bg: '#1e1f29', surface: '#282a36', primary: '#bd93f9' }
	},
	{
		id: 'nord',
		name: 'Nord',
		description: 'Arctic icy blue palette with frosty accents',
		type: 'dark',
		preview: { bg: '#242933', surface: '#2e3440', primary: '#88c0d0' }
	},
	{
		id: 'tokyonight',
		name: 'Tokyo Night',
		description: 'Japanese cyber dusk aesthetic with deep blue & purple',
		type: 'dark',
		preview: { bg: '#1a1b26', surface: '#24283b', primary: '#7aa2f7' }
	},
	{
		id: 'monokai',
		name: 'Monokai Pro',
		description: 'Vivid gold & rose magenta code editor palette',
		type: 'dark',
		preview: { bg: '#2d2a2e', surface: '#3a373b', primary: '#ffd866' }
	},
	{
		id: 'cyberpunk',
		name: 'Cyberpunk',
		description: 'Neon dark theme with yellow and magenta vibes',
		type: 'dark',
		preview: { bg: '#0b0914', surface: '#161224', primary: '#facc15' }
	},
	{
		id: 'emerald',
		name: 'Emerald Forest',
		description: 'Deep woodland dark with glowing mint emerald',
		type: 'dark',
		preview: { bg: '#041711', surface: '#0a2920', primary: '#10b981' }
	},
	{
		id: 'sunset',
		name: 'Midnight Sunset',
		description: 'Deep dusk violet with warm orange amber glow',
		type: 'dark',
		preview: { bg: '#0f0c1b', surface: '#1b1730', primary: '#f97316' }
	}
];

export function resolveEffectiveTheme(themeId: ThemeId, systemPrefersDark: boolean): ThemeId {
	if (themeId === 'system') {
		return systemPrefersDark ? 'dark' : 'light';
	}
	return themeId;
}

export function getThemeType(effectiveTheme: string): 'dark' | 'light' {
	const found = THEMES.find((t) => t.id === effectiveTheme);
	if (found) return found.type;
	return effectiveTheme === 'light' ? 'light' : 'dark';
}
