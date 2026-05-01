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
	console.log('[SESSION STORE] Session changed:', value);
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
	console.log('[AUTH] updateSession called with user:', data.user);
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
	console.log('[AUTH] updateSession completed');
}

export async function login(username: string, password: string): Promise<void> {
	console.log('[AUTH] login called with username:', username);
	const data = await apiClient.login(username, password);
	console.log('[AUTH] login API response:', data);
	updateSession(data as { access_token: string; refresh_token: string; user: { id: string; username: string } });
}

export async function register(username: string, email: string, password: string): Promise<void> {
	console.log('[AUTH] register called with username:', username, 'email:', email);
	const data = await apiClient.register(username, email, password);
	console.log('[AUTH] register API response:', data);
	updateSession(data as { access_token: string; refresh_token: string; user: { id: string; username: string } });
}

export async function logout(): Promise<void> {
	const current = get(session);
	if (current.accessToken) {
		try {
			await apiClient.logout(current.accessToken);
		} catch {
			// Ignore logout API errors; proceed with local cleanup
		}
	}

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
