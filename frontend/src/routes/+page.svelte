<script lang="ts">
	import { get } from 'svelte/store';
	import { onMount, tick } from 'svelte';
	import type {
		ChannelMessageResponse,
		ChannelResponse,
		DmMessageResponse,
		DmThreadResponse,
		InvitableFriendResponse,
		ServerInviteResponse,
		ServerResponse
	} from '$lib/api/client';
	import { apiClient, apiConfig } from '$lib/api/client';
	import { login, logout, register, session } from '$lib/stores/auth';
	import { selectedTab } from '$lib/stores/ui';
	import {
		acceptFriendRequest,
		clearFriendUnread,
		friendPending,
		friendOnline,
		friendOffline,
		friends,
		incrementFriendUnread,
		rejectFriendRequest,
		refreshFriends,
		refreshPendingRequests,
		sendFriendRequest,
		selectedFriend,
		selectFriend,
		markAvatarDirty,
		setFriendLastMessage,
		setFriendOnline
	} from '$lib/stores/friends';
	import { selectedServer, servers, selectServer } from '$lib/stores/servers';
	import type { FriendEntry, ServerEntry } from '$lib/types/domain';

	let loginIdentifier = $state('');
	let username = $state('');
	let email = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let authMode = $state<'login' | 'register'>('login');
	let authError = $state<string | null>(null);
	let addFriendModalOpen = $state(false);
	let pendingModalOpen = $state(false);
	let serverInviteModalOpen = $state(false);
	let createServerModalOpen = $state(false);
	let joinServerModalOpen = $state(false);
	let publicServersLoading = $state(false);
	let publicServersError = $state<string | null>(null);
	let publicServersSearch = $state('');
	type PublicServerSort = 'newest' | 'oldest' | 'most-members' | 'fewest-members' | 'name-asc' | 'name-desc';
	let publicServersSort = $state<PublicServerSort>('newest');
	let publicServers = $state<ServerEntry[]>([]);
	let joinServerConfirmModalOpen = $state(false);
	let joinServerTarget = $state<ServerEntry | null>(null);
	let joinServerLoading = $state(false);
	let joinServerMessage = $state<string | null>(null);
	let addFriendUsername = $state('');
	let addFriendMessage = $state<string | null>(null);
	let addFriendError = $state<string | null>(null);
	let createServerName = $state('');
	let createServerDescription = $state('');
	let createServerIsPublic = $state(true);
	let createServerAvatarFile = $state<File | null>(null);
	let createServerError = $state<string | null>(null);
	let createServerMessage = $state<string | null>(null);
	let createServerLoading = $state(false);
	let createChannelModalOpen = $state(false);
	let createChannelName = $state('');
	let createChannelEmoji = $state('💬');
	let createChannelError = $state<string | null>(null);
	let createChannelLoading = $state(false);
	let serverSettingsMenuOpen = $state(false);
	let serverSettingsMenuEl = $state<HTMLElement | null>(null);
	let createServerModalEl = $state<HTMLElement | null>(null);
	let joinServerModalEl = $state<HTMLElement | null>(null);
	let ignoreModalOutsideCloseUntil = $state(0);
	let channelSettingsMenuOpen = $state(false);
	let channelSettingsMenuEl = $state<HTMLElement | null>(null);
	let friendSettingsMenuOpen = $state(false);
	let friendSettingsMenuEl = $state<HTMLElement | null>(null);
	let updateChannelNameModalOpen = $state(false);
	let updateChannelNameValue = $state('');
	let updateChannelNameError = $state<string | null>(null);
	let updateChannelNameLoading = $state(false);
	let updateChannelEmojiModalOpen = $state(false);
	let updateChannelEmojiValue = $state('💬');
	let updateChannelEmojiError = $state<string | null>(null);
	let updateChannelEmojiLoading = $state(false);
	let deleteChannelModalOpen = $state(false);
	let deleteChannelConfirmText = $state('');
	let deleteChannelError = $state<string | null>(null);
	let deleteChannelLoading = $state(false);
	let createChannelEmojiPickerOpen = $state(false);
	let updateChannelEmojiPickerOpen = $state(false);
	let updateServerNameModalOpen = $state(false);
	let updateServerNameValue = $state('');
	let updateServerNameError = $state<string | null>(null);
	let updateServerNameLoading = $state(false);
	let updateServerDescriptionModalOpen = $state(false);
	let updateServerDescriptionValue = $state('');
	let updateServerDescriptionError = $state<string | null>(null);
	let updateServerDescriptionLoading = $state(false);
	let updateServerVisibilityModalOpen = $state(false);
	let updateServerVisibilityValue = $state(false);
	let updateServerVisibilityError = $state<string | null>(null);
	let updateServerVisibilityLoading = $state(false);
	let deleteServerModalOpen = $state(false);
	let deleteServerError = $state<string | null>(null);
	let deleteServerLoading = $state(false);
	let deleteServerConfirmText = $state('');
	let pendingActionError = $state<string | null>(null);
	let pendingActionInProgressId = $state<string | null>(null);
	let serverInvites = $state<
		Array<{
			inviteId: string;
			serverId: string;
			serverName: string;
			serverDescription: string;
			serverAvatarUrl: string | null;
			fromUserName: string;
			fromUserAvatarUrl: string | null;
			createdAt: number;
		}>
	>([]);
	let serverInviteActionError = $state<string | null>(null);
	let serverInviteActionInProgressId = $state<string | null>(null);
	let inviteUserModalOpen = $state(false);
	let inviteUserSearch = $state('');
	let inviteUserLoading = $state(false);
	let inviteUserError = $state<string | null>(null);
	let inviteUserMessage = $state<string | null>(null);
	let inviteUserInProgressId = $state<string | null>(null);
	let invitableFriends = $state<Array<{ id: string; username: string; avatarUrl: string | null }>>([]);
	let lastLoadedToken = $state<string | null>(null);
	let profileUploadMessage = $state<string | null>(null);
	let profileUploadError = $state<string | null>(null);
	let currentUserAvatarUrl = $state<string | null>(null);
	let serverAvatarUploadMessage = $state<string | null>(null);
	let serverAvatarUploadError = $state<string | null>(null);
	const transientTimeouts = new Map<string, ReturnType<typeof setTimeout>>();
	let passwordModalOpen = $state(false);
	let currentPasswordInput = $state('');
	let newPasswordInput = $state('');
	let newPasswordConfirmInput = $state('');
	let passwordChangeError = $state<string | null>(null);
	let passwordChangeMessage = $state<string | null>(null);
	let passwordChangeLoading = $state(false);
	let avatarInputEl = $state<HTMLInputElement | null>(null);
	let serverAvatarInputEl = $state<HTMLInputElement | null>(null);
	let settingsMenuOpen = $state(false);
	let settingsMenuEl = $state<HTMLElement | null>(null);

	type ChatView = 'none' | 'friend' | 'server' | 'server-channels';
	type ServerChannel = { id: string; name: string; emoji: string };
	type ChatMessage = {
		id: string;
		userId: string;
		authorName?: string;
		authorAvatarUrl?: string | null;
		content: string;
		createdAt: number;
		updatedAt?: number | null;
	};

	type MessagePart =
		| { type: 'text'; value: string }
		| { type: 'link'; url: string }
		| { type: 'image'; url: string }
		| { type: 'video'; url: string };

	let activeChat = $state<ChatView>('none');
	let selectedChannel = $state<ServerChannel | null>(null);
	let activeDmThreadId = $state<string | null>(null);
	let chatMessages = $state<ChatMessage[]>([]);
	let chatInput = $state('');
	let chatInputEl = $state<HTMLInputElement | null>(null);
	let chatEmojiPickerOpen = $state(false);
	let chatEmojiPickerEl = $state<HTMLElement | null>(null);
	let chatEmojiToggleButtonEl = $state<HTMLButtonElement | null>(null);
	let chatLoading = $state(false);
	let chatSending = $state(false);
	let chatError = $state<string | null>(null);
	let chatContainer = $state<HTMLDivElement | null>(null);
	let chatReloadTimeout: ReturnType<typeof setTimeout> | null = null;

	let serverChannels = $state<Record<string, ServerChannel[]>>({});
	let channelUnreadByServer = $state<Record<string, Record<string, number>>>({});
	let dmThreadByFriendId = $state<Record<string, string>>({});
	let friendByDmThreadId = $state<Record<string, string>>({});
	let onlineUserIdsSnapshot = $state<string[]>([]);
	let leaveServerLoading = $state(false);
	let leaveServerError = $state<string | null>(null);
	let messageAuthorAvatarUrls = $state<Record<string, string | null>>({});
	let messageActionsOpenForId = $state<string | null>(null);
	let editingMessageId = $state<string | null>(null);

	const clearTransientTimeout = (key: string) => {
		const existing = transientTimeouts.get(key);
		if (existing) {
			clearTimeout(existing);
			transientTimeouts.delete(key);
		}
	};

	const URL_REGEX = /(https?:\/\/[^\s<]+)/g;

	const trimLinkSuffix = (url: string): { cleanUrl: string; suffix: string } => {
		let cleanUrl = url;
		let suffix = '';
		while (cleanUrl.length > 0 && /[),.!?:;\]]/.test(cleanUrl[cleanUrl.length - 1] ?? '')) {
			suffix = `${cleanUrl[cleanUrl.length - 1]}${suffix}`;
			cleanUrl = cleanUrl.slice(0, -1);
		}

		return { cleanUrl, suffix };
	};

	const isImageLikeUrl = (url: string): boolean => {
		try {
			const parsed = new URL(url);
			const pathname = parsed.pathname.toLowerCase();
			return /\.(png|jpe?g|gif|webp|bmp|avif|svg)$/.test(pathname);
		} catch {
			return false;
		}
	};

	const isVideoLikeUrl = (url: string): boolean => {
		try {
			const parsed = new URL(url);
			const pathname = parsed.pathname.toLowerCase();
			return /\.(mp4|webm|ogg|mov|m4v)$/.test(pathname);
		} catch {
			return false;
		}
	};

	const parseMessageParts = (content: string): MessagePart[] => {
		const parts: MessagePart[] = [];
		let cursor = 0;

		for (const match of content.matchAll(URL_REGEX)) {
			const rawUrl = match[0];
			const start = match.index ?? 0;
			if (start > cursor) {
				parts.push({ type: 'text', value: content.slice(cursor, start) });
			}

			const { cleanUrl, suffix } = trimLinkSuffix(rawUrl);
			if (cleanUrl) {
				if (isVideoLikeUrl(cleanUrl)) {
					parts.push({ type: 'video', url: cleanUrl });
				} else if (isImageLikeUrl(cleanUrl)) {
					parts.push({ type: 'image', url: cleanUrl });
				} else {
					parts.push({ type: 'link', url: cleanUrl });
				}
			} else {
				parts.push({ type: 'text', value: rawUrl });
			}

			if (suffix) {
				parts.push({ type: 'text', value: suffix });
			}

			cursor = start + rawUrl.length;
		}

		if (cursor < content.length) {
			parts.push({ type: 'text', value: content.slice(cursor) });
		}

		if (parts.length === 0) {
			parts.push({ type: 'text', value: content });
		}

		return parts;
	};

	const openMessageUrl = async (event: MouseEvent, url: string) => {
		event.preventDefault();

		try {
			if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
				const opener = await import('@tauri-apps/plugin-opener');
				await opener.openUrl(url);
				return;
			}
		} catch {
			// Fall back to browser tab open below.
		}

		window.open(url, '_blank', 'noopener,noreferrer');
	};

	const scheduleTransientClear = (key: string, clearFn: () => void, ms = 5000) => {
		clearTransientTimeout(key);
		const timeout = setTimeout(() => {
			clearFn();
			transientTimeouts.delete(key);
		}, ms);
		transientTimeouts.set(key, timeout);
	};

	const loadUserAvatarUrl = async (accessToken: string, userId: string): Promise<string | null> => {
		const res = await fetch(`${apiConfig.baseUrl}/users/${encodeURIComponent(userId)}/avatar`, {
			cache: 'no-store',
			headers: {
				Authorization: `Bearer ${accessToken}`
			}
		});

		if (!res.ok) {
			return null;
		}

		const blob = await res.blob();
		if (blob.size === 0) {
			return null;
		}

		return URL.createObjectURL(blob);
	};

	const loadServerAvatarUrl = async (accessToken: string, serverId: string): Promise<string | null> => {
		const res = await fetch(`${apiConfig.baseUrl}/servers/${encodeURIComponent(serverId)}/avatar`, {
			cache: 'no-store',
			headers: {
				Authorization: `Bearer ${accessToken}`
			}
		});

		if (!res.ok) {
			return null;
		}

		const blob = await res.blob();
		if (blob.size === 0) {
			return null;
		}

		return URL.createObjectURL(blob);
	};

	const setCurrentUserAvatarUrl = (nextUrl: string | null) => {
		if (
			currentUserAvatarUrl &&
			currentUserAvatarUrl.startsWith('blob:') &&
			currentUserAvatarUrl !== nextUrl
		) {
			URL.revokeObjectURL(currentUserAvatarUrl);
		}

		currentUserAvatarUrl = nextUrl;
	};

	const refreshCurrentUserAvatar = async () => {
		const token = $session.accessToken;
		const userId = $session.userId;
		if (!token || !userId) {
			setCurrentUserAvatarUrl(null);
			return;
		}

		const avatarUrl = await loadUserAvatarUrl(token, userId);
		setCurrentUserAvatarUrl(avatarUrl);
	};

	const getOrLoadMessageAuthorAvatarUrl = async (userId: string): Promise<string | null> => {
		const cached = messageAuthorAvatarUrls[userId];
		if (cached !== undefined) {
			return cached;
		}

		const token = $session.accessToken;
		if (!token) {
			messageAuthorAvatarUrls = {
				...messageAuthorAvatarUrls,
				[userId]: null
			};
			return null;
		}

		const avatarUrl = await loadUserAvatarUrl(token, userId);
		messageAuthorAvatarUrls = {
			...messageAuthorAvatarUrls,
			[userId]: avatarUrl
		};
		return avatarUrl;
	};

	const clearMessageAuthorAvatarCache = () => {
		for (const avatarUrl of Object.values(messageAuthorAvatarUrls)) {
			if (avatarUrl?.startsWith('blob:')) {
				URL.revokeObjectURL(avatarUrl);
			}
		}
		messageAuthorAvatarUrls = {};
	};

	const seenDmCreatedIds = new Set<string>();
	const seenDmCreatedOrder: string[] = [];

	const rememberDmCreatedId = (messageId: string): boolean => {
		if (seenDmCreatedIds.has(messageId)) {
			return false;
		}

		seenDmCreatedIds.add(messageId);
		seenDmCreatedOrder.push(messageId);

		if (seenDmCreatedOrder.length > 1000) {
			const oldest = seenDmCreatedOrder.shift();
			if (oldest) {
				seenDmCreatedIds.delete(oldest);
			}
		}

		return true;
	};

	const switchAuthMode = () => {
		authMode = authMode === 'login' ? 'register' : 'login';
		authError = null;
		password = '';
		confirmPassword = '';
	};

	const submitAuth = async () => {
		authError = null;

		try {
			if (authMode === 'login') {
				if (!loginIdentifier.trim() || !password) {
					authError = 'Enter your username/email and password.';
					return;
				}

				await login(loginIdentifier.trim(), password);
				return;
			}

			if (!username.trim()) {
				authError = 'Username is required.';
				return;
			}

			if (!email.includes('@')) {
				authError = 'Enter a valid email address.';
				return;
			}

			if (password.length < 6) {
				authError = 'Password must be at least 6 characters.';
				return;
			}

			if (password !== confirmPassword) {
				authError = 'The passwords do not match.';
				return;
			}

			await register(username.trim(), email.trim().toLowerCase(), password);
		} catch (error) {
			authError = error instanceof Error ? error.message : 'Authentication failed.';
		}
	};

	const refreshFriendThreadCache = async () => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		const threads = await apiClient.dmThreads(token);
		dmThreadByFriendId = {};
		friendByDmThreadId = {};

		for (const thread of threads) {
			dmThreadByFriendId[thread.peer_user.id] = thread.id;
			friendByDmThreadId[thread.id] = thread.peer_user.id;
		}
	};

	const refreshFriendLatestMessages = async () => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		for (const [friendId, threadId] of Object.entries(dmThreadByFriendId)) {
			const messages = await apiClient.dmMessages(token, threadId);
			const latest = messages[0];
			setFriendLastMessage(friendId, latest?.content ?? '');
		}
	};

	const clearServerInviteAvatars = () => {
		for (const invite of serverInvites) {
			if (invite.serverAvatarUrl?.startsWith('blob:')) {
				URL.revokeObjectURL(invite.serverAvatarUrl);
			}
			if (invite.fromUserAvatarUrl?.startsWith('blob:')) {
				URL.revokeObjectURL(invite.fromUserAvatarUrl);
			}
		}
	};

	const clearInvitableFriendsAvatars = () => {
		for (const item of invitableFriends) {
			if (item.avatarUrl?.startsWith('blob:')) {
				URL.revokeObjectURL(item.avatarUrl);
			}
		}
	};

	const refreshServerInvites = async () => {
		const token = $session.accessToken;
		if (!token) {
			clearServerInviteAvatars();
			serverInvites = [];
			return;
		}

		const data = await apiClient.pendingServerInvites(token);
		clearServerInviteAvatars();

		const nextInvites = await Promise.all(
			data.map(async (item: ServerInviteResponse) => {
				const serverAvatarUrl = await loadServerAvatarUrl(token, item.server.id);
				const fromUserAvatarUrl = await loadUserAvatarUrl(token, item.from_user.id);
				return {
					inviteId: item.id,
					serverId: item.server.id,
					serverName: item.server.name,
					serverDescription: item.server.description ?? '',
					serverAvatarUrl,
					fromUserName: item.from_user.username,
					fromUserAvatarUrl,
					createdAt: item.created_at
				};
			})
		);

		serverInvites = nextInvites;
	};

	const loadInvitableFriends = async () => {
		const token = $session.accessToken;
		const serverId = $selectedServer?.id;
		if (!token || !serverId) {
			return;
		}

		inviteUserLoading = true;
		inviteUserError = null;

		try {
			const data = await apiClient.serverInvitableFriends(token, serverId);
			clearInvitableFriendsAvatars();
			invitableFriends = await Promise.all(
				data.map(async (entry: InvitableFriendResponse) => ({
					id: entry.id,
					username: entry.username,
					avatarUrl: await loadUserAvatarUrl(token, entry.id)
				}))
			);
		} catch (error) {
			inviteUserError = error instanceof Error ? error.message : 'Failed to load friends for invite.';
		} finally {
			inviteUserLoading = false;
		}
	};

	const refreshFriendsData = async () => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		await Promise.all([refreshFriends(token), refreshPendingRequests(token)]);
		await refreshFriendThreadCache();
		await refreshFriendLatestMessages();
		applyPresenceSnapshot(onlineUserIdsSnapshot);
	};

	const refreshServersData = async () => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		leaveServerError = null;
		const data = await apiClient.servers(token);

		const previousServers = get(servers);
		const previousById = new Map(previousServers.map((entry) => [entry.id, entry]));
		const incomingIds = new Set(data.map((item) => item.id));
		const selectedServerId = $selectedServer?.id ?? null;

		for (const serverId of Object.keys(channelUnreadByServer)) {
			if (!incomingIds.has(serverId)) {
				delete channelUnreadByServer[serverId];
			}
		}

		for (const entry of previousServers) {
			if (!incomingIds.has(entry.id) && entry.avatarUrl?.startsWith('blob:')) {
				URL.revokeObjectURL(entry.avatarUrl);
			}
		}

		const nextServers = await Promise.all(
			data.map(async (item) => {
				const previous = previousById.get(item.id);
				const previousAvatarUrl = previous?.avatarUrl ?? null;
				const avatarUrl = await loadServerAvatarUrl(token, item.id);
				const unreadForServer = channelUnreadByServer[item.id] ?? {};
				const hasUnread = Object.values(unreadForServer).some((count) => count > 0);

				if (
					previousAvatarUrl &&
					previousAvatarUrl.startsWith('blob:') &&
					previousAvatarUrl !== avatarUrl
				) {
					URL.revokeObjectURL(previousAvatarUrl);
				}

				return {
					id: item.id,
					name: item.name,
					description: item.description ?? '',
					isPublic: item.is_public,
					avatarUrl,
					hasUnread,
					createdAt: item.created_at,
					memberCount: item.member_count ?? undefined,
					memberRole: item.member_role ?? undefined
				};
			})
		);

		servers.set(nextServers);

		if (selectedServerId) {
			const refreshedSelected = nextServers.find((entry) => entry.id === selectedServerId) ?? null;
			if (refreshedSelected) {
				selectServer(refreshedSelected);
			} else {
				selectedServer.set(null);
				activeChat = 'none';
				selectedChannel = null;
				chatMessages = [];
				chatError = null;
				chatInput = '';
			}
		}
	};

	const updateServerUnreadDot = (serverId: string) => {
		const unreadByChannel = channelUnreadByServer[serverId] ?? {};
		const hasUnread = Object.values(unreadByChannel).some((count) => count > 0);

		servers.update((items) =>
			items.map((item) => (item.id === serverId ? { ...item, hasUnread } : item))
		);

		if ($selectedServer?.id === serverId) {
			selectServer({ ...$selectedServer, hasUnread });
		}
	};

	const incrementChannelUnread = (serverId: string, channelId: string) => {
		const isMember = get(servers).some((entry) => entry.id === serverId);
		if (!isMember) {
			return;
		}

		const byChannel = channelUnreadByServer[serverId] ?? {};
		const current = byChannel[channelId] ?? 0;
		channelUnreadByServer = {
			...channelUnreadByServer,
			[serverId]: {
				...byChannel,
				[channelId]: current + 1
			}
		};
		updateServerUnreadDot(serverId);
	};

	const clearChannelUnread = (serverId: string, channelId: string) => {
		const byChannel = channelUnreadByServer[serverId];
		if (!byChannel || !byChannel[channelId]) {
			return;
		}

		const nextByChannel = { ...byChannel };
		delete nextByChannel[channelId];

		if (Object.keys(nextByChannel).length === 0) {
			const nextUnread = { ...channelUnreadByServer };
			delete nextUnread[serverId];
			channelUnreadByServer = nextUnread;
		} else {
			channelUnreadByServer = {
				...channelUnreadByServer,
				[serverId]: nextByChannel
			};
		}

		updateServerUnreadDot(serverId);
	};

	const getChannelUnread = (serverId: string, channelId: string): number => {
		return channelUnreadByServer[serverId]?.[channelId] ?? 0;
	};

	const selectedServerRole = (): 'owner' | 'moderator' | 'user' | null => {
		const role = $selectedServer?.memberRole;
		return role === 'owner' || role === 'moderator' || role === 'user' ? role : null;
	};

	const canManageSelectedServer = () => {
		const role = selectedServerRole();
		return role === 'owner' || role === 'moderator';
	};

	const canLeaveSelectedServer = () => selectedServerRole() === 'user';

	const filteredInvitableFriends = () => {
		const needle = inviteUserSearch.trim().toLowerCase();
		if (!needle) {
			return invitableFriends;
		}

		return invitableFriends.filter((entry) => entry.username.toLowerCase().includes(needle));
	};

	const loadPublicServers = async () => {
		const token = $session.accessToken;
		if (!token || !joinServerModalOpen) {
			return;
		}

		publicServersLoading = true;
		publicServersError = null;
		joinServerMessage = null;

		try {
			const data = await apiClient.publicServers(token, publicServersSearch, publicServersSort);

			const previousById = new Map(publicServers.map((item) => [item.id, item]));
			const incomingIds = new Set(data.map((item) => item.id));

			for (const entry of publicServers) {
				if (!incomingIds.has(entry.id) && entry.avatarUrl?.startsWith('blob:')) {
					URL.revokeObjectURL(entry.avatarUrl);
				}
			}

			const nextPublicServers = await Promise.all(
				data.map(async (item: ServerResponse) => {
					const previous = previousById.get(item.id);
					const previousAvatarUrl = previous?.avatarUrl ?? null;

					const avatarUrl = await loadServerAvatarUrl(token, item.id);

					if (
						previousAvatarUrl &&
						previousAvatarUrl.startsWith('blob:') &&
						previousAvatarUrl !== avatarUrl
					) {
						URL.revokeObjectURL(previousAvatarUrl);
					}

					return {
						id: item.id,
						name: item.name,
						description: item.description ?? '',
						isPublic: item.is_public,
						avatarUrl,
						hasUnread: false,
						createdAt: item.created_at,
						memberCount: item.member_count ?? undefined
					};
				})
			);

			publicServers = nextPublicServers;
		} catch (error) {
			publicServersError = error instanceof Error ? error.message : 'Failed to load public servers.';
		} finally {
			publicServersLoading = false;
		}
	};

	const toChatMessage = async (item: DmMessageResponse | ChannelMessageResponse): Promise<ChatMessage> => {
		const base: ChatMessage = {
			id: item.id,
			userId: item.user_id,
			content: item.content,
			createdAt: item.created_at,
			updatedAt: item.updated_at ?? null
		};

		if ('channel_id' in item) {
			const authorAvatarUrl = await getOrLoadMessageAuthorAvatarUrl(item.user_id);
			return {
				...base,
				authorName: item.username ?? undefined,
				authorAvatarUrl
			};
		}

		return base;
	};

	const byCreatedAtAsc = (a: ChatMessage, b: ChatMessage) => a.createdAt - b.createdAt;

	const scrollChatOnOpen = (unreadCount: number) => {
		setTimeout(() => {
			if (!chatContainer) {
				return;
			}

			if (unreadCount > 0 && chatMessages.length > 0) {
				const firstUnreadIndex = Math.max(0, chatMessages.length - unreadCount);
				const messageNodes = chatContainer.querySelectorAll('[data-msg-index]');
				const target = messageNodes[firstUnreadIndex] as HTMLElement | undefined;
				if (target) {
					target.scrollIntoView({ behavior: 'auto', block: 'start' });
					return;
				}
			}

			chatContainer.scrollTop = chatContainer.scrollHeight;
		}, 0);
	};

	const keepChatAtBottomOnMediaLoad = (messageIndex: number) => {
		setTimeout(() => {
			if (!chatContainer) {
				return;
			}

			const distanceFromBottom =
				chatContainer.scrollHeight - (chatContainer.scrollTop + chatContainer.clientHeight);
			const isRecentMessage = messageIndex >= Math.max(0, chatMessages.length - 4);

			if (isRecentMessage || distanceFromBottom <= 180) {
				chatContainer.scrollTop = chatContainer.scrollHeight;
			}
		}, 0);
	};

	const loadActiveMessages = async (options?: { silent?: boolean; forceBottom?: boolean }) => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		const silent = options?.silent ?? false;
		const forceBottom = options?.forceBottom ?? false;
		let previousScrollTop = 0;
		let previousScrollHeight = 0;
		let shouldStickToBottom = false;

		if (silent && chatContainer) {
			previousScrollTop = chatContainer.scrollTop;
			previousScrollHeight = chatContainer.scrollHeight;
			const distanceFromBottom = chatContainer.scrollHeight - (chatContainer.scrollTop + chatContainer.clientHeight);
			shouldStickToBottom = forceBottom || distanceFromBottom <= 32;
		}

		if (!silent) {
			chatLoading = true;
		}
		chatError = null;

		try {
			let nextMessages: ChatMessage[] | null = null;

			if (activeChat === 'friend' && activeDmThreadId) {
				const data = await apiClient.dmMessages(token, activeDmThreadId);
				nextMessages = (await Promise.all(data.map(toChatMessage))).sort(byCreatedAtAsc);
			} else if (activeChat === 'server' && selectedChannel?.id) {
				const data = await apiClient.channelMessages(token, selectedChannel.id);
				nextMessages = (await Promise.all(data.map(toChatMessage))).sort(byCreatedAtAsc);
			}

			chatMessages = nextMessages ?? [];
		} catch (error) {
			chatError = error instanceof Error ? error.message : 'Failed to load messages.';
		} finally {
			if (!silent) {
				chatLoading = false;
			}
		}

		if (silent && chatContainer) {
			await tick();
			if (!chatContainer) {
				return;
			}

			if (shouldStickToBottom) {
				chatContainer.scrollTop = chatContainer.scrollHeight;
			} else {
				const heightDelta = chatContainer.scrollHeight - previousScrollHeight;
				chatContainer.scrollTop = Math.max(0, previousScrollTop + heightDelta);
			}
		}
	};

	const scheduleActiveMessagesReload = (options?: { silent?: boolean; forceBottom?: boolean }) => {
		if (chatReloadTimeout) {
			clearTimeout(chatReloadTimeout);
			chatReloadTimeout = null;
		}

		chatReloadTimeout = setTimeout(() => {
			chatReloadTimeout = null;
			void loadActiveMessages(options);
		}, 120);
	};

	const loadServerChannels = async (serverId: string) => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		const data = await apiClient.serverChannels(token, serverId);
		serverChannels = {
			...serverChannels,
			[serverId]: data.map((channel: ChannelResponse) => ({
				id: channel.id,
				name: channel.name,
				emoji: channel.emoji
			}))
		};
	};

	const resolveFriendIdByThread = async (threadId: string): Promise<string | null> => {
		const cached = friendByDmThreadId[threadId];
		if (cached) {
			return cached;
		}

		await refreshFriendThreadCache();
		return friendByDmThreadId[threadId] ?? null;
	};

	const updateFriendPreviewFromThread = async (threadId: string) => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		const friendId = await resolveFriendIdByThread(threadId);
		if (!friendId) {
			return;
		}

		const messages = await apiClient.dmMessages(token, threadId);
		const latest = messages[0];
		setFriendLastMessage(friendId, latest?.content ?? '');
	};

	const applyPresenceSnapshot = (onlineUserIds: string[]) => {
		const onlineSet = new Set(onlineUserIds);
		for (const friend of get(friends)) {
			setFriendOnline(friend.id, onlineSet.has(friend.id));
		}
	};

	const handleSendChatMessage = async () => {
		const token = $session.accessToken;
		if (!token || chatSending) {
			return;
		}

		const content = chatInput.trim();
		if (!content && !editingMessageId) {
			return;
		}

		chatSending = true;
		chatError = null;

		try {
			if (editingMessageId) {
				const messageId = editingMessageId;
				const targetMessage = chatMessages.find((msg) => msg.id === messageId);
				if (!targetMessage) {
					editingMessageId = null;
					chatInput = '';
					chatError = 'Message no longer exists.';
					return;
				}

				if (!content) {
					chatError = 'Message cannot be empty.';
					return;
				}

				if (content === targetMessage.content) {
					editingMessageId = null;
					chatInput = '';
					return;
				}

				const previousMessages = chatMessages;
				const now = Math.floor(Date.now() / 1000);
				chatMessages = chatMessages.map((msg) =>
					msg.id === messageId
						? {
							...msg,
							content,
							updatedAt: now
						}
						: msg
				);

				if (activeChat === 'friend') {
					await apiClient.dmEditMessage(token, messageId, content);
					editingMessageId = null;
					chatInput = '';
					chatEmojiPickerOpen = false;
					return;
				}

				if (activeChat === 'server') {
					await apiClient.channelEditMessage(token, messageId, content);
					editingMessageId = null;
					chatInput = '';
					chatEmojiPickerOpen = false;
					return;
				}

				chatMessages = previousMessages;
				chatError = 'No active chat selected.';
				return;
			}

			if (activeChat === 'friend' && activeDmThreadId) {
				const sent = await apiClient.dmSendMessage(token, activeDmThreadId, content);
				const mapped = await toChatMessage(sent);
				chatMessages = [...chatMessages, mapped].sort(byCreatedAtAsc);
				await tick();
				if (chatContainer) {
					chatContainer.scrollTop = chatContainer.scrollHeight;
				}
				const friendId = friendByDmThreadId[activeDmThreadId];
				if (friendId) {
					setFriendLastMessage(friendId, sent.content);
					clearFriendUnread(friendId);
				}
				chatInput = '';
				chatEmojiPickerOpen = false;
				return;
			}

			if (activeChat === 'server' && selectedChannel?.id) {
				const sent = await apiClient.channelSendMessage(token, selectedChannel.id, content);
				const mapped = await toChatMessage(sent);
				chatMessages = [...chatMessages, mapped].sort(byCreatedAtAsc);
				await tick();
				if (chatContainer) {
					chatContainer.scrollTop = chatContainer.scrollHeight;
				}
				chatInput = '';
				chatEmojiPickerOpen = false;
				return;
			}

			chatError = 'No active chat selected.';
		} catch (error) {
			chatError = error instanceof Error ? error.message : 'Failed to send message.';
		} finally {
			chatSending = false;
		}
	};

	const handleDeleteChatMessage = async (messageId: string) => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		const previousMessages = chatMessages;
		messageActionsOpenForId = null;
		if (editingMessageId === messageId) {
			editingMessageId = null;
			chatInput = '';
		}
		chatError = null;
		chatMessages = chatMessages.filter((msg) => msg.id !== messageId);

		try {
			if (activeChat === 'friend') {
				await apiClient.dmDeleteMessage(token, messageId);
				return;
			}

			if (activeChat === 'server') {
				await apiClient.channelDeleteMessage(token, messageId);
				return;
			}

			chatMessages = previousMessages;
		} catch (error) {
			chatMessages = previousMessages;
			chatError = error instanceof Error ? error.message : 'Failed to delete message.';
		}
	};

	const handleStartEditChatMessage = (messageId: string, content: string) => {
		messageActionsOpenForId = null;
		chatError = null;
		editingMessageId = messageId;
		chatInput = content;
		void tick().then(() => {
			chatInputEl?.focus();
			chatInputEl?.setSelectionRange(0, chatInputEl.value.length);
		});
	};

	const handleCancelEditChatMessage = () => {
		editingMessageId = null;
		chatInput = '';
	};

	const handleCreateFriendRequest = async () => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		addFriendError = null;
		addFriendMessage = null;

		const name = addFriendUsername.trim();
		if (!name) {
			addFriendError = 'Enter a username.';
			scheduleTransientClear('addFriendError', () => {
				addFriendError = null;
			});
			return;
		}

		try {
			await sendFriendRequest(token, name);
			addFriendUsername = '';
			addFriendMessage = 'Friend request sent.';
			scheduleTransientClear('addFriendMessage', () => {
				addFriendMessage = null;
			});
		} catch (error) {
			if (error instanceof Error) {
				if (error.message === 'not_found') {
					addFriendError = 'User not found.';
					scheduleTransientClear('addFriendError', () => {
						addFriendError = null;
					});
					return;
				}
				if (error.message.includes('already exists') || error.message.includes('request already exists')) {
					addFriendError = 'A pending request already exists for this user.';
					scheduleTransientClear('addFriendError', () => {
						addFriendError = null;
					});
					return;
				}
				addFriendError = error.message;
				scheduleTransientClear('addFriendError', () => {
					addFriendError = null;
				});
				return;
			}

			addFriendError = 'Failed to send friend request.';
			scheduleTransientClear('addFriendError', () => {
				addFriendError = null;
			});
		}
	};

	const handlePendingDecision = async (requestId: string, action: 'accept' | 'reject') => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		pendingActionInProgressId = requestId;
		pendingActionError = null;

		try {
			if (action === 'accept') {
				await acceptFriendRequest(token, requestId);
			} else {
				await rejectFriendRequest(token, requestId);
			}

			if ($friendPending.length === 0) {
				pendingModalOpen = false;
			}
		} catch (error) {
			pendingActionError = error instanceof Error ? error.message : 'Failed to process request.';
		} finally {
			pendingActionInProgressId = null;
		}
	};

	const handleServerInviteDecision = async (inviteId: string, action: 'accept' | 'reject') => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		serverInviteActionInProgressId = inviteId;
		serverInviteActionError = null;

		try {
			if (action === 'accept') {
				await apiClient.acceptServerInvite(token, inviteId);
			} else {
				await apiClient.rejectServerInvite(token, inviteId);
			}

			await Promise.all([refreshServerInvites(), refreshServersData()]);

			if (serverInvites.length === 0) {
				serverInviteModalOpen = false;
			}
		} catch (error) {
			serverInviteActionError = error instanceof Error ? error.message : 'Failed to process server invite.';
			scheduleTransientClear('serverInviteActionError', () => {
				serverInviteActionError = null;
			});
		} finally {
			serverInviteActionInProgressId = null;
		}
	};

	const openServerInviteModal = () => {
		serverInviteModalOpen = true;
		serverInviteActionError = null;
	};

	const openInviteUserModal = async () => {
		if (!canManageSelectedServer() || $selectedServer?.isPublic !== false) {
			return;
		}

		inviteUserModalOpen = true;
		inviteUserSearch = '';
		inviteUserError = null;
		inviteUserMessage = null;
		await loadInvitableFriends();
	};

	const closeInviteUserModal = () => {
		inviteUserModalOpen = false;
		inviteUserSearch = '';
		inviteUserError = null;
		inviteUserMessage = null;
		inviteUserInProgressId = null;
		clearInvitableFriendsAvatars();
		invitableFriends = [];
	};

	const handleInviteUserToSelectedServer = async (userId: string, username: string) => {
		const token = $session.accessToken;
		const serverId = $selectedServer?.id;
		if (!token || !serverId) {
			return;
		}

		inviteUserInProgressId = userId;
		inviteUserError = null;
		inviteUserMessage = null;

		try {
			await apiClient.inviteUserToServer(token, serverId, userId);
			inviteUserMessage = `${username} invited.`;
			scheduleTransientClear('inviteUserMessage', () => {
				inviteUserMessage = null;
			});
			clearInvitableFriendsAvatars();
			invitableFriends = invitableFriends.filter((item) => item.id !== userId);
			await refreshServerInvites();
		} catch (error) {
			inviteUserError = error instanceof Error ? error.message : 'Failed to send invite.';
			scheduleTransientClear('inviteUserError', () => {
				inviteUserError = null;
			});
		} finally {
			inviteUserInProgressId = null;
		}
	};

	const openFriendChat = async (friend: FriendEntry) => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		const unreadCount = friend.unread;

		selectFriend(friend);
		selectedChannel = null;
		chatInput = '';
		editingMessageId = null;
		chatEmojiPickerOpen = false;
		chatMessages = [];
		chatError = null;
		activeChat = 'friend';
		friendSettingsMenuOpen = false;
		clearFriendUnread(friend.id);

		try {
			let threadId = dmThreadByFriendId[friend.id] ?? null;
			if (!threadId) {
				const thread = (await apiClient.dmCreateOrGetThread(token, friend.id)) as DmThreadResponse;
				threadId = thread.id;
				dmThreadByFriendId[friend.id] = thread.id;
				friendByDmThreadId[thread.id] = friend.id;
			}

			activeDmThreadId = threadId;
			await loadActiveMessages();
			scrollChatOnOpen(unreadCount);
		} catch (error) {
			chatError = error instanceof Error ? error.message : 'Failed to open direct message chat.';
		}
	};

	const openServerChannels = async (server: ServerEntry) => {
		selectServer(server);
		activeChat = 'server-channels';
		friendSettingsMenuOpen = false;
		leaveServerError = null;
		selectedChannel = null;
		activeDmThreadId = null;
		chatMessages = [];
		chatError = null;
		chatInput = '';
		editingMessageId = null;
		chatEmojiPickerOpen = false;

		try {
			await loadServerChannels(server.id);
		} catch (error) {
			chatError = error instanceof Error ? error.message : 'Failed to load channels.';
		}
	};

	const openServerChannelChat = async (channel: ServerChannel) => {
		const serverId = $selectedServer?.id;
		if (serverId) {
			clearChannelUnread(serverId, channel.id);
		}

		selectedChannel = channel;
		channelSettingsMenuOpen = false;
		activeDmThreadId = null;
		chatInput = '';
		editingMessageId = null;
		chatEmojiPickerOpen = false;
		chatMessages = [];
		chatError = null;
		activeChat = 'server';
		await loadActiveMessages();
		scrollChatOnOpen(0);
	};

	const closeChat = () => {
		activeChat = 'none';
		channelSettingsMenuOpen = false;
		friendSettingsMenuOpen = false;
		activeDmThreadId = null;
		chatMessages = [];
		chatError = null;
		chatInput = '';
		editingMessageId = null;
		chatEmojiPickerOpen = false;
	};

	const handleChatBack = () => {
		if (activeChat === 'server') {
			activeChat = 'server-channels';
			selectedChannel = null;
			channelSettingsMenuOpen = false;
			chatMessages = [];
			chatError = null;
			chatInput = '';
			editingMessageId = null;
			chatEmojiPickerOpen = false;
			return;
		}

		closeChat();
	};

	const backToServerList = () => {
		activeChat = 'none';
		channelSettingsMenuOpen = false;
		leaveServerError = null;
	};

	const handleLeaveSelectedServer = async () => {
		const token = $session.accessToken;
		const serverId = $selectedServer?.id;
		if (!token || !serverId || leaveServerLoading) {
			return;
		}

		leaveServerLoading = true;
		leaveServerError = null;

		try {
			await apiClient.leaveServer(token, serverId);
			serverSettingsMenuOpen = false;
			activeChat = 'none';
			selectedChannel = null;
			chatMessages = [];
			chatError = null;
			chatInput = '';
			editingMessageId = null;
			selectedServer.set(null);
			await refreshServersData();
			if (joinServerModalOpen) {
				await loadPublicServers();
			}
		} catch (error) {
			leaveServerError = error instanceof Error ? error.message : 'Failed to leave server.';
		} finally {
			leaveServerLoading = false;
		}
	};

	const openAddFriendModal = () => {
		addFriendModalOpen = true;
		addFriendError = null;
		addFriendMessage = null;
	};

	const openPendingModal = () => {
		pendingModalOpen = true;
		pendingActionError = null;
	};

	const openCreateServerModal = () => {
		ignoreModalOutsideCloseUntil = Date.now() + 200;
		createServerModalOpen = true;
		createServerError = null;
		createServerMessage = null;
		createServerAvatarFile = null;
	};

	const closeCreateServerModal = () => {
		createServerModalOpen = false;
		createServerAvatarFile = null;
	};

	const openJoinServerModal = async () => {
		ignoreModalOutsideCloseUntil = Date.now() + 200;
		joinServerModalOpen = true;
		publicServersError = null;
		joinServerMessage = null;
		await loadPublicServers();
	};

	const closeJoinServerModal = () => {
		for (const item of publicServers) {
			if (item.avatarUrl?.startsWith('blob:')) {
				URL.revokeObjectURL(item.avatarUrl);
			}
		}

		joinServerModalOpen = false;
		joinServerConfirmModalOpen = false;
		joinServerTarget = null;
		publicServersError = null;
		joinServerMessage = null;
		publicServers = [];
	};

	const openJoinServerConfirmModal = (server: ServerEntry) => {
		joinServerTarget = server;
		joinServerConfirmModalOpen = true;
	};

	const handleJoinServer = async () => {
		const token = $session.accessToken;
		const server = joinServerTarget;
		if (!token || !server || joinServerLoading) {
			return;
		}

		joinServerLoading = true;
		publicServersError = null;
		joinServerMessage = null;

		try {
			await apiClient.joinPublicServer(token, server.id);
			if (channelUnreadByServer[server.id]) {
				const nextUnread = { ...channelUnreadByServer };
				delete nextUnread[server.id];
				channelUnreadByServer = nextUnread;
			}
			await refreshServersData();
			await loadPublicServers();
			joinServerMessage = `Joined ${server.name}.`;
			scheduleTransientClear('joinServerMessage', () => {
				joinServerMessage = null;
			});
			joinServerConfirmModalOpen = false;
			joinServerTarget = null;
		} catch (error) {
			publicServersError = error instanceof Error ? error.message : 'Failed to join server.';
			scheduleTransientClear('publicServersError', () => {
				publicServersError = null;
			});
		} finally {
			joinServerLoading = false;
		}
	};

	const handlePublicServersSearchSubmit = async () => {
		await loadPublicServers();
	};

	const handlePublicServersSortChange = async (event: Event) => {
		const nextSort = (event.currentTarget as HTMLSelectElement).value as PublicServerSort;
		publicServersSort = nextSort;
		await loadPublicServers();
	};

	const handleCreateServerAvatarPicked = (event: Event) => {
		const input = event.currentTarget as HTMLInputElement;
		createServerAvatarFile = input.files?.[0] ?? null;
	};

	const handleCreateServer = async () => {
		const token = $session.accessToken;
		if (!token || createServerLoading) {
			return;
		}

		createServerError = null;
		createServerMessage = null;

		const name = createServerName.trim();
		const description = createServerDescription.trim();

		if (!name) {
			createServerError = 'Server name is required.';
			scheduleTransientClear('createServerError', () => {
				createServerError = null;
			});
			return;
		}

		if (name.length > 64) {
			createServerError = 'Server name must be at most 64 characters.';
			scheduleTransientClear('createServerError', () => {
				createServerError = null;
			});
			return;
		}

		createServerLoading = true;
		try {
			const createdServer = await apiClient.createServer(token, name, description, createServerIsPublic);

			let uploadWarning: string | null = null;
			if (createServerAvatarFile) {
				try {
					await apiClient.uploadServerAvatar(token, createdServer.id, createServerAvatarFile);
				} catch (error) {
					uploadWarning = 'You can upload it later from server settings.';
					if (error instanceof Error && error.message && error.message !== 'Load failed') {
						uploadWarning = `${uploadWarning} (${error.message})`;
					}
				}
			}

			await refreshServersData();
			createServerMessage = uploadWarning
				? `Server created, but image upload failed. ${uploadWarning}`
				: 'Server created successfully.';
			scheduleTransientClear('createServerMessage', () => {
				createServerMessage = null;
			});
			createServerName = '';
			createServerDescription = '';
			createServerIsPublic = true;
			createServerAvatarFile = null;
			createServerModalOpen = false;
		} catch (error) {
			createServerError = error instanceof Error ? error.message : 'Failed to create server.';
			scheduleTransientClear('createServerError', () => {
				createServerError = null;
			});
		} finally {
			createServerLoading = false;
		}
	};

	const openPasswordModal = () => {
		passwordModalOpen = true;
		passwordChangeError = null;
		passwordChangeMessage = null;
		currentPasswordInput = '';
		newPasswordInput = '';
		newPasswordConfirmInput = '';
	};

	const openCreateChannelModal = () => {
		serverSettingsMenuOpen = false;
		createChannelModalOpen = true;
		createChannelError = null;
		createChannelName = '';
		createChannelEmoji = '💬';
		createChannelEmojiPickerOpen = false;
	};

	const handleOpenServerAvatarUpload = () => {
		serverSettingsMenuOpen = false;
		serverAvatarUploadError = null;
		serverAvatarUploadMessage = null;
		serverAvatarInputEl?.click();
	};

	const handleServerAvatarPicked = async (event: Event) => {
		const token = $session.accessToken;
		const serverId = $selectedServer?.id;
		if (!token || !serverId) {
			return;
		}

		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) {
			return;
		}

		serverAvatarUploadError = null;
		serverAvatarUploadMessage = null;
		clearTransientTimeout('serverAvatarUploadMessage');
		clearTransientTimeout('serverAvatarUploadError');

		try {
			await apiClient.uploadServerAvatar(token, serverId, file);
			await refreshServersData();
			serverAvatarUploadMessage = 'Server image uploaded.';
			scheduleTransientClear('serverAvatarUploadMessage', () => {
				serverAvatarUploadMessage = null;
			});
		} catch (error) {
			serverAvatarUploadError = error instanceof Error ? error.message : 'Failed to upload server image.';
			scheduleTransientClear('serverAvatarUploadError', () => {
				serverAvatarUploadError = null;
			});
		} finally {
			input.value = '';
		}
	};

	const handleCreateChannelEmojiPicked = (event: Event) => {
		const detail = (event as CustomEvent<{ unicode?: string }>).detail;
		if (detail?.unicode) {
			createChannelEmoji = detail.unicode;
			createChannelEmojiPickerOpen = false;
		}
	};

	const handleChatEmojiPicked = (event: Event) => {
		const detail = (event as CustomEvent<{ unicode?: string }>).detail;
		if (detail?.unicode) {
			chatInput = `${chatInput}${detail.unicode}`;
			chatEmojiPickerOpen = false;
		}
	};

	const handleCreateChannel = async () => {
		const token = $session.accessToken;
		const serverId = $selectedServer?.id;
		if (!token || !serverId || createChannelLoading) {
			return;
		}

		createChannelError = null;
		const name = createChannelName.trim();

		const emoji = createChannelEmoji.trim();

		if (!name) {
			createChannelError = 'Channel name is required.';
			return;
		}

		if (name.length > 64) {
			createChannelError = 'Channel name must be at most 64 characters.';
			return;
		}

		if (!emoji) {
			createChannelError = 'Emoji is required.';
			return;
		}

		createChannelLoading = true;
		try {
			await apiClient.createChannel(token, serverId, name, emoji);
			await loadServerChannels(serverId);
			createChannelName = '';
			createChannelEmoji = '💬';
			createChannelEmojiPickerOpen = false;
			createChannelModalOpen = false;
		} catch (error) {
			createChannelError = error instanceof Error ? error.message : 'Failed to create channel.';
		} finally {
			createChannelLoading = false;
		}
	};

	const openUpdateChannelNameModal = () => {
		channelSettingsMenuOpen = false;
		if (!selectedChannel) {
			return;
		}

		updateChannelNameValue = selectedChannel.name;
		updateChannelNameError = null;
		updateChannelNameModalOpen = true;
	};

	const openUpdateChannelEmojiModal = () => {
		channelSettingsMenuOpen = false;
		if (!selectedChannel) {
			return;
		}

		updateChannelEmojiValue = selectedChannel.emoji;
		updateChannelEmojiError = null;
		updateChannelEmojiPickerOpen = false;
		updateChannelEmojiModalOpen = true;
	};

	const handleUpdateChannelEmojiPicked = (event: Event) => {
		const detail = (event as CustomEvent<{ unicode?: string }>).detail;
		if (detail?.unicode) {
			updateChannelEmojiValue = detail.unicode;
			updateChannelEmojiPickerOpen = false;
		}
	};

	const syncSelectedChannelFromCache = (serverId: string) => {
		if (!selectedChannel) {
			return;
		}

		const next = (serverChannels[serverId] ?? []).find((item) => item.id === selectedChannel?.id) ?? null;
		if (next) {
			selectedChannel = next;
			return;
		}

		selectedChannel = null;
		if (activeChat === 'server') {
			activeChat = 'server-channels';
			chatMessages = [];
			chatInput = '';
		}
	};

	const handleUpdateChannelName = async () => {
		const token = $session.accessToken;
		const serverId = $selectedServer?.id;
		const channelId = selectedChannel?.id;

		if (!token || !serverId || !channelId || updateChannelNameLoading) {
			return;
		}

		updateChannelNameError = null;
		const name = updateChannelNameValue.trim();
		if (!name) {
			updateChannelNameError = 'Channel name is required.';
			return;
		}

		if (name.length > 64) {
			updateChannelNameError = 'Channel name must be at most 64 characters.';
			return;
		}

		updateChannelNameLoading = true;
		try {
			await apiClient.updateChannel(token, channelId, name);
			await loadServerChannels(serverId);
			syncSelectedChannelFromCache(serverId);
			updateChannelNameModalOpen = false;
		} catch (error) {
			updateChannelNameError = error instanceof Error ? error.message : 'Failed to update channel name.';
		} finally {
			updateChannelNameLoading = false;
		}
	};

	const handleUpdateChannelEmoji = async () => {
		const token = $session.accessToken;
		const serverId = $selectedServer?.id;
		const channelId = selectedChannel?.id;

		if (!token || !serverId || !channelId || updateChannelEmojiLoading) {
			return;
		}

		updateChannelEmojiError = null;
		const emoji = updateChannelEmojiValue.trim();
		if (!emoji) {
			updateChannelEmojiError = 'Emoji is required.';
			return;
		}

		updateChannelEmojiLoading = true;
		try {
			await apiClient.updateChannel(token, channelId, undefined, emoji);
			await loadServerChannels(serverId);
			syncSelectedChannelFromCache(serverId);
			updateChannelEmojiModalOpen = false;
			updateChannelEmojiPickerOpen = false;
		} catch (error) {
			updateChannelEmojiError = error instanceof Error ? error.message : 'Failed to update channel logo.';
		} finally {
			updateChannelEmojiLoading = false;
		}
	};

	const openDeleteChannelModal = () => {
		channelSettingsMenuOpen = false;
		deleteChannelError = null;
		deleteChannelConfirmText = '';
		deleteChannelModalOpen = true;
	};

	const handleDeleteChannel = async () => {
		const token = $session.accessToken;
		const serverId = $selectedServer?.id;
		const channelId = selectedChannel?.id;

		if (!token || !serverId || !channelId || deleteChannelLoading) {
			return;
		}

		deleteChannelError = null;
		if (deleteChannelConfirmText !== 'DELETE') {
			deleteChannelError = 'Confirm deletion by typing "DELETE".';
			return;
		}

		deleteChannelLoading = true;
		try {
			await apiClient.deleteChannel(token, channelId);
			await loadServerChannels(serverId);
			syncSelectedChannelFromCache(serverId);
			deleteChannelModalOpen = false;
		} catch (error) {
			deleteChannelError = error instanceof Error ? error.message : 'Failed to delete channel.';
		} finally {
			deleteChannelLoading = false;
		}
	};

	const handleDeleteSelectedFriend = async () => {
		const token = $session.accessToken;
		const friendId = $selectedFriend?.id;
		if (!token || !friendId) {
			return;
		}

		const confirmed = window.confirm('Delete this friend and the conversation? This cannot be undone.');
		if (!confirmed) {
			return;
		}

		chatError = null;
		friendSettingsMenuOpen = false;
		try {
			await apiClient.deleteFriend(token, friendId);
			await refreshFriendsData();
			closeChat();
		} catch (error) {
			chatError = error instanceof Error ? error.message : 'Failed to delete friend.';
		}
	};

	const openUpdateServerNameModal = () => {
		serverSettingsMenuOpen = false;
		if ($selectedServer) {
			updateServerNameValue = $selectedServer.name;
			updateServerNameModalOpen = true;
			updateServerNameError = null;
		}
	};

	const handleUpdateServerName = async () => {
		const token = $session.accessToken;
		const serverId = $selectedServer?.id;
		if (!token || !serverId || updateServerNameLoading) {
			return;
		}

		updateServerNameError = null;
		const name = updateServerNameValue.trim();

		if (!name) {
			updateServerNameError = 'Server name is required.';
			return;
		}

		if (name.length > 64) {
			updateServerNameError = 'Server name must be at most 64 characters.';
			return;
		}

		updateServerNameLoading = true;
		try {
			await apiClient.updateServer(token, serverId, name);
			await refreshServersData();
			updateServerNameModalOpen = false;
		} catch (error) {
			updateServerNameError = error instanceof Error ? error.message : 'Failed to update server name.';
		} finally {
			updateServerNameLoading = false;
		}
	};

	const openUpdateServerDescriptionModal = () => {
		serverSettingsMenuOpen = false;
		if ($selectedServer) {
			updateServerDescriptionValue = $selectedServer.description ?? '';
			updateServerDescriptionModalOpen = true;
			updateServerDescriptionError = null;
		}
	};

	const handleUpdateServerDescription = async () => {
		const token = $session.accessToken;
		const serverId = $selectedServer?.id;
		if (!token || !serverId || updateServerDescriptionLoading) {
			return;
		}

		updateServerDescriptionError = null;

		updateServerDescriptionLoading = true;
		try {
			await apiClient.updateServer(token, serverId, undefined, updateServerDescriptionValue.trim());
			await refreshServersData();
			updateServerDescriptionModalOpen = false;
		} catch (error) {
			updateServerDescriptionError = error instanceof Error ? error.message : 'Failed to update server description.';
		} finally {
			updateServerDescriptionLoading = false;
		}
	};

	const openUpdateServerVisibilityModal = () => {
		serverSettingsMenuOpen = false;
		if ($selectedServer) {
			// Guess if public based on some logic or fetch from server data if available
			// For now we'll default to false until we know the actual value
			updateServerVisibilityValue = false;
			updateServerVisibilityModalOpen = true;
			updateServerVisibilityError = null;
		}
	};

	const handleUpdateServerVisibility = async () => {
		const token = $session.accessToken;
		const serverId = $selectedServer?.id;
		if (!token || !serverId || updateServerVisibilityLoading) {
			return;
		}

		updateServerVisibilityError = null;

		updateServerVisibilityLoading = true;
		try {
			await apiClient.updateServer(token, serverId, undefined, undefined, updateServerVisibilityValue);
			await refreshServersData();
			updateServerVisibilityModalOpen = false;
		} catch (error) {
			updateServerVisibilityError = error instanceof Error ? error.message : 'Failed to update server visibility.';
		} finally {
			updateServerVisibilityLoading = false;
		}
	};

	const openDeleteServerModal = () => {
		serverSettingsMenuOpen = false;
		deleteServerModalOpen = true;
		deleteServerError = null;
		deleteServerConfirmText = '';
	};

	const handleDeleteServer = async () => {
		const token = $session.accessToken;
		const serverId = $selectedServer?.id;
		if (!token || !serverId || deleteServerLoading) {
			return;
		}

		deleteServerError = null;

		if (deleteServerConfirmText !== 'DELETE') {
			deleteServerError = 'Confirm deletion by typing "DELETE".';
			return;
		}

		deleteServerLoading = true;
		try {
			await apiClient.deleteServer(token, serverId);
			await refreshServersData();
			backToServerList();
			deleteServerModalOpen = false;
		} catch (error) {
			deleteServerError = error instanceof Error ? error.message : 'Failed to delete server.';
		} finally {
			deleteServerLoading = false;
		}
	};

	const handlePasswordChange = async () => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		passwordChangeError = null;
		passwordChangeMessage = null;

		if (!currentPasswordInput || !newPasswordInput || !newPasswordConfirmInput) {
			passwordChangeError = 'Fill in all fields.';
			return;
		}

		if (newPasswordInput.length < 6) {
			passwordChangeError = 'The new password must be at least 6 characters.';
			return;
		}

		if (newPasswordInput !== newPasswordConfirmInput) {
			passwordChangeError = 'New password and confirmation do not match.';
			return;
		}

		if (currentPasswordInput === newPasswordInput) {
			passwordChangeError = 'New password must be different from the current one.';
			return;
		}

		passwordChangeLoading = true;
		try {
			await apiClient.changeMyPassword(token, currentPasswordInput, newPasswordInput);
			passwordChangeMessage = 'Password updated successfully.';
			scheduleTransientClear('passwordChangeMessage', () => {
				passwordChangeMessage = null;
			});
			currentPasswordInput = '';
			newPasswordInput = '';
			newPasswordConfirmInput = '';
		} catch (error) {
			if (error instanceof Error && error.message === 'unauthorized') {
				passwordChangeError = 'Current password is incorrect.';
			} else {
				passwordChangeError = error instanceof Error ? error.message : 'Failed to update password.';
			}
			scheduleTransientClear('passwordChangeError', () => {
				passwordChangeError = null;
			});
		} finally {
			passwordChangeLoading = false;
		}
	};

	const handleOpenAvatarUpload = () => {
		settingsMenuOpen = false;
		avatarInputEl?.click();
	};

	const handleAvatarPicked = async (event: Event) => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) {
			return;
		}

		profileUploadError = null;
		profileUploadMessage = null;
		serverAvatarUploadError = null;
		serverAvatarUploadMessage = null;
		clearTransientTimeout('profileUploadMessage');
		clearTransientTimeout('profileUploadError');

		try {
			await apiClient.uploadMyAvatar(token, file);
			if ($session.userId) {
				markAvatarDirty($session.userId);
			}
			await Promise.all([refreshFriends(token), refreshPendingRequests(token)]);
			await refreshCurrentUserAvatar();
			profileUploadMessage = 'Profile image uploaded.';
			scheduleTransientClear('profileUploadMessage', () => {
				profileUploadMessage = null;
			});
		} catch (error) {
			profileUploadError = error instanceof Error ? error.message : 'Failed to upload profile image.';
			scheduleTransientClear('profileUploadError', () => {
				profileUploadError = null;
			});
		} finally {
			input.value = '';
		}
	};

	const handleLogout = () => {
		for (const item of publicServers) {
			if (item.avatarUrl?.startsWith('blob:')) {
				URL.revokeObjectURL(item.avatarUrl);
			}
		}

		setCurrentUserAvatarUrl(null);
		if (chatReloadTimeout) {
			clearTimeout(chatReloadTimeout);
			chatReloadTimeout = null;
		}
		for (const timeout of transientTimeouts.values()) {
			clearTimeout(timeout);
		}
		transientTimeouts.clear();

		settingsMenuOpen = false;
		activeChat = 'none';
		selectedChannel = null;
		activeDmThreadId = null;
		chatMessages = [];
		chatInput = '';
		chatError = null;
		chatEmojiPickerOpen = false;
		dmThreadByFriendId = {};
		friendByDmThreadId = {};
		onlineUserIdsSnapshot = [];
		channelSettingsMenuOpen = false;
		addFriendModalOpen = false;
		pendingModalOpen = false;
		serverInviteModalOpen = false;
		inviteUserModalOpen = false;
		createServerModalOpen = false;
		joinServerModalOpen = false;
		joinServerConfirmModalOpen = false;
		createChannelModalOpen = false;
		serverSettingsMenuOpen = false;
		updateServerNameModalOpen = false;
		updateServerDescriptionModalOpen = false;
		updateServerVisibilityModalOpen = false;
		deleteServerModalOpen = false;
		friendSettingsMenuOpen = false;
		updateChannelNameModalOpen = false;
		updateChannelEmojiModalOpen = false;
		deleteChannelModalOpen = false;
		createChannelEmojiPickerOpen = false;
		updateChannelEmojiPickerOpen = false;
		addFriendUsername = '';
		addFriendError = null;
		addFriendMessage = null;
		createServerName = '';
		createServerDescription = '';
		createServerIsPublic = true;
		createServerAvatarFile = null;
		createServerError = null;
		createServerMessage = null;
		publicServers = [];
		publicServersLoading = false;
		publicServersError = null;
		publicServersSearch = '';
		publicServersSort = 'newest';
		joinServerTarget = null;
		joinServerLoading = false;
		joinServerMessage = null;
		channelUnreadByServer = {};
		leaveServerLoading = false;
		leaveServerError = null;
		clearMessageAuthorAvatarCache();
		createChannelName = '';
		createChannelEmoji = '💬';
		createChannelError = null;
		updateChannelNameValue = '';
		updateChannelNameError = null;
		updateChannelEmojiValue = '💬';
		updateChannelEmojiError = null;
		deleteChannelError = null;
		deleteChannelConfirmText = '';
		updateServerNameValue = '';
		updateServerNameError = null;
		updateServerDescriptionValue = '';
		updateServerDescriptionError = null;
		updateServerVisibilityValue = false;
		updateServerVisibilityError = null;
		deleteServerError = null;
		deleteServerConfirmText = '';
		pendingActionError = null;
		serverInviteActionError = null;
		serverInviteActionInProgressId = null;
		inviteUserSearch = '';
		inviteUserError = null;
		inviteUserMessage = null;
		inviteUserInProgressId = null;
		clearServerInviteAvatars();
		serverInvites = [];
		clearInvitableFriendsAvatars();
		invitableFriends = [];
		profileUploadError = null;
		profileUploadMessage = null;
		passwordModalOpen = false;
		passwordChangeError = null;
		passwordChangeMessage = null;
		currentPasswordInput = '';
		newPasswordInput = '';
		newPasswordConfirmInput = '';
		lastLoadedToken = null;
		logout();
	};

	onMount(() => {
		void import('emoji-picker-element');

		const handler = async (event: Event) => {
			const custom = event as CustomEvent<{ kind: string; payload: unknown }>;
			const kind = custom.detail?.kind;
			const payload = (custom.detail?.payload ?? {}) as {
				id?: string;
				thread_id?: string;
				channel_id?: string;
				server_id?: string;
				user_id?: string;
				online?: boolean;
				online_user_ids?: string[];
			};

			if (kind === 'presence.snapshot') {
				onlineUserIdsSnapshot = payload.online_user_ids ?? [];
				applyPresenceSnapshot(onlineUserIdsSnapshot);
				return;
			}

			if (kind === 'presence.updated' && payload.user_id && typeof payload.online === 'boolean') {
				if (payload.online) {
					onlineUserIdsSnapshot = Array.from(new Set([...onlineUserIdsSnapshot, payload.user_id]));
				} else {
					onlineUserIdsSnapshot = onlineUserIdsSnapshot.filter((id) => id !== payload.user_id);
				}
				setFriendOnline(payload.user_id, payload.online);
				return;
			}

			if ((kind === 'user.avatar.updated' || kind === 'user.updated') && payload.user_id && $session.accessToken) {
				markAvatarDirty(payload.user_id);
				await Promise.all([
					refreshFriends($session.accessToken),
					refreshPendingRequests($session.accessToken),
					payload.user_id === $session.userId ? refreshCurrentUserAvatar() : Promise.resolve()
				]);
				return;
			}

			if (
				kind === 'friend.request.created' ||
				kind === 'friend.request.accepted' ||
				kind === 'friend.request.rejected' ||
				kind === 'friend.deleted'
			) {
				await Promise.all([refreshFriendsData(), refreshServerInvites()]);
				if (inviteUserModalOpen && $selectedServer?.id) {
					await loadInvitableFriends();
				}
				return;
			}

			if (
				kind === 'server.invite.created' ||
				kind === 'server.invite.accepted' ||
				kind === 'server.invite.rejected'
			) {
				await refreshServerInvites();
				if (inviteUserModalOpen && $selectedServer?.id) {
					await loadInvitableFriends();
				}
				return;
			}

			if (
				kind === 'server.created' ||
				kind === 'server.updated' ||
				kind === 'server.deleted' ||
				kind === 'server.joined' ||
				kind === 'server.left'
			) {
				if (kind === 'server.joined' && payload.user_id === $session.userId && payload.server_id) {
					if (channelUnreadByServer[payload.server_id]) {
						const nextUnread = { ...channelUnreadByServer };
						delete nextUnread[payload.server_id];
						channelUnreadByServer = nextUnread;
					}
				}

				await refreshServersData();
				if (joinServerModalOpen) {
					await loadPublicServers();
				}
				return;
			}

			if (kind === 'channel.created' || kind === 'channel.updated' || kind === 'channel.deleted') {
				if (kind === 'channel.deleted' && payload.server_id && payload.channel_id) {
					clearChannelUnread(payload.server_id, payload.channel_id);
				}

				const serverId = payload.server_id ?? $selectedServer?.id;
				if (serverId) {
					await loadServerChannels(serverId);
					syncSelectedChannelFromCache(serverId);
				}
				return;
			}

			if (kind === 'dm.message.created' || kind === 'dm.message.updated' || kind === 'dm.message.deleted') {
				if (kind === 'dm.message.created' && payload.id && !rememberDmCreatedId(payload.id)) {
					return;
				}

				if (payload.thread_id) {
					await updateFriendPreviewFromThread(payload.thread_id);

					if (
						kind === 'dm.message.created' &&
						payload.user_id &&
						payload.user_id !== $session.userId
					) {
						const friendId = await resolveFriendIdByThread(payload.thread_id);
						if (friendId) {
							const currentlyOpenWithFriend =
								activeChat === 'friend' &&
								activeDmThreadId === payload.thread_id;

							if (currentlyOpenWithFriend) {
								clearFriendUnread(friendId);
							} else {
								incrementFriendUnread(friendId);
							}
						}
					}
				}

				if (
					activeChat === 'friend' &&
					activeDmThreadId &&
					payload.thread_id === activeDmThreadId
				) {
					const ownCreatedInActiveThread =
						kind === 'dm.message.created' && payload.user_id === $session.userId;
					if (!ownCreatedInActiveThread) {
						scheduleActiveMessagesReload({ silent: true, forceBottom: true });
					}
				}
				return;
			}

			if (kind === 'channel.message.created' || kind === 'channel.message.updated' || kind === 'channel.message.deleted') {
				const isActiveChannel =
					activeChat === 'server' &&
					selectedChannel?.id &&
					payload.channel_id === selectedChannel.id;

				if (
					kind === 'channel.message.created' &&
					payload.user_id &&
					payload.user_id !== $session.userId &&
					payload.server_id &&
					payload.channel_id &&
					!isActiveChannel
				) {
					incrementChannelUnread(payload.server_id, payload.channel_id);
				}

				if (isActiveChannel) {
					const ownCreatedInActiveChannel =
						kind === 'channel.message.created' && payload.user_id === $session.userId;
					if (!ownCreatedInActiveChannel) {
						scheduleActiveMessagesReload({ silent: true, forceBottom: true });
					}
				}
			}
		};

		const handleOutsideClick = (event: MouseEvent) => {
			const target = event.target as Node | null;
			const targetElement = event.target instanceof Element ? event.target : null;
			const ignoreModalOutsideClose = Date.now() < ignoreModalOutsideCloseUntil;

			if (
				messageActionsOpenForId &&
				targetElement &&
				!targetElement.closest('[data-message-actions]')
			) {
				messageActionsOpenForId = null;
			}

			if (settingsMenuOpen && settingsMenuEl && target && !settingsMenuEl.contains(target)) {
				settingsMenuOpen = false;
			}

			if (serverSettingsMenuOpen && serverSettingsMenuEl && target && !serverSettingsMenuEl.contains(target)) {
				serverSettingsMenuOpen = false;
			}

			if (friendSettingsMenuOpen && friendSettingsMenuEl && target && !friendSettingsMenuEl.contains(target)) {
				friendSettingsMenuOpen = false;
			}

			if (channelSettingsMenuOpen && channelSettingsMenuEl && target && !channelSettingsMenuEl.contains(target)) {
				channelSettingsMenuOpen = false;
			}

			if (
				chatEmojiPickerOpen &&
				target &&
				(!chatEmojiPickerEl || !chatEmojiPickerEl.contains(target)) &&
				(!chatEmojiToggleButtonEl || !chatEmojiToggleButtonEl.contains(target))
			) {
				chatEmojiPickerOpen = false;
			}

			if (
				!ignoreModalOutsideClose &&
				createServerModalOpen &&
				createServerModalEl &&
				target &&
				!createServerModalEl.contains(target)
			) {
				closeCreateServerModal();
			}

			if (
				!ignoreModalOutsideClose &&
				joinServerModalOpen &&
				joinServerModalEl &&
				target &&
				!joinServerModalEl.contains(target)
			) {
				closeJoinServerModal();
			}
		};

		window.addEventListener('poseidon:ws-event', handler as EventListener);
		window.addEventListener('click', handleOutsideClick);
		return () => {
			window.removeEventListener('poseidon:ws-event', handler as EventListener);
			window.removeEventListener('click', handleOutsideClick);
			if (chatReloadTimeout) {
				clearTimeout(chatReloadTimeout);
				chatReloadTimeout = null;
			}
			for (const timeout of transientTimeouts.values()) {
				clearTimeout(timeout);
			}
			transientTimeouts.clear();
		};
	});

	$effect(() => {
		const token = $session.accessToken;
		if (!token) {
			lastLoadedToken = null;
			setCurrentUserAvatarUrl(null);
			clearMessageAuthorAvatarCache();
			clearServerInviteAvatars();
			serverInvites = [];
			clearInvitableFriendsAvatars();
			invitableFriends = [];
			return;
		}

		if (lastLoadedToken !== token) {
			lastLoadedToken = token;
			void Promise.all([
				refreshFriendsData(),
				refreshServersData(),
				refreshServerInvites(),
				refreshCurrentUserAvatar()
			]);
		}
	});
</script>

{#if !$session.accessToken}
	<section class="min-h-screen w-full bg-slate-900 text-slate-100 flex flex-col">
		<header class="py-4 text-center">
			<h1 class="text-5xl font-bold tracking-tight">Poseidon</h1>
		</header>

		<div class="flex-1 overflow-auto px-4 py-6 sm:px-8 sm:py-8 md:px-12 md:py-10">
			<div class="w-full max-w-xl mx-auto rounded-xl border border-slate-700/70 bg-slate-800/50 p-4 sm:p-6 md:p-8">
				<form
					class="space-y-6"
					onsubmit={(e) => {
						e.preventDefault();
						void submitAuth();
					}}
				>
					<h2 class="text-xl font-semibold">{authMode === 'login' ? 'Sign in' : 'Sign up'}</h2>

					<div class="space-y-3">
						{#if authMode === 'login'}
							<label class="block w-full">
								<span class="label-text block mb-2">Username or email</span>
								<input class="input input-bordered w-full" bind:value={loginIdentifier} autocomplete="username" />
							</label>
						{:else}
							<label class="block w-full">
								<span class="label-text block mb-2">Username</span>
								<input class="input input-bordered w-full" bind:value={username} autocomplete="username" />
							</label>
							<label class="block w-full">
								<span class="label-text block mb-2">Email</span>
								<input class="input input-bordered w-full" type="email" bind:value={email} autocomplete="email" />
							</label>
						{/if}

						<label class="block w-full">
							<span class="label-text block mb-2">Password</span>
							<input class="input input-bordered w-full" type="password" bind:value={password} autocomplete="current-password" />
						</label>

						{#if authMode === 'register'}
							<label class="block w-full">
								<span class="label-text block mb-2">Confirm password</span>
								<input class="input input-bordered w-full" type="password" bind:value={confirmPassword} autocomplete="new-password" />
							</label>
						{/if}
					</div>

					<div class="space-y-3 pt-0">
						{#if authError}
							<div class="alert alert-error py-2 text-sm">
								<span>{authError}</span>
							</div>
						{/if}

						<button class="btn btn-primary w-full" type="submit">
							{authMode === 'login' ? 'Sign in' : 'Sign up'}
						</button>

						<button class="btn btn-ghost btn-sm w-full" type="button" onclick={switchAuthMode}>
							{authMode === 'login' ? "Don't have an account? Sign up" : 'Already have an account? Sign in'}
						</button>
					</div>
				</form>
			</div>
		</div>
	</section>
{:else}
	<main class="h-screen w-full overflow-hidden">
		<div class="h-full w-full overflow-hidden bg-slate-900 flex flex-col">
			<div class="flex-1 min-h-0">
			{#if activeChat === 'none'}
				<section class="h-full p-3 overflow-auto">
					<div class="h-12 border border-slate-700/60 rounded-md px-3 mb-3 flex items-center justify-between bg-slate-900/70">
						<div class="flex items-center gap-2 min-w-0">
							<div class="avatar">
								<div class="w-8 rounded-full bg-slate-700 text-slate-100 flex items-center justify-center overflow-hidden">
									{#if currentUserAvatarUrl}
										<img src={currentUserAvatarUrl} alt="Your profile" class="h-full w-full object-cover" />
									{:else}
										<span class="text-xs font-semibold">{$session.username?.slice(0, 1).toUpperCase() ?? 'U'}</span>
									{/if}
								</div>
							</div>
							<p class="text-sm text-slate-300 truncate">{$session.username ?? 'User'}</p>
						</div>
						<div class="relative" bind:this={settingsMenuEl}>
							<button
								type="button"
								class="btn btn-sm btn-circle btn-ghost text-xl leading-none"
								onclick={() => {
									settingsMenuOpen = !settingsMenuOpen;
								}}
							>
								⚙
							</button>

							{#if settingsMenuOpen}
								<ul class="menu absolute right-0 z-[60] mt-2 w-56 rounded-box bg-slate-800 border border-slate-700 p-2 shadow">
									<li><button type="button" onclick={handleOpenAvatarUpload}>Upload profile image</button></li>
									<li>
										<button
											type="button"
											onclick={() => {
												settingsMenuOpen = false;
												openPasswordModal();
											}}
										>
											Change password
										</button>
									</li>
									<li><button type="button" onclick={handleLogout}>Sign out</button></li>
								</ul>
							{/if}
						</div>
					</div>

					{#if profileUploadError}
						<div class="alert alert-error py-2 text-sm mb-3"><span>{profileUploadError}</span></div>
					{/if}
					{#if profileUploadMessage}
						<div class="alert alert-success py-2 text-sm mb-3"><span>{profileUploadMessage}</span></div>
					{/if}

					<div class="tabs tabs-boxed rounded-lg bg-slate-800/60 border border-slate-700/70 mb-3 w-full p-1">
						<button
							class={`tab rounded-md flex-1 text-base transition-colors ${$selectedTab === 'friends' ? 'bg-primary/40 text-primary-content font-extrabold border border-primary/50 shadow' : 'text-slate-300/90 font-semibold hover:bg-slate-700/70'}`}
							onclick={() => { $selectedTab = 'friends'; }}
						>
							Friends
						</button>
						<button
							class={`tab rounded-md flex-1 text-base transition-colors ${$selectedTab === 'servers' ? 'bg-primary/40 text-primary-content font-extrabold border border-primary/50 shadow' : 'text-slate-300/90 font-semibold hover:bg-slate-700/70'}`}
							onclick={() => { $selectedTab = 'servers'; }}
						>
							Servers
						</button>
					</div>

					{#if $selectedTab === 'friends'}
						<div class="mb-3">
							<button class="btn btn-primary btn-sm w-full" type="button" onclick={openAddFriendModal}>Add friend</button>
						</div>

						{#if $friendPending.length > 0}
							<ul class="menu rounded-box bg-slate-800/40 mb-3 w-full">
								<li>
									<button type="button" onclick={openPendingModal} class="flex items-center justify-between">
										<span class="font-medium">Friend requests</span>
										<span class="badge badge-primary">{$friendPending.length}</span>
									</button>
								</li>
							</ul>
						{/if}

						<p class="text-xs uppercase tracking-wider text-emerald-300 mb-1">Online</p>
						<ul class="menu rounded-box bg-slate-800/40 mb-3 w-full">
							{#each $friendOnline as item}
								<li>
									<button type="button" onclick={() => openFriendChat(item)}>
										<div class="avatar mr-3">
											<div class="w-9 rounded-full bg-slate-700 text-slate-100 flex items-center justify-center overflow-hidden">
												{#if item.avatarUrl}
													<img src={item.avatarUrl} alt={`${item.username} avatar`} class="h-full w-full object-cover" />
												{:else}
													<span class="text-xs font-semibold">{item.username.slice(0, 1).toUpperCase()}</span>
												{/if}
											</div>
										</div>
										<div class="flex-1 text-left">
											<p class="font-medium">{item.username}</p>
											<p class="text-xs text-slate-400 truncate">{item.lastMessage}</p>
										</div>
										{#if item.unread > 0}
											<div class="badge badge-secondary">{item.unread}</div>
										{/if}
									</button>
								</li>
							{/each}
						</ul>

						<p class="text-xs uppercase tracking-wider text-slate-400 mb-1">Offline</p>
						<ul class="menu rounded-box bg-slate-800/40 w-full">
							{#each $friendOffline as item}
								<li>
									<button type="button" onclick={() => openFriendChat(item)}>
										<div class="avatar mr-3">
											<div class="w-9 rounded-full bg-slate-700 text-slate-100 flex items-center justify-center overflow-hidden">
												{#if item.avatarUrl}
													<img src={item.avatarUrl} alt={`${item.username} avatar`} class="h-full w-full object-cover" />
												{:else}
													<span class="text-xs font-semibold">{item.username.slice(0, 1).toUpperCase()}</span>
												{/if}
											</div>
										</div>
										<div class="flex-1 text-left">
											<p class="font-medium">{item.username}</p>
											<p class="text-xs text-slate-400 truncate">{item.lastMessage}</p>
										</div>
										{#if item.unread > 0}
											<div class="badge badge-secondary">{item.unread}</div>
										{/if}
									</button>
								</li>
							{/each}
						</ul>
					{:else}
						<div class="mb-3 flex gap-1">
							<button class="btn btn-primary btn-sm flex-1" type="button" onclick={openCreateServerModal}>Create server</button>
							<button class="btn btn-primary btn-sm flex-1" type="button" onclick={() => void openJoinServerModal()}>Join a server</button>
						</div>

						{#if serverInvites.length > 0}
							<ul class="menu rounded-box bg-slate-800/40 mb-3 w-full">
								<li>
									<button type="button" onclick={openServerInviteModal} class="flex items-center justify-between">
										<span class="font-medium">Server invites</span>
										<span class="badge badge-primary">{serverInvites.length}</span>
									</button>
								</li>
							</ul>
						{/if}

						{#if createServerError}
							<div class="alert alert-error py-2 text-sm mb-3"><span>{createServerError}</span></div>
						{/if}
						{#if createServerMessage}
							<div class="alert alert-success py-2 text-sm mb-3"><span>{createServerMessage}</span></div>
						{/if}

						<ul class="menu rounded-box bg-slate-800/40 w-full">
							{#each $servers as item}
								<li>
									<button type="button" onclick={() => openServerChannels(item)}>
										<div class="avatar mr-3">
											<div class="w-9 rounded-full bg-slate-700 text-slate-100 flex items-center justify-center overflow-hidden">
												{#if item.avatarUrl}
													<img src={item.avatarUrl} alt={`${item.name} avatar`} class="h-full w-full object-cover" />
												{:else}
													<span class="text-xs font-semibold">{item.name.slice(0, 1).toUpperCase()}</span>
												{/if}
											</div>
										</div>
										<div class="flex-1">
											<p class="font-medium">{item.name}</p>
											<p class="text-xs text-slate-400 truncate">{item.description}</p>
										</div>
										{#if item.hasUnread}
											<div class="w-2 h-2 rounded-full bg-emerald-400"></div>
										{/if}
									</button>
								</li>
							{/each}
						</ul>
					{/if}
				</section>
			{:else if activeChat === 'server-channels'}
				<section class="h-full flex flex-col">
				<div class="h-14 border-b border-slate-700/60 flex items-center justify-between px-4 bg-slate-800/40">
					<button class="btn btn-sm btn-ghost" onclick={backToServerList}>Back</button>
					<p class="font-semibold truncate flex-1 text-center">
						{#if $selectedServer}
							{$selectedServer.name}
						{:else}
							Server
						{/if}
					</p>
					{#if $selectedServer && (canManageSelectedServer() || canLeaveSelectedServer())}
						<div class="relative" bind:this={serverSettingsMenuEl}>
							<button
								type="button"
								class="btn btn-sm btn-circle btn-ghost text-xl leading-none"
								onclick={() => {
									serverSettingsMenuOpen = !serverSettingsMenuOpen;
								}}
							>
								⚙
							</button>

							{#if serverSettingsMenuOpen}
								<ul class="menu absolute right-0 z-[60] mt-2 w-56 rounded-box bg-slate-800 border border-slate-700 p-2 shadow">
									{#if canManageSelectedServer()}
										{#if $selectedServer?.isPublic === false}
											<li><button type="button" onclick={() => void openInviteUserModal()}>Invite user</button></li>
										{/if}
										<li><button type="button" onclick={openCreateChannelModal}>Create channel</button></li>
										<li><button type="button" onclick={handleOpenServerAvatarUpload}>Change server image</button></li>
										<li><button type="button" onclick={openUpdateServerNameModal}>Rename server</button></li>
										<li><button type="button" onclick={openUpdateServerDescriptionModal}>Edit server description</button></li>
										<li><button type="button" onclick={openUpdateServerVisibilityModal}>Edit server visibility</button></li>
										<li><button type="button" class="text-error" onclick={openDeleteServerModal}>Delete server</button></li>
									{/if}
									{#if canLeaveSelectedServer()}
										<li>
											<button type="button" class="text-error" disabled={leaveServerLoading} onclick={() => void handleLeaveSelectedServer()}>
												{leaveServerLoading ? 'Leaving...' : 'Leave server'}
											</button>
										</li>
									{/if}
								</ul>
							{/if}
						</div>
					{:else}
						<div class="w-10"></div>
					{/if}
				</div>

				{#if serverAvatarUploadError}
					<div class="p-3 pb-0">
						<div class="alert alert-error py-2 text-sm"><span>{serverAvatarUploadError}</span></div>
					</div>
				{/if}
				{#if serverAvatarUploadMessage}
					<div class="p-3 pb-0">
						<div class="alert alert-success py-2 text-sm"><span>{serverAvatarUploadMessage}</span></div>
					</div>
				{/if}
				{#if leaveServerError}
					<div class="p-3 pb-0">
						<div class="alert alert-error py-2 text-sm"><span>{leaveServerError}</span></div>
					</div>
				{/if}

				<div class="flex-1 overflow-auto p-3">
					{#if $selectedServer}
						{#if (serverChannels[$selectedServer.id] ?? []).length > 0}
							<ul class="menu rounded-box bg-slate-800/40 w-full space-y-1">
								{#each serverChannels[$selectedServer.id] ?? [] as channel}
									<li class="rounded-md border border-slate-700/55 bg-slate-800/40">
										<button class="min-h-11 px-3 text-base" type="button" onclick={() => openServerChannelChat(channel)}>
											<span>{channel.emoji} {channel.name}</span>
											{#if getChannelUnread($selectedServer.id, channel.id) > 0}
												<span class="badge badge-secondary ml-auto">{getChannelUnread($selectedServer.id, channel.id)}</span>
											{/if}
										</button>
									</li>
								{/each}
							</ul>
						{:else}
							<div class="h-full flex items-center justify-center">
								<p class="text-slate-400 text-center">No channels yet.</p>
							</div>
						{/if}
					{:else}
						<div class="h-full flex items-center justify-center">
							<p class="text-slate-400">No server selected.</p>
						</div>
					{/if}
				</div>
			</section>
		{:else}
				<section class="h-full flex flex-col">
					<div class="h-14 border-b border-slate-700/60 flex items-center justify-between px-4 bg-slate-800/40">
						<button class="btn btn-sm btn-ghost" onclick={handleChatBack}>Back</button>
						<p class="font-semibold truncate flex-1 text-center">
							{#if activeChat === 'friend'}
								{$selectedFriend?.username ?? 'Direct message'}
							{:else}
								{selectedChannel ? `${selectedChannel.emoji} ${selectedChannel.name}` : 'Channel chat'}
							{/if}
						</p>
						{#if activeChat === 'friend'}
							<div class="relative" bind:this={friendSettingsMenuEl}>
								<button
									type="button"
									class="btn btn-sm btn-circle btn-ghost text-xl leading-none"
									onclick={() => {
										friendSettingsMenuOpen = !friendSettingsMenuOpen;
									}}
								>
									⚙
								</button>

								{#if friendSettingsMenuOpen}
									<ul class="menu absolute right-0 z-[60] mt-2 w-56 rounded-box bg-slate-800 border border-slate-700 p-2 shadow">
										<li><button type="button" class="text-error" onclick={handleDeleteSelectedFriend}>Delete friend</button></li>
									</ul>
								{/if}
							</div>
						{:else if activeChat === 'server' && canManageSelectedServer()}
							<div class="relative" bind:this={channelSettingsMenuEl}>
								<button
									type="button"
									class="btn btn-sm btn-circle btn-ghost text-xl leading-none"
									onclick={() => {
										channelSettingsMenuOpen = !channelSettingsMenuOpen;
									}}
								>
									⚙
								</button>

								{#if channelSettingsMenuOpen}
									<ul class="menu absolute right-0 z-[60] mt-2 w-56 rounded-box bg-slate-800 border border-slate-700 p-2 shadow">
										<li><button type="button" onclick={openUpdateChannelNameModal}>Rename channel</button></li>
										<li><button type="button" onclick={openUpdateChannelEmojiModal}>Change channel logo</button></li>
										<li><button type="button" class="text-error" onclick={openDeleteChannelModal}>Delete channel</button></li>
									</ul>
								{/if}
							</div>
						{:else}
							<div class="w-16"></div>
						{/if}
					</div>

					<div class="flex-1 overflow-auto p-4 space-y-1" bind:this={chatContainer}>
						{#if chatLoading}
							<p class="text-sm text-slate-400">Loading messages...</p>
						{:else if chatMessages.length === 0}
							<p class="text-sm text-slate-400">No messages yet.</p>
						{:else}
							{#each chatMessages as msg, idx (msg.id)}
								<div class={`chat ${msg.userId === $session.userId ? 'chat-end' : 'chat-start'}`} data-msg-index={idx}>
									{#if activeChat === 'server' || activeChat === 'friend'}
										<div class={`chat-image avatar ${msg.userId === $session.userId ? 'order-2' : ''}`}>
											<div class="w-8 rounded-full bg-slate-700 text-slate-100 flex items-center justify-center overflow-hidden">
												{#if activeChat === 'server' && msg.authorAvatarUrl}
													<img src={msg.authorAvatarUrl} alt={`${msg.authorName ?? 'User'} avatar`} class="h-full w-full object-cover" />
												{:else if activeChat === 'friend' && msg.userId === $session.userId && currentUserAvatarUrl}
													<img src={currentUserAvatarUrl} alt="Your avatar" class="h-full w-full object-cover" />
												{:else if activeChat === 'friend' && msg.userId !== $session.userId && $selectedFriend?.avatarUrl}
													<img src={$selectedFriend.avatarUrl} alt={`${$selectedFriend.username} avatar`} class="h-full w-full object-cover" />
												{:else}
													<span class="text-[10px] font-semibold">
														{activeChat === 'friend'
															? (msg.userId === $session.userId
																? ($session.username?.slice(0, 1).toUpperCase() ?? 'U')
																: ($selectedFriend?.username?.slice(0, 1).toUpperCase() ?? 'U'))
															: (msg.authorName ?? 'U').slice(0, 1).toUpperCase()}
													</span>
												{/if}
											</div>
										</div>
									{/if}
									{#if activeChat === 'server' || activeChat === 'friend'}
										<div class="chat-header text-[11px] text-slate-300 mb-1">
											{activeChat === 'friend'
												? (msg.userId === $session.userId ? ($session.username ?? 'You') : ($selectedFriend?.username ?? 'Unknown user'))
												: (msg.userId === $session.userId ? ($session.username ?? msg.authorName ?? 'You') : (msg.authorName ?? 'Unknown user'))}
										</div>
									{/if}
									<div
										class={`chat-bubble whitespace-pre-wrap break-words ${msg.userId === $session.userId ? 'chat-bubble-primary' : 'bg-slate-700 text-slate-100'}`}
									>
										<div class="space-y-2">
											{#each parseMessageParts(msg.content) as part, partIndex (partIndex)}
												{#if part.type === 'text'}
													<span class="whitespace-pre-wrap break-words">{part.value}</span>
												{:else if part.type === 'link'}
													<a
														href={part.url}
														target="_blank"
														rel="noopener noreferrer"
														class="underline break-all"
														onclick={(event) => void openMessageUrl(event, part.url)}
													>
														{part.url}
													</a>
												{:else if part.type === 'image'}
													<a
														href={part.url}
														target="_blank"
														rel="noopener noreferrer"
														class="block"
														onclick={(event) => void openMessageUrl(event, part.url)}
													>
														<img
															src={part.url}
															alt="Shared media"
															loading="lazy"
															onload={() => keepChatAtBottomOnMediaLoad(idx)}
															onerror={() => keepChatAtBottomOnMediaLoad(idx)}
															class="mt-1 max-h-80 max-w-full rounded-md border border-slate-500/50 object-contain bg-black/20"
														/>
													</a>
												{:else}
													<div class="mt-1 max-w-full">
														<video
															src={part.url}
															controls
															preload="metadata"
															onloadedmetadata={() => keepChatAtBottomOnMediaLoad(idx)}
															onerror={() => keepChatAtBottomOnMediaLoad(idx)}
															class="max-h-80 max-w-full rounded-md border border-slate-500/50 bg-black/30"
														>
															<track kind="captions" srclang="en" label="No captions" />
														</video>
														<a
															href={part.url}
															target="_blank"
															rel="noopener noreferrer"
															class="mt-1 inline-block text-[11px] underline break-all"
															onclick={(event) => void openMessageUrl(event, part.url)}
														>
															Open video source
														</a>
													</div>
												{/if}
											{/each}
										</div>
									</div>
									<div class="chat-footer text-[10px] text-slate-500 mt-1">
										<div class="flex items-center gap-2">
											<span>{new Date(msg.createdAt * 1000).toLocaleTimeString()}</span>
											{#if msg.updatedAt}
												<span class="italic">(edited)</span>
											{/if}
											{#if msg.userId === $session.userId}
												<div class="relative" data-message-actions>
													<button
														type="button"
														class="btn btn-ghost btn-xs h-5 min-h-0 px-1"
														onclick={() => {
															messageActionsOpenForId = messageActionsOpenForId === msg.id ? null : msg.id;
														}}
													>
														⋯
													</button>
													{#if messageActionsOpenForId === msg.id}
														<div class="absolute right-0 bottom-6 z-20 rounded-md border border-slate-700 bg-slate-900/95 p-1 shadow">
															<button
																type="button"
																class="btn btn-ghost btn-xs whitespace-nowrap"
																onclick={() => handleStartEditChatMessage(msg.id, msg.content)}
															>
																Edit
															</button>
															<button
																type="button"
																class="btn btn-ghost btn-xs text-error whitespace-nowrap"
																onclick={() => void handleDeleteChatMessage(msg.id)}
															>
																Delete
															</button>
														</div>
													{/if}
												</div>
											{/if}
										</div>
									</div>
								</div>
							{/each}
						{/if}
					</div>

					<div class="border-t border-slate-700/60 p-3 bg-slate-800/50">
						{#if chatError}
							<div class="alert alert-error py-2 text-sm mb-2"><span>{chatError}</span></div>
						{/if}
						<div class="w-full space-y-2">
							{#if editingMessageId}
								<div class="flex items-center justify-between rounded-md border border-slate-700 bg-slate-900/70 px-3 py-2 text-xs text-slate-300">
									<span>Editing message. Press Esc to cancel.</span>
									<button type="button" class="btn btn-ghost btn-xs" onclick={handleCancelEditChatMessage}>Cancel</button>
								</div>
							{/if}
							{#if (activeChat === 'friend' || activeChat === 'server') && chatEmojiPickerOpen}
								<div class="flex justify-center" bind:this={chatEmojiPickerEl}>
									<div class="w-full max-w-md rounded-md border border-slate-700 overflow-hidden bg-slate-950">
										<emoji-picker style="width:100%;" onemoji-click={handleChatEmojiPicked}></emoji-picker>
									</div>
								</div>
							{/if}
							<div class="join w-full">
								<button
									bind:this={chatEmojiToggleButtonEl}
									class="btn join-item btn-ghost"
									type="button"
									onclick={() => {
										chatEmojiPickerOpen = !chatEmojiPickerOpen;
									}}
								>
									😊
								</button>
								<input
									bind:this={chatInputEl}
									class="input input-bordered join-item flex-1"
									placeholder={editingMessageId ? 'Edit your message...' : 'Type a message...'}
									bind:value={chatInput}
									onkeydown={(event) => {
										if (event.key === 'Escape' && editingMessageId) {
											event.preventDefault();
											handleCancelEditChatMessage();
											return;
										}

										if (event.key === 'Enter' && !event.shiftKey) {
											event.preventDefault();
											void handleSendChatMessage();
										}
									}}
								/>
								<button class="btn join-item btn-primary" onclick={handleSendChatMessage} disabled={chatSending}>
									{chatSending ? (editingMessageId ? 'Saving...' : 'Sending...') : (editingMessageId ? 'Save' : 'Send')}
								</button>
							</div>
						</div>
					</div>
				</section>
			{/if}
			</div>

			<input
				bind:this={avatarInputEl}
				type="file"
				accept="image/png,image/jpeg"
				class="hidden"
				onchange={handleAvatarPicked}
			/>

			<input
				bind:this={serverAvatarInputEl}
				type="file"
				accept="image/png,image/jpeg"
				class="hidden"
				onchange={handleServerAvatarPicked}
			/>

			{#if passwordModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-sm rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Change password</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { passwordModalOpen = false; }}>✕</button>
						</div>

						<label class="block w-full">
							<span class="label-text block mb-2">Current password</span>
							<input class="input input-bordered w-full" type="password" bind:value={currentPasswordInput} />
						</label>

						<label class="block w-full">
							<span class="label-text block mb-2">New password</span>
							<input class="input input-bordered w-full" type="password" bind:value={newPasswordInput} />
						</label>

						<label class="block w-full">
							<span class="label-text block mb-2">Confirm new password</span>
							<input class="input input-bordered w-full" type="password" bind:value={newPasswordConfirmInput} />
						</label>

						{#if passwordChangeError}
							<div class="alert alert-error py-2 text-sm"><span>{passwordChangeError}</span></div>
						{/if}
						{#if passwordChangeMessage}
							<div class="alert alert-success py-2 text-sm"><span>{passwordChangeMessage}</span></div>
						{/if}

						<div class="flex justify-end gap-2">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { passwordModalOpen = false; }}>Cancel</button>
							<button class="btn btn-sm btn-primary" type="button" disabled={passwordChangeLoading} onclick={handlePasswordChange}>
								{passwordChangeLoading ? 'Saving...' : 'Save'}
							</button>
						</div>
					</div>
				</div>
			{/if}

			{#if addFriendModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-sm rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Add friend</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { addFriendModalOpen = false; }}>✕</button>
						</div>

						<label class="block w-full">
							<span class="label-text block mb-2">Username</span>
							<input class="input input-bordered w-full" bind:value={addFriendUsername} />
						</label>

						{#if addFriendError}
							<div class="alert alert-error py-2 text-sm"><span>{addFriendError}</span></div>
						{/if}
						{#if addFriendMessage}
							<div class="alert alert-success py-2 text-sm"><span>{addFriendMessage}</span></div>
						{/if}

						<div class="flex gap-2 justify-end">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { addFriendModalOpen = false; }}>Cancel</button>
							<button class="btn btn-sm btn-primary" type="button" onclick={handleCreateFriendRequest}>Send</button>
						</div>
					</div>
				</div>
			{/if}

			{#if pendingModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-md rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3 max-h-[80vh] overflow-auto">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Pending requests</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { pendingModalOpen = false; }}>✕</button>
						</div>

						{#if pendingActionError}
							<div class="alert alert-error py-2 text-sm"><span>{pendingActionError}</span></div>
						{/if}

						{#if $friendPending.length === 0}
							<p class="text-sm text-slate-400">No pending requests.</p>
						{:else}
							<ul class="space-y-2">
								{#each $friendPending as req}
									<li class="flex items-center justify-between rounded-md border border-slate-700/70 bg-slate-800/40 px-3 py-2">
										<div class="flex items-center gap-3 min-w-0">
											{#if req.avatarUrl}
												<div class="avatar">
													<div class="w-9 rounded-full overflow-hidden">
														<img src={req.avatarUrl} alt={`${req.username} avatar`} />
													</div>
												</div>
											{:else}
												<div class="avatar placeholder">
													<div class="w-9 rounded-full bg-slate-700 text-slate-100">
														<span>{req.username[0]?.toUpperCase() ?? '?'}</span>
													</div>
												</div>
											{/if}
											<p class="font-medium truncate">{req.username}</p>
										</div>
										<div class="flex items-center gap-2">
											<button
												class="btn btn-success btn-xs"
												type="button"
												disabled={pendingActionInProgressId === req.requestId}
												onclick={() => handlePendingDecision(req.requestId, 'accept')}
											>
												✓
											</button>
											<button
												class="btn btn-error btn-xs"
												type="button"
												disabled={pendingActionInProgressId === req.requestId}
												onclick={() => handlePendingDecision(req.requestId, 'reject')}
											>
												✕
											</button>
										</div>
									</li>
								{/each}
							</ul>
						{/if}
					</div>
				</div>
			{/if}

			{#if serverInviteModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-lg rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3 max-h-[80vh] overflow-auto">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Server invites</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { serverInviteModalOpen = false; }}>✕</button>
						</div>

						{#if serverInviteActionError}
							<div class="alert alert-error py-2 text-sm"><span>{serverInviteActionError}</span></div>
						{/if}

						{#if serverInvites.length === 0}
							<p class="text-sm text-slate-400">No pending server invites.</p>
						{:else}
							<ul class="space-y-2">
								{#each serverInvites as invite}
									<li class="flex items-center justify-between rounded-md border border-slate-700/70 bg-slate-800/40 px-3 py-2">
										<div class="flex items-center gap-3 min-w-0 flex-1">
											<div class="avatar">
												<div class="w-9 rounded-full bg-slate-700 text-slate-100 flex items-center justify-center overflow-hidden">
													{#if invite.serverAvatarUrl}
														<img src={invite.serverAvatarUrl} alt={`${invite.serverName} avatar`} class="h-full w-full object-cover" />
													{:else}
														<span class="text-xs font-semibold">{invite.serverName.slice(0, 1).toUpperCase()}</span>
													{/if}
												</div>
											</div>
											<div class="min-w-0 flex-1">
												<p class="font-medium truncate">{invite.serverName}</p>
												<p class="text-xs text-slate-400 truncate">Invited by {invite.fromUserName}</p>
											</div>
										</div>
										<div class="flex items-center gap-2 pl-2">
											<button
												class="btn btn-success btn-xs"
												type="button"
												disabled={serverInviteActionInProgressId === invite.inviteId}
												onclick={() => void handleServerInviteDecision(invite.inviteId, 'accept')}
											>
												✓
											</button>
											<button
												class="btn btn-error btn-xs"
												type="button"
												disabled={serverInviteActionInProgressId === invite.inviteId}
												onclick={() => void handleServerInviteDecision(invite.inviteId, 'reject')}
											>
												✕
											</button>
										</div>
									</li>
								{/each}
							</ul>
						{/if}
					</div>
				</div>
			{/if}

			{#if inviteUserModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-lg rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3 max-h-[80vh] overflow-auto">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Invite user</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={closeInviteUserModal}>✕</button>
						</div>

						<input
							class="input input-bordered w-full"
							placeholder="Search friends..."
							bind:value={inviteUserSearch}
						/>

						{#if inviteUserError}
							<div class="alert alert-error py-2 text-sm"><span>{inviteUserError}</span></div>
						{/if}
						{#if inviteUserMessage}
							<div class="alert alert-success py-2 text-sm"><span>{inviteUserMessage}</span></div>
						{/if}

						{#if inviteUserLoading}
							<p class="text-sm text-slate-400">Loading friends...</p>
						{:else if filteredInvitableFriends().length === 0}
							<p class="text-sm text-slate-400">No invitable friends found.</p>
						{:else}
							<ul class="space-y-2">
								{#each filteredInvitableFriends() as friend}
									<li class="flex items-center justify-between rounded-md border border-slate-700/70 bg-slate-800/40 px-3 py-2">
										<div class="flex items-center gap-3 min-w-0">
											<div class="avatar">
												<div class="w-9 rounded-full bg-slate-700 text-slate-100 flex items-center justify-center overflow-hidden">
													{#if friend.avatarUrl}
														<img src={friend.avatarUrl} alt={`${friend.username} avatar`} class="h-full w-full object-cover" />
													{:else}
														<span class="text-xs font-semibold">{friend.username.slice(0, 1).toUpperCase()}</span>
													{/if}
												</div>
											</div>
											<p class="font-medium truncate">{friend.username}</p>
										</div>
										<button
											class="btn btn-primary btn-xs"
											type="button"
											disabled={inviteUserInProgressId === friend.id}
											onclick={() => void handleInviteUserToSelectedServer(friend.id, friend.username)}
										>
											Invite
										</button>
									</li>
								{/each}
							</ul>
						{/if}
					</div>
				</div>
			{/if}

			{#if joinServerModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div
						bind:this={joinServerModalEl}
						class="w-full max-w-2xl rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3 max-h-[85vh] overflow-hidden flex flex-col"
					>
						<div class="h-10 flex items-center justify-between">
							<button class="btn btn-sm btn-ghost" type="button" onclick={closeJoinServerModal}>Back</button>
							<p class="font-semibold flex-1 text-center">Public servers</p>
							<div class="w-16"></div>
						</div>

						<form
							class="join w-full"
							onsubmit={(event) => {
								event.preventDefault();
								void handlePublicServersSearchSubmit();
							}}
						>
							<input
								class="input input-bordered join-item flex-1"
								placeholder="Search public servers by name..."
								bind:value={publicServersSearch}
							/>
							<button class="btn btn-primary join-item" type="submit" disabled={publicServersLoading}>Search</button>
						</form>

						<select class="select select-bordered w-full" bind:value={publicServersSort} onchange={(event) => void handlePublicServersSortChange(event)}>
							<option value="newest">Newest first</option>
							<option value="oldest">Oldest first</option>
							<option value="most-members">Most members</option>
							<option value="fewest-members">Fewest members</option>
							<option value="name-asc">Name (A-Z)</option>
							<option value="name-desc">Name (Z-A)</option>
						</select>

						{#if publicServersError}
							<div class="alert alert-error py-2 text-sm"><span>{publicServersError}</span></div>
						{/if}
						{#if joinServerMessage}
							<div class="alert alert-success py-2 text-sm"><span>{joinServerMessage}</span></div>
						{/if}

						<div class="flex-1 overflow-auto rounded-md border border-slate-700/60 bg-slate-800/30 p-2">
							{#if publicServersLoading}
								<p class="text-sm text-slate-400 p-2">Loading public servers...</p>
							{:else if publicServers.length === 0}
								<p class="text-sm text-slate-400 p-2">No matching public servers.</p>
							{:else}
								<ul class="menu rounded-box w-full">
									{#each publicServers as item}
										<li>
											<div class="flex items-center gap-3">
												<div class="avatar">
													<div class="w-9 rounded-full bg-slate-700 text-slate-100 flex items-center justify-center overflow-hidden">
														{#if item.avatarUrl}
															<img src={item.avatarUrl} alt={`${item.name} avatar`} class="h-full w-full object-cover" />
														{:else}
															<span class="text-xs font-semibold">{item.name.slice(0, 1).toUpperCase()}</span>
														{/if}
													</div>
												</div>
												<div class="flex-1 min-w-0">
													<p class="font-medium truncate">{item.name}</p>
													<p class="text-xs text-slate-400 truncate">{item.description || 'No MOTD set.'}</p>
													<p class="text-[11px] text-slate-500 truncate">{item.memberCount ?? 0} members</p>
												</div>
												<button class="btn btn-xs btn-primary" type="button" onclick={() => openJoinServerConfirmModal(item)}>+</button>
											</div>
										</li>
									{/each}
								</ul>
							{/if}
						</div>
					</div>
				</div>
			{/if}

			{#if joinServerConfirmModalOpen && joinServerTarget}
				<div class="fixed inset-0 z-50 bg-black/70 flex items-center justify-center p-4">
					<div class="w-full max-w-sm rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3">
						<h3 class="font-semibold">Join server</h3>
						<p class="text-sm text-slate-300">Do you want to join {joinServerTarget.name}?</p>
						<div class="flex justify-end gap-2">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { joinServerConfirmModalOpen = false; joinServerTarget = null; }}>Cancel</button>
							<button class="btn btn-sm btn-primary" type="button" disabled={joinServerLoading} onclick={() => void handleJoinServer()}>
								{joinServerLoading ? 'Joining...' : 'Join'}
							</button>
						</div>
					</div>
				</div>
			{/if}

			{#if createServerModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div
						bind:this={createServerModalEl}
						class="w-full max-w-md rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3"
					>
						<div class="h-10 flex items-center justify-between">
							<button class="btn btn-sm btn-ghost" type="button" onclick={closeCreateServerModal}>Back</button>
							<p class="font-semibold flex-1 text-center">Create server</p>
							<div class="w-16"></div>
						</div>

						<label class="block w-full">
							<span class="label-text block mb-2">Server name</span>
							<input class="input input-bordered w-full" bind:value={createServerName} maxlength="64" placeholder="e.g. Poseidon Guild" />
						</label>

						<label class="block w-full">
							<span class="label-text block mb-2">Description (optional)</span>
							<textarea class="textarea textarea-bordered w-full" rows="3" bind:value={createServerDescription} placeholder="Short server description"></textarea>
						</label>

						<label class="label cursor-pointer justify-start gap-3">
							<input class="checkbox checkbox-sm" type="checkbox" bind:checked={createServerIsPublic} />
							<span class="label-text">Public server</span>
						</label>

						<label class="block w-full">
							<span class="label-text block mb-2">Server image (optional)</span>
							<input
								class="file-input file-input-bordered w-full"
								type="file"
								accept="image/png,image/jpeg"
								onchange={handleCreateServerAvatarPicked}
							/>
							{#if createServerAvatarFile}
								<p class="mt-2 text-xs text-slate-400 truncate">Selected: {createServerAvatarFile.name}</p>
							{/if}
						</label>

						{#if createServerError}
							<div class="alert alert-error py-2 text-sm"><span>{createServerError}</span></div>
						{/if}

						<div class="flex gap-2 justify-end">
							<button class="btn btn-sm btn-ghost" type="button" onclick={closeCreateServerModal}>Cancel</button>
							<button class="btn btn-sm btn-primary" type="button" disabled={createServerLoading} onclick={handleCreateServer}>
								{createServerLoading ? 'Creating...' : 'Create'}
							</button>
						</div>
					</div>
				</div>
			{/if}

			{#if createChannelModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-md rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Create channel</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { createChannelModalOpen = false; }}>✕</button>
						</div>

						<label class="block w-full">
							<span class="label-text block mb-2">Channel name</span>
							<input class="input input-bordered w-full" bind:value={createChannelName} maxlength="64" placeholder="e.g. general" />
						</label>

						<label class="block w-full">
							<span class="label-text block mb-2">Emoji</span>
							<button
								type="button"
								class="btn btn-outline w-full justify-start"
								onclick={() => {
									createChannelEmojiPickerOpen = !createChannelEmojiPickerOpen;
								}}
							>
								<span class="text-lg">{createChannelEmoji}</span>
								<span class="text-slate-300">Select channel logo</span>
							</button>
						</label>

						{#if createChannelEmojiPickerOpen}
							<div class="channel-emoji-picker flex justify-center">
								<div class="w-full rounded-md border border-slate-700 overflow-hidden bg-slate-950">
									<emoji-picker style="width:100%;" onemoji-click={handleCreateChannelEmojiPicked}></emoji-picker>
								</div>
							</div>
						{/if}

						{#if createChannelError}
							<div class="alert alert-error py-2 text-sm"><span>{createChannelError}</span></div>
						{/if}

						<div class="flex gap-2 justify-end">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { createChannelModalOpen = false; }}>Cancel</button>
							<button class="btn btn-sm btn-primary" type="button" disabled={createChannelLoading} onclick={handleCreateChannel}>
								{createChannelLoading ? 'Creating...' : 'Create'}
							</button>
						</div>
					</div>
				</div>
			{/if}

			{#if updateChannelNameModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-md rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Rename channel</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { updateChannelNameModalOpen = false; }}>✕</button>
						</div>

						<label class="block w-full">
							<span class="label-text block mb-2">Channel name</span>
							<input class="input input-bordered w-full" bind:value={updateChannelNameValue} maxlength="64" />
						</label>

						{#if updateChannelNameError}
							<div class="alert alert-error py-2 text-sm"><span>{updateChannelNameError}</span></div>
						{/if}

						<div class="flex gap-2 justify-end">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { updateChannelNameModalOpen = false; }}>Cancel</button>
							<button class="btn btn-sm btn-primary" type="button" disabled={updateChannelNameLoading} onclick={handleUpdateChannelName}>
								{updateChannelNameLoading ? 'Saving...' : 'Save'}
							</button>
						</div>
					</div>
				</div>
			{/if}

			{#if updateChannelEmojiModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-md rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Change channel logo</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { updateChannelEmojiModalOpen = false; }}>✕</button>
						</div>

						<button
							type="button"
							class="btn btn-outline w-full justify-start"
							onclick={() => {
								updateChannelEmojiPickerOpen = !updateChannelEmojiPickerOpen;
							}}
						>
							<span class="text-lg">{updateChannelEmojiValue}</span>
							<span class="text-slate-300">Select channel logo</span>
						</button>

						{#if updateChannelEmojiPickerOpen}
							<div class="channel-emoji-picker flex justify-center">
								<div class="w-full rounded-md border border-slate-700 overflow-hidden bg-slate-950">
									<emoji-picker style="width:100%;" onemoji-click={handleUpdateChannelEmojiPicked}></emoji-picker>
								</div>
							</div>
						{/if}

						{#if updateChannelEmojiError}
							<div class="alert alert-error py-2 text-sm"><span>{updateChannelEmojiError}</span></div>
						{/if}

						<div class="flex gap-2 justify-end">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { updateChannelEmojiModalOpen = false; }}>Cancel</button>
							<button class="btn btn-sm btn-primary" type="button" disabled={updateChannelEmojiLoading} onclick={handleUpdateChannelEmoji}>
								{updateChannelEmojiLoading ? 'Saving...' : 'Save'}
							</button>
						</div>
					</div>
				</div>
			{/if}

			{#if deleteChannelModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-md rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold text-error">Delete channel</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { deleteChannelModalOpen = false; }}>✕</button>
						</div>

						<p class="text-sm text-slate-300">Are you sure you want to delete this channel? This cannot be undone.</p>

						<label class="block w-full">
							<span class="label-text block mb-2">Type "DELETE" to confirm</span>
							<input class="input input-bordered w-full" bind:value={deleteChannelConfirmText} placeholder="DELETE" />
						</label>

						{#if deleteChannelError}
							<div class="alert alert-error py-2 text-sm"><span>{deleteChannelError}</span></div>
						{/if}

						<div class="flex gap-2 justify-end">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { deleteChannelModalOpen = false; }}>Cancel</button>
							<button class="btn btn-sm btn-error" type="button" disabled={deleteChannelLoading || deleteChannelConfirmText !== 'DELETE'} onclick={handleDeleteChannel}>
								{deleteChannelLoading ? 'Deleting...' : 'Delete'}
							</button>
						</div>
					</div>
				</div>
			{/if}

			{#if updateServerNameModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-md rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Rename server</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { updateServerNameModalOpen = false; }}>✕</button>
						</div>

						<label class="block w-full">
							<span class="label-text block mb-2">Server name</span>
							<input class="input input-bordered w-full" bind:value={updateServerNameValue} maxlength="64" />
						</label>

						{#if updateServerNameError}
							<div class="alert alert-error py-2 text-sm"><span>{updateServerNameError}</span></div>
						{/if}

						<div class="flex gap-2 justify-end">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { updateServerNameModalOpen = false; }}>Cancel</button>
							<button class="btn btn-sm btn-primary" type="button" disabled={updateServerNameLoading} onclick={handleUpdateServerName}>
								{updateServerNameLoading ? 'Saving...' : 'Save'}
							</button>
						</div>
					</div>
				</div>
			{/if}

			{#if updateServerDescriptionModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-md rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Edit server description</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { updateServerDescriptionModalOpen = false; }}>✕</button>
						</div>

						<label class="block w-full">
							<span class="label-text block mb-2">Server description</span>
							<textarea class="textarea textarea-bordered w-full" rows="4" bind:value={updateServerDescriptionValue} placeholder="Server description"></textarea>
						</label>

						{#if updateServerDescriptionError}
							<div class="alert alert-error py-2 text-sm"><span>{updateServerDescriptionError}</span></div>
						{/if}

						<div class="flex gap-2 justify-end">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { updateServerDescriptionModalOpen = false; }}>Cancel</button>
							<button class="btn btn-sm btn-primary" type="button" disabled={updateServerDescriptionLoading} onclick={handleUpdateServerDescription}>
								{updateServerDescriptionLoading ? 'Saving...' : 'Save'}
							</button>
						</div>
					</div>
				</div>
			{/if}

			{#if updateServerVisibilityModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-md rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Edit server visibility</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { updateServerVisibilityModalOpen = false; }}>✕</button>
						</div>

						<label class="label cursor-pointer justify-start gap-3">
							<input class="checkbox checkbox-sm" type="checkbox" bind:checked={updateServerVisibilityValue} />
							<span class="label-text">Public server</span>
						</label>
						<p class="text-xs text-slate-400">Public servers are visible in discovery and anyone can join.</p>

						{#if updateServerVisibilityError}
							<div class="alert alert-error py-2 text-sm"><span>{updateServerVisibilityError}</span></div>
						{/if}

						<div class="flex gap-2 justify-end">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { updateServerVisibilityModalOpen = false; }}>Cancel</button>
							<button class="btn btn-sm btn-primary" type="button" disabled={updateServerVisibilityLoading} onclick={handleUpdateServerVisibility}>
								{updateServerVisibilityLoading ? 'Saving...' : 'Save'}
							</button>
						</div>
					</div>
				</div>
			{/if}

			{#if deleteServerModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-md rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold text-error">Delete server</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { deleteServerModalOpen = false; }}>✕</button>
						</div>

						<p class="text-sm text-slate-300">Are you sure you want to delete this server? This cannot be undone.</p>

						<label class="block w-full">
							<span class="label-text block mb-2">Type "DELETE" to confirm</span>
							<input class="input input-bordered w-full" bind:value={deleteServerConfirmText} placeholder="DELETE" />
						</label>

						{#if deleteServerError}
							<div class="alert alert-error py-2 text-sm"><span>{deleteServerError}</span></div>
						{/if}

						<div class="flex gap-2 justify-end">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { deleteServerModalOpen = false; }}>Cancel</button>
							<button class="btn btn-sm btn-error" type="button" disabled={deleteServerLoading || deleteServerConfirmText !== 'DELETE'} onclick={handleDeleteServer}>
								{deleteServerLoading ? 'Deleting...' : 'Delete'}
							</button>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</main>
{/if}
