<script lang="ts">
	import { get } from 'svelte/store';
	import { onMount } from 'svelte';
	import type {
		ChannelMessageResponse,
		ChannelResponse,
		DmMessageResponse,
		DmThreadResponse
	} from '$lib/api/client';
	import { apiClient } from '$lib/api/client';
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
	let addFriendUsername = $state('');
	let addFriendMessage = $state<string | null>(null);
	let addFriendError = $state<string | null>(null);
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
	type ServerChannel = { id: string; name: string };
	type ChatMessage = { id: string; userId: string; content: string; createdAt: number };

	let activeChat = $state<ChatView>('none');
	let selectedChannel = $state<ServerChannel | null>(null);
	let activeDmThreadId = $state<string | null>(null);
	let chatMessages = $state<ChatMessage[]>([]);
	let chatInput = $state('');
	let chatLoading = $state(false);
	let chatSending = $state(false);
	let chatError = $state<string | null>(null);

	let serverChannels = $state<Record<string, ServerChannel[]>>({});
	let dmThreadByFriendId = $state<Record<string, string>>({});
	let friendByDmThreadId = $state<Record<string, string>>({});
	let onlineUserIdsSnapshot = $state<string[]>([]);

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
					authError = 'Add meg a felhasználóneved/e-mail címed és a jelszavad.';
					return;
				}

				await login(loginIdentifier.trim(), password);
				return;
			}

			if (!username.trim()) {
				authError = 'A felhasználónév kötelező.';
				return;
			}

			if (!email.includes('@')) {
				authError = 'Adj meg egy érvényes e-mail címet.';
				return;
			}

			if (password.length < 6) {
				authError = 'A jelszónak legalább 6 karakteresnek kell lennie.';
				return;
			}

			if (password !== confirmPassword) {
				authError = 'A két jelszó nem egyezik.';
				return;
			}

			await register(username.trim(), email.trim().toLowerCase(), password);
		} catch (error) {
			authError = error instanceof Error ? error.message : 'Sikertelen hitelesítés.';
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
		}>;

		servers.set(
			data.map((item) => ({
				id: item.id,
				name: item.name,
				description: item.description ?? '',
				hasUnread: false
			}))
		);
	};

	const toChatMessage = (item: DmMessageResponse | ChannelMessageResponse): ChatMessage => ({
		id: item.id,
		userId: item.user_id,
		content: item.content,
		createdAt: item.created_at
	});

	const byCreatedAtAsc = (a: ChatMessage, b: ChatMessage) => a.createdAt - b.createdAt;

	const loadActiveMessages = async () => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		chatLoading = true;
		chatError = null;

		try {
			if (activeChat === 'friend' && activeDmThreadId) {
				const data = await apiClient.dmMessages(token, activeDmThreadId);
				chatMessages = data.map(toChatMessage).sort(byCreatedAtAsc);
				return;
			}

			if (activeChat === 'server' && selectedChannel?.id) {
				const data = await apiClient.channelMessages(token, selectedChannel.id);
				chatMessages = data.map(toChatMessage).sort(byCreatedAtAsc);
				return;
			}

			chatMessages = [];
		} catch (error) {
			chatError = error instanceof Error ? error.message : 'Az üzenetek betöltése sikertelen.';
		} finally {
			chatLoading = false;
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
			[serverId]: data.map((channel: ChannelResponse) => ({ id: channel.id, name: channel.name }))
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

			chatError = 'Nincs aktív chat kiválasztva.';
		} catch (error) {
			chatError = error instanceof Error ? error.message : 'Az üzenet küldése sikertelen.';
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
			addFriendError = 'Adj meg egy felhasználónevet.';
			return;
		}

		try {
			await sendFriendRequest(token, name);
			addFriendUsername = '';
			addFriendMessage = 'A barátkérelem elküldve.';
		} catch (error) {
			if (error instanceof Error) {
				if (error.message === 'not_found') {
					addFriendError = 'Nincs ilyen felhasználó.';
					return;
				}
				if (error.message.includes('already exists') || error.message.includes('request already exists')) {
					addFriendError = 'Ehhez a felhasználóhoz már van függő kérelem.';
					return;
				}
				addFriendError = error.message;
				return;
			}

			addFriendError = 'A barátkérelem küldése sikertelen.';
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
			pendingActionError = error instanceof Error ? error.message : 'A kérés feldolgozása sikertelen.';
		} finally {
			pendingActionInProgressId = null;
		}
	};

	const openFriendChat = async (friend: FriendEntry) => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		selectFriend(friend);
		selectedChannel = null;
		chatInput = '';
		chatMessages = [];
		chatError = null;
		activeChat = 'friend';
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
		} catch (error) {
			chatError = error instanceof Error ? error.message : 'A DM chat megnyitása sikertelen.';
		}
	};

	const openServerChannels = async (server: ServerEntry) => {
		selectServer(server);
		activeChat = 'server-channels';
		selectedChannel = null;
		activeDmThreadId = null;
		chatMessages = [];
		chatError = null;
		chatInput = '';

		try {
			await loadServerChannels(server.id);
		} catch (error) {
			chatError = error instanceof Error ? error.message : 'A csatornák betöltése sikertelen.';
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
	};

	const closeChat = () => {
		activeChat = 'none';
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

	const openPasswordModal = () => {
		passwordModalOpen = true;
		passwordChangeError = null;
		passwordChangeMessage = null;
		currentPasswordInput = '';
		newPasswordInput = '';
		newPasswordConfirmInput = '';
	};

	const handlePasswordChange = async () => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		passwordChangeError = null;
		passwordChangeMessage = null;

		if (!currentPasswordInput || !newPasswordInput || !newPasswordConfirmInput) {
			passwordChangeError = 'Minden mezőt tölts ki.';
			return;
		}

		if (newPasswordInput.length < 6) {
			passwordChangeError = 'Az új jelszónak legalább 6 karakteresnek kell lennie.';
			return;
		}

		if (newPasswordInput !== newPasswordConfirmInput) {
			passwordChangeError = 'Az új jelszó és a megerősítés nem egyezik.';
			return;
		}

		if (currentPasswordInput === newPasswordInput) {
			passwordChangeError = 'Az új jelszó nem lehet azonos a régivel.';
			return;
		}

		passwordChangeLoading = true;
		try {
			await apiClient.changeMyPassword(token, currentPasswordInput, newPasswordInput);
			passwordChangeMessage = 'A jelszó sikeresen módosítva.';
			currentPasswordInput = '';
			newPasswordInput = '';
			newPasswordConfirmInput = '';
		} catch (error) {
			if (error instanceof Error && error.message === 'unauthorized') {
				passwordChangeError = 'A jelenlegi jelszó hibás.';
			} else {
				passwordChangeError = error instanceof Error ? error.message : 'A jelszó módosítása sikertelen.';
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
			profileUploadMessage = 'Profilkép feltöltve.';
		} catch (error) {
			profileUploadError = error instanceof Error ? error.message : 'A profilkép feltöltése sikertelen.';
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
		addFriendUsername = '';
		addFriendError = null;
		addFriendMessage = null;
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
					await loadActiveMessages();
				}
				return;
			}

			if (
				(kind === 'channel.message.created' || kind === 'channel.message.updated' || kind === 'channel.message.deleted') &&
				activeChat === 'server' &&
				selectedChannel?.id &&
				payload.channel_id === selectedChannel.id
			) {
				await loadActiveMessages();
			}
		};

		const handleOutsideClick = (event: MouseEvent) => {
			if (!settingsMenuOpen || !settingsMenuEl) {
				return;
			}

			const target = event.target as Node | null;
			if (target && !settingsMenuEl.contains(target)) {
				settingsMenuOpen = false;
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
					<h2 class="text-xl font-semibold">{authMode === 'login' ? 'Belépés' : 'Regisztráció'}</h2>

					<div class="space-y-3">
						{#if authMode === 'login'}
							<label class="block w-full">
								<span class="label-text block mb-2">Felhasználónév vagy e-mail</span>
								<input class="input input-bordered w-full" bind:value={loginIdentifier} autocomplete="username" />
							</label>
						{:else}
							<label class="block w-full">
								<span class="label-text block mb-2">Felhasználónév</span>
								<input class="input input-bordered w-full" bind:value={username} autocomplete="username" />
							</label>
							<label class="block w-full">
								<span class="label-text block mb-2">E-mail</span>
								<input class="input input-bordered w-full" type="email" bind:value={email} autocomplete="email" />
							</label>
						{/if}

						<label class="block w-full">
							<span class="label-text block mb-2">Jelszó</span>
							<input class="input input-bordered w-full" type="password" bind:value={password} autocomplete="current-password" />
						</label>

						{#if authMode === 'register'}
							<label class="block w-full">
								<span class="label-text block mb-2">Jelszó újra</span>
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
							{authMode === 'login' ? 'Belépés' : 'Regisztráció'}
						</button>

						<button class="btn btn-ghost btn-sm w-full" type="button" onclick={switchAuthMode}>
							{authMode === 'login' ? 'Nincs fiókod? Regisztrálj' : 'Van fiókod? Lépj be'}
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
						<p class="text-sm text-slate-300 truncate">{$session.username ?? 'Felhasználó'}</p>
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
									<li><button type="button" onclick={handleOpenAvatarUpload}>Profilkép feltöltés</button></li>
									<li>
										<button
											type="button"
											onclick={() => {
												settingsMenuOpen = false;
												openPasswordModal();
											}}
										>
											Jelszó módosítás
										</button>
									</li>
									<li><button type="button" onclick={handleLogout}>Kijelentkezés</button></li>
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
							<button class="btn btn-primary btn-sm w-full" type="button" onclick={openAddFriendModal}>+ Barát hozzáadása</button>
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
						<ul class="menu rounded-box bg-slate-800/40 w-full">
							{#each $servers as item}
								<li>
									<button type="button" onclick={() => openServerChannels(item)}>
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
				<section class="h-full p-3 overflow-auto">
					<div class="h-14 border-b border-slate-700/60 flex items-center justify-between px-1 mb-3">
						<p class="font-semibold truncate">
							{#if $selectedServer}
								# {$selectedServer.name} csatornák
							{:else}
								Csatornák
							{/if}
						</p>
						<button class="btn btn-sm btn-ghost" onclick={backToServerList}>Vissza a szerverekhez</button>
					</div>

					<ul class="menu rounded-box bg-slate-800/40 w-full">
						{#if $selectedServer}
							{#if (serverChannels[$selectedServer.id] ?? []).length > 0}
								{#each serverChannels[$selectedServer.id] ?? [] as channel}
									<li>
										<button type="button" onclick={() => openServerChannelChat(channel)}>
											# {channel.name}
										</button>
									</li>
								{/each}
							{:else}
								<li><span class="text-slate-400">Nincsenek csatornák.</span></li>
							{/if}
						{:else}
							<li><span class="text-slate-400">Nincs kiválasztott szerver.</span></li>
						{/if}
					</ul>
				</section>
			{:else}
				<section class="h-full flex flex-col">
					<div class="h-14 border-b border-slate-700/60 flex items-center justify-between px-4 bg-slate-800/40">
						<p class="font-semibold truncate">
							{#if activeChat === 'friend'}
								{$selectedFriend?.username ?? 'Barát chat'}
							{:else}
								{#if $selectedServer}# {$selectedServer.name} / {/if}{selectedChannel?.name ?? 'Csatorna chat'}
							{/if}
						</p>
						<button class="btn btn-sm btn-ghost" onclick={closeChat}>Vissza</button>
					</div>

					<div class="flex-1 overflow-auto p-4 space-y-2">
						{#if chatLoading}
							<p class="text-sm text-slate-400">Üzenetek betöltése...</p>
						{:else if chatMessages.length === 0}
							<p class="text-sm text-slate-400">Még nincs üzenet.</p>
						{:else}
							{#each chatMessages as msg}
								<div class={`chat ${msg.userId === $session.userId ? 'chat-end' : 'chat-start'}`}>
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
								placeholder="Írj üzenetet..."
								bind:value={chatInput}
								onkeydown={(event) => {
									if (event.key === 'Enter' && !event.shiftKey) {
										event.preventDefault();
										void handleSendChatMessage();
									}
								}}
							/>
							<button class="btn join-item btn-primary" onclick={handleSendChatMessage} disabled={chatSending}>
								{chatSending ? 'Küldés...' : 'Küldés'}
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
							<h3 class="font-semibold">Jelszó módosítás</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { passwordModalOpen = false; }}>✕</button>
						</div>

						<label class="block w-full">
							<span class="label-text block mb-2">Jelenlegi jelszó</span>
							<input class="input input-bordered w-full" type="password" bind:value={currentPasswordInput} />
						</label>

						<label class="block w-full">
							<span class="label-text block mb-2">Új jelszó</span>
							<input class="input input-bordered w-full" type="password" bind:value={newPasswordInput} />
						</label>

						<label class="block w-full">
							<span class="label-text block mb-2">Új jelszó megerősítése</span>
							<input class="input input-bordered w-full" type="password" bind:value={newPasswordConfirmInput} />
						</label>

						{#if passwordChangeError}
							<div class="alert alert-error py-2 text-sm"><span>{passwordChangeError}</span></div>
						{/if}
						{#if passwordChangeMessage}
							<div class="alert alert-success py-2 text-sm"><span>{passwordChangeMessage}</span></div>
						{/if}

						<div class="flex justify-end gap-2">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { passwordModalOpen = false; }}>Mégse</button>
							<button class="btn btn-sm btn-primary" type="button" disabled={passwordChangeLoading} onclick={handlePasswordChange}>
								{passwordChangeLoading ? 'Mentés...' : 'Mentés'}
							</button>
						</div>
					</div>
				</div>
			{/if}

			{#if addFriendModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-sm rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Barát hozzáadása</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { addFriendModalOpen = false; }}>✕</button>
						</div>

						<label class="block w-full">
							<span class="label-text block mb-2">Felhasználónév</span>
							<input class="input input-bordered w-full" bind:value={addFriendUsername} />
						</label>

						{#if addFriendError}
							<div class="alert alert-error py-2 text-sm"><span>{addFriendError}</span></div>
						{/if}
						{#if addFriendMessage}
							<div class="alert alert-success py-2 text-sm"><span>{addFriendMessage}</span></div>
						{/if}

						<div class="flex gap-2 justify-end">
							<button class="btn btn-sm btn-ghost" type="button" onclick={() => { addFriendModalOpen = false; }}>Mégse</button>
							<button class="btn btn-sm btn-primary" type="button" onclick={handleCreateFriendRequest}>Küldés</button>
						</div>
					</div>
				</div>
			{/if}

			{#if pendingModalOpen}
				<div class="fixed inset-0 z-40 bg-black/60 flex items-center justify-center p-4">
					<div class="w-full max-w-md rounded-lg border border-slate-700 bg-slate-900 p-4 space-y-3 max-h-[80vh] overflow-auto">
						<div class="flex items-center justify-between">
							<h3 class="font-semibold">Pending kérelmek</h3>
							<button class="btn btn-ghost btn-xs" type="button" onclick={() => { pendingModalOpen = false; }}>✕</button>
						</div>

						{#if pendingActionError}
							<div class="alert alert-error py-2 text-sm"><span>{pendingActionError}</span></div>
						{/if}

						{#if $friendPending.length === 0}
							<p class="text-sm text-slate-400">Nincs pending kérelem.</p>
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
		</div>
	</main>
{/if}
