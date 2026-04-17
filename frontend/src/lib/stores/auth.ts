import { get, writable } from 'svelte/store';
import { apiClient } from '$lib/api/client';
import { wsClient } from '$lib/api/ws';
import type { AuthSession } from '$lib/types/domain';

type RealtimeEvent = { kind: string; payload: unknown };

const SESSION_KEY = 'poseidon_session';

const emptySession = (): AuthSession => ({
	accessToken: null,
	refreshToken: null,
	userId: null,
	username: null
});

const storage = (): Storage | null => {
	if (typeof window === 'undefined') {
		return null;
	}

	const candidate = window.localStorage as Partial<Storage> | undefined;
	if (!candidate) {
		return null;
	}

	if (typeof candidate.getItem !== 'function' || typeof candidate.setItem !== 'function') {
		return null;
	}

	return candidate as Storage;
};

const initialSession = (): AuthSession => {
	const s = storage();
	if (!s) {
		return emptySession();
	}

	const raw = s.getItem(SESSION_KEY);
	if (!raw) {
		return emptySession();
	}

	try {
		return JSON.parse(raw) as AuthSession;
	} catch {
		return emptySession();
	}
};

export const session = writable<AuthSession>(initialSession());

session.subscribe((value) => {
	const s = storage();
	if (!s) {
		return;
	}

	try {
		s.setItem(SESSION_KEY, JSON.stringify(value));
	} catch {
		// Ignore storage write errors in non-standard runtimes.
	}
});

function updateSession(data: {
	access_token: string;
	refresh_token: string;
	user: { id: string; username: string };
}): void {
	session.set({
		accessToken: data.access_token,
		refreshToken: data.refresh_token,
		userId: data.user.id,
		username: data.user.username
	});

	wsClient.connect(data.access_token, (event: RealtimeEvent) => {
		if (typeof window !== 'undefined') {
			window.dispatchEvent(new CustomEvent<RealtimeEvent>('poseidon:ws-event', { detail: event }));
		}
	});
}

export async function login(username: string, password: string): Promise<void> {
	const data = await apiClient.login(username, password);
	updateSession(data as { access_token: string; refresh_token: string; user: { id: string; username: string } });
}

export async function register(username: string, email: string, password: string): Promise<void> {
	const data = await apiClient.register(username, email, password);
	updateSession(data as { access_token: string; refresh_token: string; user: { id: string; username: string } });
}

export function logout(): void {
	wsClient.disconnect();
	session.set({ accessToken: null, refreshToken: null, userId: null, username: null });
}

export async function restoreSession(): Promise<void> {
	const current = get(session);
	if (!current.refreshToken) {
		return;
	}

	try {
		const refreshed = (await apiClient.refresh(current.refreshToken)) as {
			access_token: string;
			refresh_token: string;
			user: { id: string; username: string };
		};
		updateSession(refreshed);
	} catch {
		logout();
	}
}
