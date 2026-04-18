export type AppTab = 'friends' | 'servers';

export type FriendEntry = {
	id: string;
	username: string;
	avatarMime?: string | null;
	avatarUrl?: string | null;
	online: boolean;
	lastMessage: string;
	unread: number;
};

export type ServerEntry = {
	id: string;
	name: string;
	description: string;
	avatarMime?: string | null;
	avatarUrl?: string | null;
	hasUnread: boolean;
	createdAt?: number;
	memberCount?: number;
};

export type AuthSession = {
	accessToken: string | null;
	refreshToken: string | null;
	userId: string | null;
	username: string | null;
};
