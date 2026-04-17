import { apiConfig } from '$lib/api/client';

type EventCallback = (event: { kind: string; payload: unknown }) => void;

export class WsClient {
	private socket: WebSocket | null = null;
	private callback: EventCallback | null = null;
	private accessToken: string | null = null;
	private shouldReconnect = false;
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	private reconnectAttempts = 0;
	private heartbeatTimer: ReturnType<typeof setInterval> | null = null;

	private clearReconnectTimer(): void {
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer);
			this.reconnectTimer = null;
		}
	}

	private clearHeartbeatTimer(): void {
		if (this.heartbeatTimer) {
			clearInterval(this.heartbeatTimer);
			this.heartbeatTimer = null;
		}
	}

	private scheduleReconnect(): void {
		if (!this.shouldReconnect || !this.accessToken || this.reconnectTimer) {
			return;
		}

		const delayMs = Math.min(10_000, 1_000 * Math.max(1, this.reconnectAttempts));
		this.reconnectTimer = setTimeout(() => {
			this.reconnectTimer = null;
			if (!this.shouldReconnect || !this.accessToken || !this.callback) {
				return;
			}
			this.connect(this.accessToken, this.callback);
		}, delayMs);
	}

	connect(accessToken: string, callback: EventCallback): void {
		this.clearReconnectTimer();
		this.clearHeartbeatTimer();
		this.shouldReconnect = true;
		this.accessToken = accessToken;
		this.callback = callback;

		if (this.socket) {
			this.socket.close();
		}

		const wsBase = apiConfig.baseUrl.replace('/api/v1', '').replace('http', 'ws');
		const socket = new WebSocket(`${wsBase}/api/v1/ws?token=${encodeURIComponent(accessToken)}`);
		this.socket = socket;

		socket.onopen = () => {
			if (this.socket !== socket) {
				return;
			}
			this.reconnectAttempts = 0;
			this.clearHeartbeatTimer();
			this.heartbeatTimer = setInterval(() => {
				if (this.socket === socket && socket.readyState === WebSocket.OPEN) {
					socket.send(JSON.stringify({ kind: 'ping' }));
				}
			}, 25_000);
		};

		socket.onmessage = (evt) => {
			if (this.socket !== socket) {
				return;
			}
			try {
				const parsed = JSON.parse(evt.data) as { kind: string; payload: unknown };
				this.callback?.(parsed);
			} catch {
				// Ignore malformed events.
			}
		};

		socket.onerror = () => {
			if (this.socket === socket) {
				socket.close();
			}
		};

		socket.onclose = () => {
			if (this.socket !== socket) {
				return;
			}
			this.clearHeartbeatTimer();
			this.socket = null;
			if (this.shouldReconnect) {
				this.reconnectAttempts += 1;
				this.scheduleReconnect();
			}
		};
	}

	disconnect(): void {
		this.shouldReconnect = false;
		this.accessToken = null;
		this.reconnectAttempts = 0;
		this.clearReconnectTimer();
		this.clearHeartbeatTimer();
		this.socket?.close();
		this.socket = null;
	}
}

export const wsClient = new WsClient();
