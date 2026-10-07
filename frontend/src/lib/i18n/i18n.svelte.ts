import en from './locales/en.json';

type TranslationSchema = typeof en;

type NestedKeyOf<ObjectType extends object> = {
	[Key in keyof ObjectType & (string | number)]: ObjectType[Key] extends object
		? `${Key}` | `${Key}.${NestedKeyOf<ObjectType[Key]>}`
		: `${Key}`;
}[keyof ObjectType & (string | number)];

export type TranslationKey = NestedKeyOf<TranslationSchema> | (string & {});

class I18nManager {
	currentLocale = $state<string>('en');
	private catalogs: Record<string, Record<string, any>> = {
		en
	};

	setLocale(locale: string) {
		if (this.catalogs[locale]) {
			this.currentLocale = locale;
		}
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
