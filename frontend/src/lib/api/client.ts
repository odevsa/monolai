export class ApiError extends Error {
	constructor(
		message: string,
		public status: number,
		public details?: any
	) {
		super(message);
		this.name = 'ApiError';
	}
}

export interface RequestOptions extends RequestInit {
	params?: Record<string, string | number | boolean | undefined | null>;
}

export async function request<T>(endpoint: string, options: RequestOptions = {}): Promise<T> {
	const { params, headers, ...restOptions } = options;

	let url = endpoint;
	if (params) {
		const searchParams = new URLSearchParams();
		for (const [key, val] of Object.entries(params)) {
			if (val !== undefined && val !== null) {
				searchParams.append(key, String(val));
			}
		}
		const queryString = searchParams.toString();
		if (queryString) {
			url += (url.includes('?') ? '&' : '?') + queryString;
		}
	}

	const defaultHeaders: Record<string, string> = {
		Accept: 'application/json'
	};

	if (restOptions.body && !(restOptions.body instanceof FormData)) {
		defaultHeaders['Content-Type'] = 'application/json';
	}

	const response = await fetch(url, {
		...restOptions,
		headers: {
			...defaultHeaders,
			...(headers as Record<string, string>)
		}
	});

	if (!response.ok) {
		let errorMsg = `HTTP Error ${response.status}: ${response.statusText}`;
		let details: any = null;
		try {
			const body = await response.json();
			if (body?.error?.message) {
				errorMsg = body.error.message;
			} else if (body?.message) {
				errorMsg = body.message;
			} else if (typeof body === 'string') {
				errorMsg = body;
			}
			details = body;
		} catch {
			const text = await response.text().catch(() => '');
			if (text) errorMsg = text;
		}
		throw new ApiError(errorMsg, response.status, details);
	}

	if (response.status === 204) {
		return null as unknown as T;
	}

	const contentType = response.headers.get('content-type');
	if (contentType && contentType.includes('application/json')) {
		return response.json() as Promise<T>;
	}

	return response.text() as unknown as Promise<T>;
}

export const api = {
	get: <T>(url: string, params?: RequestOptions['params']) =>
		request<T>(url, { method: 'GET', params }),
	post: <T>(url: string, body?: any, params?: RequestOptions['params']) =>
		request<T>(url, {
			method: 'POST',
			body: body ? JSON.stringify(body) : undefined,
			params
		}),
	put: <T>(url: string, body?: any, params?: RequestOptions['params']) =>
		request<T>(url, {
			method: 'PUT',
			body: body ? JSON.stringify(body) : undefined,
			params
		}),
	delete: <T>(url: string, params?: RequestOptions['params']) =>
		request<T>(url, { method: 'DELETE', params })
};
