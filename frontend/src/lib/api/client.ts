const API_BASE_URL = 'http://127.0.0.1:8080/api/v1';

type JsonBody = Record<string, unknown>;

async function request<T>(
	path: string,
	method: string,
	body?: JsonBody,
	token?: string
): Promise<T> {
	const res = await fetch(`${API_BASE_URL}${path}`, {
		method,
		headers: {
			'Content-Type': 'application/json',
			...(token ? { Authorization: `Bearer ${token}` } : {})
		},
		body: body ? JSON.stringify(body) : undefined
	});

	if (!res.ok) {
		let message = `Request failed (${res.status})`;
		const contentType = res.headers.get('content-type') ?? '';

		if (contentType.includes('application/json')) {
			const body = (await res.json()) as { error?: string };
			if (body.error) {
				message = body.error;
			}
		} else {
			const text = (await res.text()).trim();
			if (text) {
				message = text;
			}
		}

		throw new Error(message);
	}

	if (res.status === 204) {
		return undefined as T;
	}

	return (await res.json()) as T;
}

export const apiClient = {
	register: (username: string, email: string, password: string) =>
		request('/auth/register', 'POST', { username, email, password }),
	login: (username: string, password: string) => request('/auth/login', 'POST', { username, password }),
	refresh: (refreshToken: string) => request('/auth/refresh', 'POST', { refresh_token: refreshToken }),
	me: (accessToken: string) => request('/auth/me', 'GET', undefined, accessToken),
	friends: (accessToken: string) => request('/friends', 'GET', undefined, accessToken),
	sendFriendRequest: (accessToken: string, username: string) =>
		request('/friends/requests', 'POST', { username }, accessToken),
	pendingRequests: (accessToken: string) => request('/friends/requests', 'GET', undefined, accessToken),
	acceptFriendRequest: (accessToken: string, requestId: string) =>
		request(`/friends/requests/${requestId}/accept`, 'POST', undefined, accessToken),
	rejectFriendRequest: (accessToken: string, requestId: string) =>
		request(`/friends/requests/${requestId}`, 'DELETE', undefined, accessToken),
	servers: (accessToken: string) => request('/servers', 'GET', undefined, accessToken)
};

export const apiConfig = {
	baseUrl: API_BASE_URL
};
