# Uno Over WebSockets 🃏

Wunos is a Rust-based implementation of the card game Uno, designed to be played over a WebSocket. The project leverages the asynchronous capabilities of Rust to handle real-time communication between clients and the server, ensuring a **blazingly fast** gaming experience 🦀

## Technical Overview ⚙️

### Architecture 
Wunos is a WebSocket-based Uno game server implemented in Rust. The project is structured into multiple sub-projects:

1. **Server**: Manages game state and client communication.
2. **Client**: Connects to the server to participate in the game.
3. **Test Client**: Used for testing the server functionality.

### Tokio Runtime 🗼
The project leverages the Tokio runtime, an asynchronous runtime for the Rust programming language. Tokio enables efficient handling of multiple connections by using asynchronous I/O, allowing the server to manage numerous WebSocket connections concurrently without blocking.

### Warp Framework 🕸️
The Warp framework is used to build the WebSocket server. Warp is a highly performant, composable web server framework for Rust. It simplifies routing and handling of HTTP/WebSocket requests.

### Mutexes and RwLocks 🔐
- **Mutexes**: Used to ensure exclusive access to critical sections of the code where game state is modified. This prevents data races and ensures thread safety when multiple clients attempt to update the game state simultaneously.
- **RwLocks**: Employed for sections of the code where read access is more frequent than write access. RwLocks allow multiple readers or a single writer, optimizing performance by reducing contention.

### Sub-projects
- **Server**: Hosts the game logic and handles WebSocket connections. It uses `tokio::sync::Mutex` and `tokio::sync::RwLock` for managing the game state.
- **Client**: Provides a command-line interface for players to connect to the server and participate in the game.
- **Test Client**: Simulates multiple clients for testing purposes, uses a CLI based approach instead of the fancy TUI graphics.
  
Enjoy!
