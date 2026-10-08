import en from './locales/en.json';
import es from './locales/es.json';

type TranslationSchema = typeof en;

type NestedKeyOf<ObjectType extends object> = {
	[Key in keyof ObjectType & (string | number)]: ObjectType[Key] extends object
		? `${Key}` | `${Key}.${NestedKeyOf<ObjectType[Key]>}`
		: `${Key}`;
}[keyof ObjectType & (string | number)];

export type TranslationKey = NestedKeyOf<TranslationSchema> | (string & {});

export interface LocaleOption {
	id: string;
	label: string;
}

export const DEFAULT_LOCALE = 'en';

export const AVAILABLE_LOCALES: LocaleOption[] = [
	{ id: 'en', label: 'English' },
	{ id: 'es', label: 'Español' }
];

export function isLocaleSupported(locale: string): boolean {
	return AVAILABLE_LOCALES.some((loc) => loc.id === locale);
}

function detectInitialLocale(): string {
	if (typeof window !== 'undefined') {
		const saved = localStorage.getItem('monolai_language');
		if (saved && isLocaleSupported(saved)) {
			return saved;
		}

		// Dynamically match against configured locales
		const languages = navigator.languages?.length
			? navigator.languages
			: [navigator.language || ''];

		for (const rawLang of languages) {
			if (!rawLang) continue;
			const clean = rawLang.toLowerCase().trim();
			const baseCode = clean.split('-')[0].split('_')[0];

			// 1. Exact match (e.g. 'es' or 'pt-br')
			const exactMatch = AVAILABLE_LOCALES.find((loc) => loc.id.toLowerCase() === clean);
			if (exactMatch) {
				return exactMatch.id;
			}

			// 2. Base language prefix match (e.g. 'es-419' -> 'es')
			const baseMatch = AVAILABLE_LOCALES.find((loc) => loc.id.toLowerCase() === baseCode);
			if (baseMatch) {
				return baseMatch.id;
			}
		}
	}
	return DEFAULT_LOCALE;
}

class I18nManager {
	currentLocale = $state<string>(detectInitialLocale());
	private catalogs: Record<string, Record<string, any>> = {
		en,
		es
	};

	setLocale(locale: string) {
		if (this.catalogs[locale]) {
			this.currentLocale = locale;
			if (typeof window !== 'undefined') {
				localStorage.setItem('monolai_language', locale);
			}
		}
	}

	getLocale(): string {
		return this.currentLocale;
	}

	t(key: TranslationKey, params?: Record<string, string | number>): string {
		const catalog = this.catalogs[this.currentLocale] || this.catalogs.en;
		const segments = key.split('.');
		let current: any = catalog;

		for (const segment of segments) {
			if (current && typeof current === 'object' && segment in current) {
				current = current[segment];
			} else {
				// Fallback: look in 'en' if not current locale
				let fallback: any = this.catalogs.en;
				for (const s of segments) {
					if (fallback && typeof fallback === 'object' && s in fallback) {
						fallback = fallback[s];
					} else {
						return key;
					}
				}
				current = fallback;
				break;
			}
		}

		if (typeof current !== 'string') {
			return key;
		}

		let result = current;
		if (params) {
			for (const [paramKey, paramVal] of Object.entries(params)) {
				result = result.replaceAll(`{{${paramKey}}}`, String(paramVal));
				result = result.replaceAll(`{${paramKey}}`, String(paramVal));
			}
		}

		return result;
	}
}

export const i18n = new I18nManager();
export const t = (key: TranslationKey, params?: Record<string, string | number>) =>
	i18n.t(key, params);
