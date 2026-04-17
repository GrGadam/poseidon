export type AppTab = 'friends' | 'servers';

export type FriendEntry = {
	id: string;
	username: string;
	online: boolean;
	lastMessage: string;
	unread: number;
};

export type ServerEntry = {
	id: string;
	name: string;
	description: string;
	hasUnread: boolean;
};

export type AuthSession = {
	accessToken: string | null;
	refreshToken: string | null;
	userId: string | null;
	username: string | null;
};
