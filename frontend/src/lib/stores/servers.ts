import { writable } from 'svelte/store';
import type { ServerEntry } from '$lib/types/domain';

const initialServers: ServerEntry[] = [];

export const servers = writable<ServerEntry[]>(initialServers);
export const selectedServer = writable<ServerEntry | null>(null);

export function selectServer(server: ServerEntry): void {
	selectedServer.set(server);
}
