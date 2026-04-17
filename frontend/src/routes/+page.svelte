<script lang="ts">
	import { onMount } from 'svelte';
	import { login, logout, register, session } from '$lib/stores/auth';
	import { selectedTab } from '$lib/stores/ui';
	import {
		acceptFriendRequest,
		friendPending,
		friendOnline,
		friendOffline,
		rejectFriendRequest,
		refreshFriends,
		refreshPendingRequests,
		sendFriendRequest,
		selectedFriend,
		selectFriend
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

	type ChatView = 'none' | 'friend' | 'server' | 'server-channels';
	type ServerChannel = { id: string; name: string };

	let activeChat = $state<ChatView>('none');
	let selectedChannel = $state<ServerChannel | null>(null);

	const serverChannels: Record<string, ServerChannel[]> = {};

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

	const refreshFriendsData = async () => {
		const token = $session.accessToken;
		if (!token) {
			return;
		}

		await Promise.all([refreshFriends(token), refreshPendingRequests(token)]);
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

	const openFriendChat = (friend: FriendEntry) => {
		selectFriend(friend);
		selectedChannel = null;
		activeChat = 'friend';
	};

	const openServerChannels = (server: ServerEntry) => {
		selectServer(server);
		activeChat = 'server-channels';
		selectedChannel = null;
	};

	const openServerChannelChat = (channel: ServerChannel) => {
		selectedChannel = channel;
		activeChat = 'server';
	};

	const closeChat = () => {
		activeChat = 'none';
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

	const handleLogout = () => {
		activeChat = 'none';
		selectedChannel = null;
		addFriendModalOpen = false;
		pendingModalOpen = false;
		addFriendUsername = '';
		addFriendError = null;
		addFriendMessage = null;
		pendingActionError = null;
		lastLoadedToken = null;
		logout();
	};

	onMount(() => {
		const handler = async (event: Event) => {
			const custom = event as CustomEvent<{ kind: string; payload: unknown }>;
			const kind = custom.detail?.kind;

			if (
				kind === 'friend.request.created' ||
				kind === 'friend.request.accepted' ||
				kind === 'friend.request.rejected' ||
				kind === 'friend.deleted'
			) {
				await refreshFriendsData();
			}
		};

		window.addEventListener('poseidon:ws-event', handler as EventListener);
		return () => window.removeEventListener('poseidon:ws-event', handler as EventListener);
	});

	$effect(() => {
		const token = $session.accessToken;
		if (!token) {
			lastLoadedToken = null;
			return;
		}

		if (lastLoadedToken !== token) {
			lastLoadedToken = token;
			void refreshFriendsData();
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
			<div class="h-12 border-b border-slate-700/60 px-3 flex items-center justify-between bg-slate-900/70">
				<p class="text-sm text-slate-300 truncate">{$session.username ?? 'Felhasználó'}</p>
				<button class="btn btn-xs btn-outline" type="button" onclick={handleLogout}>Kijelentkezés</button>
			</div>

			<div class="flex-1 min-h-0">
			{#if activeChat === 'none'}
				<section class="h-full p-3 overflow-auto">
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
										<div class="flex-1">
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
										<div class="flex-1">
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

					<div class="flex-1 overflow-auto p-4"></div>

					<div class="border-t border-slate-700/60 p-3 bg-slate-800/50">
						<div class="join w-full">
							<input class="input input-bordered join-item flex-1" placeholder="Írj üzenetet..." />
							<button class="btn join-item btn-primary">Küldés</button>
						</div>
					</div>
				</section>
			{/if}
			</div>

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
