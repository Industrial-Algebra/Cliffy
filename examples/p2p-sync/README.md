# P2P Sync Demo

Real-time peer-to-peer state synchronization using WebRTC. This example demonstrates Cliffy's distributed state capabilities with direct browser-to-browser connections.

> **Note**: This example requires running a local signaling server and is **not deployed to Netlify**. WebRTC signaling requires a WebSocket server that cannot run on static hosting.

## Features

- Real WebRTC peer-to-peer data channels
- Automatic peer discovery via signaling server
- Delta-based state synchronization
- Vector clock causal ordering
- Binary message encoding for efficiency
- Connection state monitoring and RTT tracking

## Quick Start

### 1. Start the Signaling Server

From the `examples/` directory:

```bash
npm run signaling
```

This starts the WebSocket signaling server on `ws://localhost:8080`.

### 2. Start the P2P Demo

In another terminal, from `examples/`:

```bash
npm run dev:p2p
```

This starts the Vite dev server, typically on `http://localhost:5173`.

### 3. Test P2P Sync

1. Open the demo URL in **two browser tabs** (or two different browsers)
2. Click "Connect" in both tabs
3. Wait for peers to discover each other (you'll see them in the "Connected Peers" panel)
4. Click "Increment (+1)" in one tab
5. Watch the counter sync to the other tab in real-time

## Architecture

```
┌─────────────────┐           ┌─────────────────┐
│   Browser A     │           │   Browser B     │
│                 │           │                 │
│  ┌───────────┐  │   WebRTC  │  ┌───────────┐  │
│  │ P2P Demo  │◄─┼───────────┼─►│ P2P Demo  │  │
│  └───────────┘  │  (direct) │  └───────────┘  │
│        │        │           │        │        │
└────────┼────────┘           └────────┼────────┘
         │                             │
         │  WebSocket                  │  WebSocket
         │  (signaling)                │  (signaling)
         │                             │
         ▼                             ▼
    ┌─────────────────────────────────────┐
    │        Signaling Server             │
    │    (peer discovery & relay)         │
    │         ws://localhost:8080         │
    └─────────────────────────────────────┘
```

### Connection Flow

1. **Join Room**: Browser connects to signaling server and joins a room
2. **Peer Discovery**: Signaling server notifies about other peers in the room
3. **WebRTC Handshake**: Peers exchange SDP offers/answers via signaling
4. **ICE Candidates**: NAT traversal candidates exchanged via signaling
5. **Data Channel**: Direct peer-to-peer connection established
6. **State Sync**: Sync messages flow directly between browsers

## Components

### Signaling Server (`../signaling-server/`)

WebSocket server that handles:
- Room management (join/leave)
- Peer discovery (notifies when peers join/leave)
- WebRTC signaling relay (offer/answer/ICE candidates)

### WebRTC Transport (`../shared/src/webrtc/transport.ts`)

Low-level WebRTC wrapper:
- RTCPeerConnection management
- Data channel creation and handling
- Binary message encoding/decoding

### Peer Manager (`../shared/src/webrtc/peer-manager.ts`)

High-level integration layer:
- Combines transport and signaling
- Handles peer lifecycle (Hello, Heartbeat, Goodbye)
- Vector clock management
- RTT estimation

### P2P Demo (`src/main.ts`)

Application layer demonstrating:
- Shared counter state
- Delta broadcasting
- Full state synchronization
- Connection UI and statistics

## Why Local Only?

WebRTC requires a signaling mechanism to exchange connection information before peers can connect directly. This signaling typically uses WebSockets, which require a server-side component.

Options for production deployment:
1. **Self-hosted signaling**: Deploy the signaling server to a cloud provider (Fly.io, Railway, etc.)
2. **Serverless WebSockets**: Use services like AWS API Gateway WebSocket or Cloudflare Durable Objects
3. **Third-party signaling**: Use services like PeerJS Cloud or similar

## Configuration

The demo uses these defaults in `src/main.ts`:

```typescript
const SIGNALING_SERVER_URL = 'ws://localhost:8080';
const ROOM_ID = 'cliffy-p2p-demo';
```

To use a deployed signaling server, update `SIGNALING_SERVER_URL` to point to your server.

## Troubleshooting

### "Connection failed" or peers don't connect

1. Ensure the signaling server is running (`npm run signaling`)
2. Check browser console for WebSocket errors
3. Try refreshing both tabs and reconnecting

### Counter doesn't sync

1. Verify both peers show as "Connected" in the peer list
2. Check for errors in browser console
3. Ensure both tabs are connected to the same signaling server

### High latency

WebRTC connections go through STUN/TURN servers for NAT traversal. The demo uses Google's public STUN servers. For production:
- Consider adding TURN servers for restrictive firewalls
- Deploy signaling server closer to users

## Development

### Project Structure

```
p2p-sync/
├── src/
│   └── main.ts       # Demo application
├── index.html        # HTML entry point
├── package.json
├── tsconfig.json
└── vite.config.ts
```

### Dependencies

The demo depends on:
- `@industrialalgebra/cliffy-core` - Cliffy WASM core (Behavior, CRDT, VectorClock)
- `@cliffy/shared` - Shared WebRTC utilities

### Scripts

```bash
npm run dev      # Start dev server
npm run build    # Build for production
npm run preview  # Preview production build
```

## Related Examples

- **whiteboard** - Collaborative drawing (simulated sync)
- **document-editor** - Real-time text editing (simulated sync)
- **crdt-playground** - CRDT operations visualization

## License

MIT
