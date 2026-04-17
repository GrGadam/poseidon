const API_BASE_URL = 'http://127.0.0.1:8080/api/v1';

type JsonBody = Record<string, unknown>;

export type DmThreadResponse = {
	id: string;
	peer_user: { id: string; username: string; avatar_mime?: string | null; created_at: number };
	created_at: number;
};

export type DmMessageResponse = {
	id: string;
	thread_id: string;
	user_id: string;
	content: string;
	created_at: number;
};

export type ChannelResponse = {
	id: string;
	server_id: string;
	name: string;
	emoji: string;
	created_at: number;
};

export type ChannelMessageResponse = {
	id: string;
	channel_id: string;
	user_id: string;
	content: string;
	created_at: number;
};

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
	servers: (accessToken: string) => request('/servers', 'GET', undefined, accessToken),
	serverChannels: (accessToken: string, serverId: string) =>
		request<ChannelResponse[]>(`/servers/${encodeURIComponent(serverId)}/channels`, 'GET', undefined, accessToken),
	dmThreads: (accessToken: string) => request<DmThreadResponse[]>('/dm/threads', 'GET', undefined, accessToken),
	dmCreateOrGetThread: (accessToken: string, userId: string) =>
		request<DmThreadResponse>('/dm/threads', 'POST', { user_id: userId }, accessToken),
	dmMessages: (accessToken: string, threadId: string) =>
		request<DmMessageResponse[]>(`/dm/threads/${encodeURIComponent(threadId)}/messages`, 'GET', undefined, accessToken),
	dmSendMessage: (accessToken: string, threadId: string, content: string) =>
		request<DmMessageResponse>(`/dm/threads/${encodeURIComponent(threadId)}/messages`, 'POST', { content }, accessToken),
	channelMessages: (accessToken: string, channelId: string) =>
		request<ChannelMessageResponse[]>(`/channels/${encodeURIComponent(channelId)}/messages`, 'GET', undefined, accessToken),
	channelSendMessage: (accessToken: string, channelId: string, content: string) =>
		request<ChannelMessageResponse>(`/channels/${encodeURIComponent(channelId)}/messages`, 'POST', { content }, accessToken),
	logout: (accessToken: string) => request('/auth/logout', 'POST', undefined, accessToken),
	changeMyPassword: (accessToken: string, currentPassword: string, newPassword: string) =>
		request('/users/me/password', 'POST', { current_password: currentPassword, new_password: newPassword }, accessToken),
	uploadMyAvatar: async (accessToken: string, file: File): Promise<void> => {
		const formData = new FormData();
		formData.append('avatar', file);

		const res = await fetch(`${API_BASE_URL}/users/me/avatar`, {
			method: 'POST',
			headers: {
				Authorization: `Bearer ${accessToken}`
			},
			body: formData
		});

		if (!res.ok) {
			let message = `Request failed (${res.status})`;
			const contentType = res.headers.get('content-type') ?? '';
			if (contentType.includes('application/json')) {
				const body = (await res.json()) as { error?: string };
				if (body.error) {
					message = body.error;
				}
			}
			throw new Error(message);
		}
	}
};

export const apiConfig = {
	baseUrl: API_BASE_URL
};
