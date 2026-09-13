#!/usr/bin/env python3
"""Minimal request-counting OpenAI-compatible stub for Aura acceptance runs.

Why this exists: the milestone's P0 rule is "recalling the same text must not issue a new
request". That is a statement about how many HTTP requests leave the machine, so it can only be
verified by counting them. This stub streams a canned translation and records every request.

Usage:
    scripts/release/mock-translate-server.py --port 8787

Counting:
    GET /__count   -> {"count": N, "requests": [...]}
    POST /__reset  -> resets the counter
"""

from __future__ import annotations

import argparse
import json
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

REPLY = "你好，世界！"

_lock = threading.Lock()
_requests: list[dict] = []


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, fmt: str, *args) -> None:  # noqa: D102 - quieter test output
        pass

    def _json(self, status: int, payload: dict) -> None:
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:  # noqa: N802 - stdlib naming
        if self.path.startswith("/__count"):
            with _lock:
                self._json(200, {"count": len(_requests), "requests": list(_requests)})
            return
        self._json(404, {"error": "not found"})

    def do_POST(self) -> None:  # noqa: N802 - stdlib naming
        if self.path.startswith("/__reset"):
            with _lock:
                _requests.clear()
            self._json(200, {"count": 0})
            return

        length = int(self.headers.get("Content-Length") or 0)
        raw = self.rfile.read(length) if length else b"{}"
        try:
            payload = json.loads(raw.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError):
            payload = {}

        user_text = ""
        for message in payload.get("messages") or []:
            if message.get("role") == "user":
                user_text = message.get("content") or ""

        with _lock:
            _requests.append(
                {
                    "at": time.time(),
                    "model": payload.get("model"),
                    "stream": payload.get("stream"),
                    "text": user_text,
                }
            )

        if not payload.get("stream"):
            # Readiness probes ask for a non-streaming reply.
            self._json(200, {"choices": [{"message": {"role": "assistant", "content": "OK"}}]})
            return

        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        # Without a Content-Length the body must be framed, otherwise an HTTP/1.1 client waits
        # forever for a message boundary. Real providers stream chunked SSE, so match that.
        self.send_header("Transfer-Encoding", "chunked")
        self.end_headers()

        for piece in (REPLY[:2], REPLY[2:]):
            chunk = {"choices": [{"delta": {"content": piece}}]}
            self._write_chunk(f"data: {json.dumps(chunk, ensure_ascii=False)}\n\n".encode("utf-8"))
            time.sleep(0.02)

        self._write_chunk(b"data: [DONE]\n\n")
        self._write_chunk(b"")  # terminating zero-length chunk

    def _write_chunk(self, payload: bytes) -> None:
        """Writes one HTTP/1.1 chunked-encoding frame."""
        self.wfile.write(f"{len(payload):X}\r\n".encode("ascii"))
        self.wfile.write(payload)
        self.wfile.write(b"\r\n")
        self.wfile.flush()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=8787)
    parser.add_argument("--host", default="127.0.0.1")
    args = parser.parse_args()

    server = ThreadingHTTPServer((args.host, args.port), Handler)
    print(
        f"mock translate server on http://{args.host}:{args.port} "
        f"(count: /__count, reset: POST /__reset)",
        flush=True,
    )
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
