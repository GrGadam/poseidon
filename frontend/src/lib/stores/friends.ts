import { derived, get, writable } from 'svelte/store';
import { apiClient, apiConfig } from '$lib/api/client';
import type { FriendEntry } from '$lib/types/domain';

const initialFriends: FriendEntry[] = [];

export type PendingRequestEntry = {
	requestId: string;
	userId: string;
	username: string;
	avatarMime: string | null;
	avatarUrl: string | null;
};

const pendingData: PendingRequestEntry[] = [];

export const friends = writable<FriendEntry[]>(initialFriends);
export const friendPending = writable<PendingRequestEntry[]>(pendingData);
export const selectedFriend = writable<FriendEntry | null>(null);

export const friendOnline = derived(friends, ($friends) => $friends.filter((f) => f.online));
export const friendOffline = derived(friends, ($friends) => $friends.filter((f) => !f.online));

export function selectFriend(friend: FriendEntry): void {
	selectedFriend.set(friend);
}

export async function refreshFriends(accessToken: string): Promise<void> {
	const data = (await apiClient.friends(accessToken)) as Array<{
		user: { id: string; username: string; avatar_mime?: string | null };
	}>;

	const previous = get(friends);
	const previousById = new Map(previous.map((entry) => [entry.id, entry]));
	const incomingIds = new Set(data.map((item) => item.user.id));

	for (const entry of previous) {
		if (!incomingIds.has(entry.id) && entry.avatarUrl?.startsWith('blob:')) {
			URL.revokeObjectURL(entry.avatarUrl);
		}
	}

	const friendsWithAvatars = await Promise.all(
		data.map(async (item) => {
			const avatarMime = item.user.avatar_mime ?? null;
			const prev = previousById.get(item.user.id);
			let avatarUrl = prev?.avatarUrl ?? null;

			if (prev?.avatarMime !== avatarMime) {
				if (prev?.avatarUrl?.startsWith('blob:')) {
					URL.revokeObjectURL(prev.avatarUrl);
				}
				avatarUrl = await loadAvatarUrl(accessToken, item.user.id, avatarMime);
			}

			return {
				id: item.user.id,
				username: item.user.username,
				avatarMime,
				avatarUrl,
				online: prev?.online ?? false,
				lastMessage: prev?.lastMessage ?? '',
				unread: prev?.unread ?? 0
			};
		})
	);

	friends.set(friendsWithAvatars);
}

export async function refreshPendingRequests(accessToken: string): Promise<void> {
	const data = (await apiClient.pendingRequests(accessToken)) as Array<{
		id: string;
		from_user: { id: string; username: string; avatar_mime?: string | null };
	}>;

	for (const entry of get(friendPending)) {
		if (entry.avatarUrl?.startsWith('blob:')) {
			URL.revokeObjectURL(entry.avatarUrl);
		}
	}

	const pendingWithAvatars = await Promise.all(
		data.map(async (item) => {
			const avatarMime = item.from_user.avatar_mime ?? null;
			const avatarUrl = await loadAvatarUrl(accessToken, item.from_user.id, avatarMime);

			return {
				requestId: item.id,
				userId: item.from_user.id,
				username: item.from_user.username,
				avatarMime,
				avatarUrl
			};
		})
	);

	friendPending.set(pendingWithAvatars);
}

async function loadAvatarUrl(
	accessToken: string,
	userId: string,
	avatarMime: string | null
): Promise<string | null> {
	if (!avatarMime) {
		return null;
	}

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
}

export async function sendFriendRequest(accessToken: string, username: string): Promise<void> {
	await apiClient.sendFriendRequest(accessToken, username);
}

export async function acceptFriendRequest(accessToken: string, requestId: string): Promise<void> {
	await apiClient.acceptFriendRequest(accessToken, requestId);
	await Promise.all([refreshPendingRequests(accessToken), refreshFriends(accessToken)]);
}

export async function rejectFriendRequest(accessToken: string, requestId: string): Promise<void> {
	await apiClient.rejectFriendRequest(accessToken, requestId);
	await refreshPendingRequests(accessToken);
}

export function setFriendOnline(userId: string, online: boolean): void {
	friends.update((items) =>
		items.map((item) => (item.id === userId ? { ...item, online } : item))
	);
}

export function setFriendLastMessage(userId: string, lastMessage: string): void {
	friends.update((items) =>
		items.map((item) => (item.id === userId ? { ...item, lastMessage } : item))
	);
}

export function incrementFriendUnread(userId: string): void {
	friends.update((items) =>
		items.map((item) =>
			item.id === userId ? { ...item, unread: item.unread + 1 } : item
		)
	);
}

export function clearFriendUnread(userId: string): void {
	friends.update((items) =>
		items.map((item) => (item.id === userId ? { ...item, unread: 0 } : item))
	);
}
