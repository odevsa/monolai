/**
 * Formatting utilities for bytes, duration, time, and metrics
 */

/**
 * Formats a byte number into human-readable string (e.g. 1.25 GB, 500 MB).
 */
export function formatBytes(bytes: number): string {
	if (!bytes || bytes <= 0) return '0 B';
	const k = 1024;
	const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
	const i = Math.floor(Math.log(bytes) / Math.log(k));
	const idx = Math.min(i, sizes.length - 1);
	return (bytes / Math.pow(k, idx)).toFixed(2) + ' ' + sizes[idx];
}

/**
 * Formats seconds into human-readable uptime string (e.g. "2d 4h 12m 30s").
 */
export function formatUptime(seconds: number): string {
	if (!seconds || seconds <= 0) return '0s';
	const days = Math.floor(seconds / 86400);
	const hours = Math.floor((seconds % 86400) / 3600);
	const mins = Math.floor((seconds % 3600) / 60);
	const secs = Math.floor(seconds % 60);
	const res: string[] = [];
	if (days > 0) res.push(`${days}d`);
	if (hours > 0) res.push(`${hours}h`);
	if (mins > 0) res.push(`${mins}m`);
	res.push(`${secs}s`);
	return res.join(' ');
}

/**
 * Formats seconds into a human-readable timeout/interval label.
 * (e.g. "5 minutes (300s)", "Never (disabled)")
 */
export function formatTimeoutDuration(seconds: number): string {
	if (seconds <= 0) return 'Never (disabled)';
	if (seconds < 60) return `${seconds} seconds`;
	const mins = Math.floor(seconds / 60);
	const secs = seconds % 60;
	if (secs === 0) {
		return `${mins} ${mins === 1 ? 'minute' : 'minutes'} (${seconds}s)`;
	}
	return `${mins}m ${secs}s (${seconds}s)`;
}
