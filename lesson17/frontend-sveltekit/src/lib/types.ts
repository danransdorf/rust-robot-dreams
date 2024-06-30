type Vec<T> = Array<T>;
type Option<T> = T | null;

export type MessageContent = { Image: Vec<number> } | 'File' | 'Text';

export interface User {
	//** Option<i32> */
	id: Option<number>;
	username: string;
	password: string;
	//** Vec<u8> */
	salt: Vec<number>;
}

export interface MessageResponse {
	id: number;
	user: User;
	content: MessageContent;
}

export type ServerResponse = { Message: MessageResponse } | { AuthToken: string };
export interface MessageRequest {
	jwt: string;
	message: MessageContent;
}
export type AuthRequestKind = 'Login' | 'Register';
export interface AuthRequest {
	kind: AuthRequestKind;
	username: string;
	password: string;
}
export interface ReadRequest {
	jwt: string;
	amount: number;
}
export type StreamRequest =
	| { MessageRequest: MessageRequest }
	| { AuthRequest: AuthRequest }
	| { ReadRequest: ReadRequest };
