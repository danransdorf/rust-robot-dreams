import type { ServerResponse } from './types';

export const connect_ws = (
	address: string | undefined,
	onopen: () => void,
	onmessage: (data: { data: ServerResponse }) => void,
	onclose: () => void
) => {
	const ws = new WebSocket(address || 'ws://localhost:3000');
	ws.onopen = onopen;
	ws.onmessage = onmessage as (data: unknown) => void;
	ws.onclose = onclose;
	return ws;
};
