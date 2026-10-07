/**
 * Clipboard helper utilities
 */

/**
 * Copies text to user's clipboard with fallback support for older browsers
 * or environments without navigator.clipboard permissions.
 */
export async function copyToClipboard(text: string): Promise<boolean> {
	if (!text) return false;

	if (
		typeof navigator !== 'undefined' &&
		navigator.clipboard &&
		typeof navigator.clipboard.writeText === 'function'
	) {
		try {
			await navigator.clipboard.writeText(text);
			return true;
		} catch (err) {
			console.warn('navigator.clipboard failed, attempting fallback:', err);
		}
	}

	if (typeof document !== 'undefined') {
		try {
			const textArea = document.createElement('textarea');
			textArea.value = text;
			textArea.style.position = 'fixed';
			textArea.style.left = '-999999px';
			textArea.style.top = '-999999px';
			document.body.appendChild(textArea);
			textArea.focus();
			textArea.select();
			const success = document.execCommand('copy');
			document.body.removeChild(textArea);
			return success;
		} catch (fallbackErr) {
			console.error('Fallback clipboard copying failed:', fallbackErr);
		}
	}

	return false;
}
