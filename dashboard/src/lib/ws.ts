import type { RealtimeData } from './types';

export function createWsConnection(
	onMessage: (data: RealtimeData) => void,
) {
	const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
	const ws = new WebSocket(`${protocol}//${location.host}/ws`);

	ws.onmessage = (event) => {
		try {
			const data: RealtimeData = JSON.parse(event.data);
			onMessage(data);
		} catch (e) {
			console.error('ws parse error', e);
		}
	};

	ws.onclose = () => {
		setTimeout(() => createWsConnection(onMessage), 2000);
	};

	return ws;
}
