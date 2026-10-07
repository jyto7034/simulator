"""Local RFC6455 probe client adapted from the preserved U06 public host tool.
Protocol unchanged; handshake 64KiB, frame8MiB and total message16MiB caps.
"""
import base64
import hashlib
import json
import os
import socket
import struct

class WebSocket:
    """Small RFC6455 client for local, unauthenticated test service traffic."""
    def __init__(self, port):
        self.sock = socket.create_connection(("127.0.0.1", port), timeout=10)
        self.buffer = b""
        key = base64.b64encode(os.urandom(16)).decode()
        self.sock.sendall(f"GET /game HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: {key}\r\n\r\n".encode())
        while b"\r\n\r\n" not in self.buffer:
            self.buffer += self.sock.recv(4096)
            assert len(self.buffer) <= 65536, "bounded handshake"
        header, self.buffer = self.buffer.split(b"\r\n\r\n", 1)
        assert b"101 Switching Protocols" in header
        accept = base64.b64encode(hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest())
        assert accept in header

    def read(self, count):
        while len(self.buffer) < count:
            chunk = self.sock.recv(max(4096, count - len(self.buffer)))
            assert chunk, "unexpected websocket EOF"
            self.buffer += chunk
        result, self.buffer = self.buffer[:count], self.buffer[count:]
        return result

    def send_frame(self, opcode, body):
        mask = os.urandom(4)
        size = len(body)
        header = bytes([0x80 | opcode, 0x80 | (size if size < 126 else 126)])
        if size >= 126:
            header += struct.pack("!H", size)
        self.sock.sendall(header + mask + bytes(byte ^ mask[index % 4] for index, byte in enumerate(body)))

    def send(self, message):
        self.send_frame(1, json.dumps(message).encode())

    def receive(self):
        body = b""
        while True:
            first, second = self.read(2)
            assert not second & 0x80
            size = second & 0x7f
            if size == 126:
                size, = struct.unpack("!H", self.read(2))
            elif size == 127:
                size, = struct.unpack("!Q", self.read(8))
            assert size <= 8 * 1024 * 1024
            payload = self.read(size)
            if first & 15 == 9:
                self.send_frame(10, payload)
                continue
            assert first & 15 in (0, 1), first
            body += payload
            assert len(body) <= 16 * 1024 * 1024, "bounded complete websocket message"
            if first & 0x80:
                return json.loads(body)

    def close(self):
        self.sock.close()

