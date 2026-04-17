import { apiConfig } from '$lib/api/client';

type EventCallback = (event: { kind: string; payload: unknown }) => void;

export class WsClient {
	private socket: WebSocket | null = null;
	private callback: EventCallback | null = null;

	connect(accessToken: string, callback: EventCallback): void {
		this.callback = callback;
		const wsBase = apiConfig.baseUrl.replace('/api/v1', '').replace('http', 'ws');
		this.socket = new WebSocket(`${wsBase}/api/v1/ws?token=${encodeURIComponent(accessToken)}`);

		this.socket.onmessage = (evt) => {
			try {
				const parsed = JSON.parse(evt.data) as { kind: string; payload: unknown };
				this.callback?.(parsed);
			} catch {
				// Ignore malformed events.
			}
		};
	}

	disconnect(): void {
		this.socket?.close();
		this.socket = null;
	}
}

export const wsClient = new WsClient();
