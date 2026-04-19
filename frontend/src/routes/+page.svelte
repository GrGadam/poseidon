<script lang="ts">
	import { get } from 'svelte/store';
	import { onMount, tick } from 'svelte';
	import type {
		ChannelMessageResponse,
		ChannelResponse,
		DmMessageResponse,
		DmThreadResponse,
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
	let lastLoadedToken = $state<string | null>(null);
	let profileUploadMessage = $state<string | null>(null);
	let profileUploadError = $state<string | null>(null);
	let currentUserAvatarUrl = $state<string | null>(null);
	let serverAvatarUploadMessage = $state<string | null>(null);
	let serverAvatarUploadError = $state<string | null>(null);
	let serverAvatarMessageTimeout = $state<ReturnType<typeof setTimeout> | null>(null);
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
	};

	let activeChat = $state<ChatView>('none');
	let selectedChannel = $state<ServerChannel | null>(null);
	let activeDmThreadId = $state<string | null>(null);
	let chatMessages = $state<ChatMessage[]>([]);
	let chatInput = $state('');
	let chatEmojiPickerOpen = $state(false);
	let chatEmojiPickerEl = $state<HTMLElement | null>(null);
	let chatEmojiToggleButtonEl = $state<HTMLButtonElement | null>(null);
	let chatLoading = $state(false);
	let chatSending = $state(false);
	let chatError = $state<string | null>(null);
	let chatContainer = $state<HTMLDivElement | null>(null);

	let serverChannels = $state<Record<string, ServerChannel[]>>({});
	let channelUnreadByServer = $state<Record<string, Record<string, number>>>({});
	let dmThreadByFriendId = $state<Record<string, string>>({});
	let friendByDmThreadId = $state<Record<string, string>>({});
	let onlineUserIdsSnapshot = $state<string[]>([]);
	let leaveServerLoading = $state(false);
	let leaveServerError = $state<string | null>(null);
	let messageAuthorAvatarUrls = $state<Record<string, string | null>>({});

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
			createdAt: item.created_at
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
		if (!content) {
			return;
		}

		chatSending = true;
		chatError = null;

		try {
			if (activeChat === 'friend' && activeDmThreadId) {
				const sent = await apiClient.dmSendMessage(token, activeDmThreadId, content);
				const mapped = await toChatMessage(sent);
				chatMessages = [...chatMessages, mapped].sort(byCreatedAtAsc);
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
			return;
		}

		try {
			await sendFriendRequest(token, name);
			addFriendUsername = '';
			addFriendMessage = 'Friend request sent.';
		} catch (error) {
			if (error instanceof Error) {
				if (error.message === 'not_found') {
					addFriendError = 'User not found.';
					return;
				}
				if (error.message.includes('already exists') || error.message.includes('request already exists')) {
					addFriendError = 'A pending request already exists for this user.';
					return;
				}
				addFriendError = error.message;
				return;
			}

			addFriendError = 'Failed to send friend request.';
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

	const openFriendChat = async (friend: FriendEntry) => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		const unreadCount = friend.unread;

		selectFriend(friend);
		selectedChannel = null;
		chatInput = '';
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
			joinServerConfirmModalOpen = false;
			joinServerTarget = null;
		} catch (error) {
			publicServersError = error instanceof Error ? error.message : 'Failed to join server.';
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
			return;
		}

		if (name.length > 64) {
			createServerError = 'Server name must be at most 64 characters.';
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
					uploadWarning =
						error instanceof Error
							? error.message
							: 'Server created, but image upload failed. You can upload it later from server settings.';
				}
			}

			await refreshServersData();
			createServerMessage = uploadWarning
				? `Server created, but image upload failed: ${uploadWarning}`
				: 'Server created successfully.';
			createServerName = '';
			createServerDescription = '';
			createServerIsPublic = true;
			createServerAvatarFile = null;
			createServerModalOpen = false;
		} catch (error) {
			createServerError = error instanceof Error ? error.message : 'Failed to create server.';
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
		if (serverAvatarMessageTimeout) {
			clearTimeout(serverAvatarMessageTimeout);
			serverAvatarMessageTimeout = null;
		}

		try {
			await apiClient.uploadServerAvatar(token, serverId, file);
			await refreshServersData();
			serverAvatarUploadMessage = 'Server image uploaded.';
			serverAvatarMessageTimeout = setTimeout(() => {
				serverAvatarUploadMessage = null;
				serverAvatarMessageTimeout = null;
			}, 3000);
		} catch (error) {
			serverAvatarUploadError = error instanceof Error ? error.message : 'Failed to upload server image.';
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
			currentPasswordInput = '';
			newPasswordInput = '';
			newPasswordConfirmInput = '';
		} catch (error) {
			if (error instanceof Error && error.message === 'unauthorized') {
				passwordChangeError = 'Current password is incorrect.';
			} else {
				passwordChangeError = error instanceof Error ? error.message : 'Failed to update password.';
			}
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
		if (serverAvatarMessageTimeout) {
			clearTimeout(serverAvatarMessageTimeout);
			serverAvatarMessageTimeout = null;
		}

		try {
			await apiClient.uploadMyAvatar(token, file);
			if ($session.userId) {
				markAvatarDirty($session.userId);
			}
			await Promise.all([refreshFriends(token), refreshPendingRequests(token)]);
			await refreshCurrentUserAvatar();
			profileUploadMessage = 'Profile image uploaded.';
		} catch (error) {
			profileUploadError = error instanceof Error ? error.message : 'Failed to upload profile image.';
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
				await refreshFriendsData();
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
					await loadActiveMessages({ silent: true, forceBottom: true });
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
					await loadActiveMessages({ silent: true, forceBottom: true });
				}
			}
		};

		const handleOutsideClick = (event: MouseEvent) => {
			const target = event.target as Node | null;
			const ignoreModalOutsideClose = Date.now() < ignoreModalOutsideCloseUntil;

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
		};
	});

	$effect(() => {
		const token = $session.accessToken;
		if (!token) {
			lastLoadedToken = null;
			setCurrentUserAvatarUrl(null);
			clearMessageAuthorAvatarCache();
			return;
		}

		if (lastLoadedToken !== token) {
			lastLoadedToken = token;
			void Promise.all([refreshFriendsData(), refreshServersData(), refreshCurrentUserAvatar()]);
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
	<main class="h-screen w-full p-2">
		<div class="h-full w-full overflow-hidden border border-slate-700/70 bg-slate-900/85 flex flex-col">
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
							<button class="btn btn-primary btn-sm w-full" type="button" onclick={openAddFriendModal}>+ Add friend</button>
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

					<div class="flex-1 overflow-auto p-4 space-y-2" bind:this={chatContainer}>
						{#if chatLoading}
							<p class="text-sm text-slate-400">Loading messages...</p>
						{:else if chatMessages.length === 0}
							<p class="text-sm text-slate-400">No messages yet.</p>
						{:else}
							{#each chatMessages as msg, idx}
								<div class={`chat ${msg.userId === $session.userId ? 'chat-end' : 'chat-start'}`} data-msg-index={idx}>
									{#if activeChat === 'server'}
										<div class={`chat-image avatar ${msg.userId === $session.userId ? 'order-2' : ''}`}>
											<div class="w-8 rounded-full bg-slate-700 text-slate-100 flex items-center justify-center overflow-hidden">
												{#if msg.authorAvatarUrl}
													<img src={msg.authorAvatarUrl} alt={`${msg.authorName ?? 'User'} avatar`} class="h-full w-full object-cover" />
												{:else}
													<span class="text-[10px] font-semibold">{(msg.authorName ?? 'U').slice(0, 1).toUpperCase()}</span>
												{/if}
											</div>
										</div>
									{/if}
									{#if activeChat === 'server'}
										<div class="chat-header text-[11px] text-slate-300 mb-1">
											{msg.userId === $session.userId ? ($session.username ?? msg.authorName ?? 'You') : (msg.authorName ?? 'Unknown user')}
										</div>
									{/if}
									<div
										class={`chat-bubble whitespace-pre-wrap break-words ${msg.userId === $session.userId ? 'chat-bubble-primary' : 'bg-slate-700 text-slate-100'}`}
									>
										{msg.content}
									</div>
									<div class="chat-footer text-[10px] text-slate-500 mt-1">
										{new Date(msg.createdAt * 1000).toLocaleTimeString()}
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
									class="input input-bordered join-item flex-1"
									placeholder="Type a message..."
									bind:value={chatInput}
									onkeydown={(event) => {
										if (event.key === 'Enter' && !event.shiftKey) {
											event.preventDefault();
											void handleSendChatMessage();
										}
									}}
								/>
								<button class="btn join-item btn-primary" onclick={handleSendChatMessage} disabled={chatSending}>
									{chatSending ? 'Sending...' : 'Send'}
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
