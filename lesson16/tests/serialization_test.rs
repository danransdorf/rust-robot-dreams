#[cfg(test)]
mod tests {
    use utils::{
        auth_request, db::structs::User, message, AuthRequest, AuthRequestKind, MessageContent,
        MessageResponse,
    };

    #[test]
    fn test_serialization() {
        use utils::{serialize_data, serialize_server_response, serialize_stream};

        let data = MessageContent::Text("Hello, world!".to_string());
        let data_bytes = vec![
            2, 0, 0, 0, 13, 0, 0, 0, 0, 0, 0, 0, 72, 101, 108, 108, 111, 44, 32, 119, 111, 114,
            108, 100, 33,
        ];

        assert_eq!(serialize_data(data).unwrap(), data_bytes);

        let server_response = message(MessageResponse {
            id: 123,
            user: User {
                id: Some(123),
                username: "test".to_string(),
                password: "randomhashedpassword".to_string(),
                salt: vec![0u8, 1u8, 2u8],
            },
            content: MessageContent::Text("Hello, world!".to_string()),
        });
        let server_response_bytes = vec![
            0, 0, 0, 0, 123, 0, 0, 0, 1, 123, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 116, 101, 115, 116,
            20, 0, 0, 0, 0, 0, 0, 0, 114, 97, 110, 100, 111, 109, 104, 97, 115, 104, 101, 100, 112,
            97, 115, 115, 119, 111, 114, 100, 3, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 0, 0, 0, 13, 0,
            0, 0, 0, 0, 0, 0, 72, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33,
        ];
        assert_eq!(
            serialize_server_response(server_response).unwrap(),
            server_response_bytes
        );

        let stream = auth_request(AuthRequest::new(
            AuthRequestKind::Register,
            "test".to_string(),
            "password".to_string(),
        ));
        let stream_bytes = vec![
            1, 0, 0, 0, 1, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 116, 101, 115, 116, 8, 0, 0, 0, 0, 0,
            0, 0, 112, 97, 115, 115, 119, 111, 114, 100,
        ];
        assert_eq!(serialize_stream(stream).unwrap(), stream_bytes);
    }
}
