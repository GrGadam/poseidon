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
	updated_at?: number | null;
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
	username?: string | null;
	avatar_mime?: string | null;
	content: string;
	created_at: number;
	updated_at?: number | null;
};

export type ServerResponse = {
	id: string;
	name: string;
	description?: string | null;
	owner_id: string;
	is_public: boolean;
	created_at: number;
	member_count?: number | null;
	member_role?: 'owner' | 'moderator' | 'user' | null;
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
	deleteFriend: (accessToken: string, friendUserId: string) =>
		request(`/friends/${encodeURIComponent(friendUserId)}`, 'DELETE', undefined, accessToken),
	createServer: (accessToken: string, name: string, description: string, isPublic: boolean) =>
		request<ServerResponse>('/servers', 'POST', { name, description, is_public: isPublic }, accessToken),
	servers: (accessToken: string) => request<ServerResponse[]>('/servers', 'GET', undefined, accessToken),
	publicServers: (accessToken: string, query?: string, sort?: string) => {
		const searchParams = new URLSearchParams();
		if (query?.trim()) {
			searchParams.set('query', query.trim());
		}
		if (sort?.trim()) {
			searchParams.set('sort', sort.trim());
		}

		const suffix = searchParams.toString();
		return request<ServerResponse[]>(`/servers/public${suffix ? `?${suffix}` : ''}`, 'GET', undefined, accessToken);
	},
	joinPublicServer: (accessToken: string, serverId: string) =>
		request(`/servers/${encodeURIComponent(serverId)}/join`, 'POST', undefined, accessToken),
	leaveServer: (accessToken: string, serverId: string) =>
		request(`/servers/${encodeURIComponent(serverId)}/leave`, 'POST', undefined, accessToken),
	serverChannels: (accessToken: string, serverId: string) =>
		request<ChannelResponse[]>(`/servers/${encodeURIComponent(serverId)}/channels`, 'GET', undefined, accessToken),
	createChannel: (accessToken: string, serverId: string, name: string, emoji: string) =>
		request('/servers/' + encodeURIComponent(serverId) + '/channels', 'POST', { name, emoji }, accessToken),
	updateChannel: (accessToken: string, channelId: string, name?: string, emoji?: string) => {
		const body: Record<string, unknown> = {};
		if (name !== undefined) body.name = name;
		if (emoji !== undefined) body.emoji = emoji;
		return request('/channels/' + encodeURIComponent(channelId), 'PATCH', body, accessToken);
	},
	deleteChannel: (accessToken: string, channelId: string) =>
		request('/channels/' + encodeURIComponent(channelId), 'DELETE', undefined, accessToken),
	updateServer: (accessToken: string, serverId: string, name?: string, description?: string, isPublic?: boolean) => {
		const body: Record<string, unknown> = {};
		if (name !== undefined) body.name = name;
		if (description !== undefined) body.description = description;
		if (isPublic !== undefined) body.is_public = isPublic;
		return request('/servers/' + encodeURIComponent(serverId), 'PUT', body, accessToken);
	},
	deleteServer: (accessToken: string, serverId: string) =>
		request('/servers/' + encodeURIComponent(serverId), 'DELETE', undefined, accessToken),
	dmThreads: (accessToken: string) => request<DmThreadResponse[]>('/dm/threads', 'GET', undefined, accessToken),
	dmCreateOrGetThread: (accessToken: string, userId: string) =>
		request<DmThreadResponse>('/dm/threads', 'POST', { user_id: userId }, accessToken),
	dmMessages: (accessToken: string, threadId: string) =>
		request<DmMessageResponse[]>(`/dm/threads/${encodeURIComponent(threadId)}/messages`, 'GET', undefined, accessToken),
	dmSendMessage: (accessToken: string, threadId: string, content: string) =>
		request<DmMessageResponse>(`/dm/threads/${encodeURIComponent(threadId)}/messages`, 'POST', { content }, accessToken),
	dmEditMessage: (accessToken: string, messageId: string, content: string) =>
		request(`/dm/messages/${encodeURIComponent(messageId)}`, 'PATCH', { content }, accessToken),
	dmDeleteMessage: (accessToken: string, messageId: string) =>
		request(`/dm/messages/${encodeURIComponent(messageId)}`, 'DELETE', undefined, accessToken),
	channelMessages: (accessToken: string, channelId: string) =>
		request<ChannelMessageResponse[]>(`/channels/${encodeURIComponent(channelId)}/messages`, 'GET', undefined, accessToken),
	channelSendMessage: (accessToken: string, channelId: string, content: string) =>
		request<ChannelMessageResponse>(`/channels/${encodeURIComponent(channelId)}/messages`, 'POST', { content }, accessToken),
	channelEditMessage: (accessToken: string, messageId: string, content: string) =>
		request(`/messages/${encodeURIComponent(messageId)}`, 'PATCH', { content }, accessToken),
	channelDeleteMessage: (accessToken: string, messageId: string) =>
		request(`/messages/${encodeURIComponent(messageId)}`, 'DELETE', undefined, accessToken),
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
	},
	uploadServerAvatar: async (accessToken: string, serverId: string, file: File): Promise<void> => {
		const formData = new FormData();
		formData.append('avatar', file);

		const res = await fetch(`${API_BASE_URL}/servers/${encodeURIComponent(serverId)}/avatar`, {
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
