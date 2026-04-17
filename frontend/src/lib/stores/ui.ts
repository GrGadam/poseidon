import { writable } from 'svelte/store';
import type { AppTab } from '$lib/types/domain';

export const selectedTab = writable<AppTab>('friends');
