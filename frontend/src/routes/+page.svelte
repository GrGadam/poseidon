<script lang="ts">
	import { get } from 'svelte/store';
	import { onMount, tick } from 'svelte';
	import type {
		ChannelMessageResponse,
		ChannelResponse,
		DmMessageResponse,
		DmThreadResponse
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
	let addFriendUsername = $state('');
	let addFriendMessage = $state<string | null>(null);
	let addFriendError = $state<string | null>(null);
	let createServerName = $state('');
	let createServerDescription = $state('');
	let createServerIsPublic = $state(true);
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
	let friendSettingsMenuOpen = $state(false);
	let friendSettingsMenuEl = $state<HTMLElement | null>(null);
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
	let passwordModalOpen = $state(false);
	let currentPasswordInput = $state('');
	let newPasswordInput = $state('');
	let newPasswordConfirmInput = $state('');
	let passwordChangeError = $state<string | null>(null);
	let passwordChangeMessage = $state<string | null>(null);
	let passwordChangeLoading = $state(false);
	let avatarInputEl = $state<HTMLInputElement | null>(null);
	let settingsMenuOpen = $state(false);
	let settingsMenuEl = $state<HTMLElement | null>(null);

	type ChatView = 'none' | 'friend' | 'server' | 'server-channels';
	type ServerChannel = { id: string; name: string; emoji: string };
	type ChatMessage = { id: string; userId: string; content: string; createdAt: number };

	let activeChat = $state<ChatView>('none');
	let selectedChannel = $state<ServerChannel | null>(null);
	let activeDmThreadId = $state<string | null>(null);
	let chatMessages = $state<ChatMessage[]>([]);
	let chatInput = $state('');
	let chatLoading = $state(false);
	let chatSending = $state(false);
	let chatError = $state<string | null>(null);
	let chatContainer = $state<HTMLDivElement | null>(null);

	let serverChannels = $state<Record<string, ServerChannel[]>>({});
	let dmThreadByFriendId = $state<Record<string, string>>({});
	let friendByDmThreadId = $state<Record<string, string>>({});
	let onlineUserIdsSnapshot = $state<string[]>([]);

	const loadUserAvatarUrl = async (accessToken: string, userId: string): Promise<string | null> => {
		const res = await fetch(`${apiConfig.baseUrl}/users/${encodeURIComponent(userId)}/avatar`, {
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

		const data = (await apiClient.servers(token)) as Array<{
			id: string;
			name: string;
			description?: string | null;
			owner_id: string;
		}>;

		const previousServers = get(servers);
		const previousById = new Map(previousServers.map((entry) => [entry.id, entry]));
		const incomingIds = new Set(data.map((item) => item.id));

		for (const entry of previousServers) {
			if (!incomingIds.has(entry.id) && entry.avatarUrl?.startsWith('blob:')) {
				URL.revokeObjectURL(entry.avatarUrl);
			}
		}

		const nextServers = await Promise.all(
			data.map(async (item) => {
				const previous = previousById.get(item.id);
				let avatarUrl = previous?.avatarUrl ?? null;

				if (!avatarUrl) {
					avatarUrl = await loadUserAvatarUrl(token, item.owner_id);
				}

				return {
					id: item.id,
					name: item.name,
					description: item.description ?? '',
					avatarUrl,
					hasUnread: false
				};
			})
		);

		servers.set(nextServers);
	};

	const toChatMessage = (item: DmMessageResponse | ChannelMessageResponse): ChatMessage => ({
		id: item.id,
		userId: item.user_id,
		content: item.content,
		createdAt: item.created_at
	});

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
				nextMessages = data.map(toChatMessage).sort(byCreatedAtAsc);
			} else if (activeChat === 'server' && selectedChannel?.id) {
				const data = await apiClient.channelMessages(token, selectedChannel.id);
				nextMessages = data.map(toChatMessage).sort(byCreatedAtAsc);
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
				chatMessages = [...chatMessages, toChatMessage(sent)].sort(byCreatedAtAsc);
				const friendId = friendByDmThreadId[activeDmThreadId];
				if (friendId) {
					setFriendLastMessage(friendId, sent.content);
					clearFriendUnread(friendId);
				}
				chatInput = '';
				return;
			}

			if (activeChat === 'server' && selectedChannel?.id) {
				const sent = await apiClient.channelSendMessage(token, selectedChannel.id, content);
				chatMessages = [...chatMessages, toChatMessage(sent)].sort(byCreatedAtAsc);
				chatInput = '';
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
		selectedChannel = null;
		activeDmThreadId = null;
		chatMessages = [];
		chatError = null;
		chatInput = '';

		try {
			await loadServerChannels(server.id);
		} catch (error) {
			chatError = error instanceof Error ? error.message : 'Failed to load channels.';
		}
	};

	const openServerChannelChat = async (channel: ServerChannel) => {
		selectedChannel = channel;
		activeDmThreadId = null;
		chatInput = '';
		chatMessages = [];
		chatError = null;
		activeChat = 'server';
		await loadActiveMessages();
		scrollChatOnOpen(0);
	};

	const closeChat = () => {
		activeChat = 'none';
		friendSettingsMenuOpen = false;
		activeDmThreadId = null;
		chatMessages = [];
		chatError = null;
		chatInput = '';
	};

	const backToServerList = () => {
		activeChat = 'none';
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
		createServerModalOpen = true;
		createServerError = null;
		createServerMessage = null;
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
			await apiClient.createServer(token, name, description, createServerIsPublic);
			await refreshServersData();
			createServerMessage = 'Server created successfully.';
			createServerName = '';
			createServerDescription = '';
			createServerIsPublic = true;
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
			createChannelModalOpen = false;
		} catch (error) {
			createChannelError = error instanceof Error ? error.message : 'Failed to create channel.';
		} finally {
			createChannelLoading = false;
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

		try {
			await apiClient.uploadMyAvatar(token, file);
			if ($session.userId) {
				markAvatarDirty($session.userId);
			}
			await Promise.all([refreshFriends(token), refreshPendingRequests(token)]);
			profileUploadMessage = 'Profile image uploaded.';
		} catch (error) {
			profileUploadError = error instanceof Error ? error.message : 'Failed to upload profile image.';
		} finally {
			input.value = '';
		}
	};

	const handleLogout = () => {
		settingsMenuOpen = false;
		activeChat = 'none';
		selectedChannel = null;
		activeDmThreadId = null;
		chatMessages = [];
		chatInput = '';
		chatError = null;
		dmThreadByFriendId = {};
		friendByDmThreadId = {};
		onlineUserIdsSnapshot = [];
		addFriendModalOpen = false;
		pendingModalOpen = false;
		createServerModalOpen = false;
		createChannelModalOpen = false;
		serverSettingsMenuOpen = false;
		updateServerNameModalOpen = false;
		updateServerDescriptionModalOpen = false;
		updateServerVisibilityModalOpen = false;
		deleteServerModalOpen = false;
		friendSettingsMenuOpen = false;
		addFriendUsername = '';
		addFriendError = null;
		addFriendMessage = null;
		createServerName = '';
		createServerDescription = '';
		createServerIsPublic = true;
		createServerError = null;
		createServerMessage = null;
		createChannelName = '';
		createChannelEmoji = '💬';
		createChannelError = null;
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
		const handler = async (event: Event) => {
			const custom = event as CustomEvent<{ kind: string; payload: unknown }>;
			const kind = custom.detail?.kind;
			const payload = (custom.detail?.payload ?? {}) as {
				id?: string;
				thread_id?: string;
				channel_id?: string;
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
				await Promise.all([refreshFriends($session.accessToken), refreshPendingRequests($session.accessToken)]);
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

			if (kind === 'server.created' || kind === 'server.updated' || kind === 'server.deleted' || kind === 'server.joined') {
				await refreshServersData();
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

			if (
				(kind === 'channel.message.created' || kind === 'channel.message.updated' || kind === 'channel.message.deleted') &&
				activeChat === 'server' &&
				selectedChannel?.id &&
				payload.channel_id === selectedChannel.id
			) {
				await loadActiveMessages({ silent: true, forceBottom: true });
			}
		};

		const handleOutsideClick = (event: MouseEvent) => {
			const target = event.target as Node | null;

			if (settingsMenuOpen && settingsMenuEl && target && !settingsMenuEl.contains(target)) {
				settingsMenuOpen = false;
			}

			if (serverSettingsMenuOpen && serverSettingsMenuEl && target && !serverSettingsMenuEl.contains(target)) {
				serverSettingsMenuOpen = false;
			}

			if (friendSettingsMenuOpen && friendSettingsMenuEl && target && !friendSettingsMenuEl.contains(target)) {
				friendSettingsMenuOpen = false;
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
			return;
		}

		if (lastLoadedToken !== token) {
			lastLoadedToken = token;
			void Promise.all([refreshFriendsData(), refreshServersData()]);
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
						<p class="text-sm text-slate-300 truncate">{$session.username ?? 'User'}</p>
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

					<div class="tabs tabs-boxed bg-slate-800/70 mb-3 w-full">
						<button class={`tab flex-1 ${$selectedTab === 'friends' ? 'tab-active' : ''}`} onclick={() => { $selectedTab = 'friends'; }}>
							Friends
						</button>
						<button class={`tab flex-1 ${$selectedTab === 'servers' ? 'tab-active' : ''}`} onclick={() => { $selectedTab = 'servers'; }}>
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
						<div class="mb-3">
							<button class="btn btn-primary btn-sm w-full" type="button" onclick={openCreateServerModal}>+ Create server</button>
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
								<li><button type="button" onclick={openCreateChannelModal}>Create channel</button></li>
								<li><button type="button" onclick={openUpdateServerNameModal}>Rename server</button></li>
								<li><button type="button" onclick={openUpdateServerDescriptionModal}>Edit server description</button></li>
								<li><button type="button" onclick={openUpdateServerVisibilityModal}>Edit server visibility</button></li>
								<li><button type="button" class="text-error" onclick={openDeleteServerModal}>Delete server</button></li>
							</ul>
						{/if}
					</div>
				</div>

				<div class="flex-1 overflow-auto p-3">
					{#if $selectedServer}
						{#if (serverChannels[$selectedServer.id] ?? []).length > 0}
							<ul class="menu rounded-box bg-slate-800/40 w-full">
								{#each serverChannels[$selectedServer.id] ?? [] as channel}
									<li>
										<button type="button" onclick={() => openServerChannelChat(channel)}>
											{channel.emoji} {channel.name}
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
						<button class="btn btn-sm btn-ghost" onclick={closeChat}>Back</button>
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
										<li><button type="button" class="text-error" onclick={handleDeleteSelectedFriend}>Delete friend and conversation</button></li>
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
						<div class="join w-full">
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

			{#if createServerModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-md rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Create server</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { createServerModalOpen = false; }}>✕</button>
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

						{#if createServerError}
							<div class="alert alert-error py-2 text-sm"><span>{createServerError}</span></div>
						{/if}

						<div class="flex gap-2 justify-end">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { createServerModalOpen = false; }}>Cancel</button>
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
							<input class="input input-bordered w-full" bind:value={createChannelEmoji} maxlength="5" placeholder="e.g. 💬" />
						</label>

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
