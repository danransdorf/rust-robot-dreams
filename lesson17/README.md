## Install probably necessary stuff
### [SQLite](https://www.sqlite.org/index.html)

## Server
I left a prepared database for some testing here. It has a user with username/password test/test. I sent 7 messages in total, and one file.

### Run server (dev)
```bash
cargo run --bin server
```


## Client
After running client, you can login to the test user with username `test` and password `test`. Then you can replay the previous chat by sending `.history 7` <= Expect 7 messages ending with `test: sent a file Cargo.toml`.
Client, who is not logged in or who has an expired token, won't receive any messages.
### Run client (dev)
```bash
cargo run --bin client
```

### Built executable args
```bash
--port <port> --hostname <hostname>
```
