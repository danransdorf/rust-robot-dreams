import { writable } from 'svelte/store';
import type { ServerResponse } from './types';




/* export const socketStore = writable<WebSocket | null>(null); */

export const connectWebsocket = (
	address: string | undefined,
	onopen: () => void,
	onmessage: (data: {data: string}) => void,
	onclose: () => void
) => {
	const ws = new WebSocket(address || 'ws://localhost:11111');
	ws.onopen = onopen;
	ws.onmessage = onmessage as (data: unknown) => void;
	ws.onclose = onclose;
	return ws;
};