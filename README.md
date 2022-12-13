# Ozes


Ozes is a queue message system... for learning purposes only.

Ozes run on top of tokio

## Roadmap

- [X] Muliple Consumers
- [X] Muliple Publishers
- [X] Muliple Queues
- [X] Muliple Queues With Groups
- [ ] Protocol:
    - [X] Create consumer
    - [X] Create publisher
    - [X] Ok
    - [X] Error
    - [ ] Create queue
    - [ ] Create group in queue
- [X] Support to send and receive binaries in messages.
- [ ] Improve way to read messages from clients
- [ ] Add graceful shutdown
- [X] Add len to message send to Ozes like "message +l17 #foo" check in [parser](https://github.com/pgjbz/ozes-parser)

Run project:

```bash
cargo run
```

Run tests:

```bash
cargo test
```

Run examples:

```bash
cargo run --example <example-name>
```
