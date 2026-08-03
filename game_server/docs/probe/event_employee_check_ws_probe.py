#!/usr/bin/env python3
"""Actual game_server WebSocket probe for the live employee Event check contract."""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import socket
import struct
import time
from typing import Any, Callable


class ProbeFailure(RuntimeError):
    pass


class WebSocket:
    def __init__(self, host: str, port: int, path: str, timeout: float) -> None:
        self._socket = socket.create_connection((host, port), timeout=timeout)
        self._socket.settimeout(timeout)
        self._buffer = bytearray()
        key = base64.b64encode(os.urandom(16)).decode("ascii")
        request = (
            f"GET {path} HTTP/1.1\r\n"
            f"Host: {host}:{port}\r\n"
            "Upgrade: websocket\r\n"
            "Connection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\n"
            "Sec-WebSocket-Version: 13\r\n\r\n"
        ).encode("ascii")
        self._socket.sendall(request)
        header = self._read_http_header()
        lines = header.decode("latin-1").split("\r\n")
        if not lines[0].startswith("HTTP/1.1 101"):
            raise ProbeFailure(f"WebSocket upgrade failed: {lines[0]}")
        headers = {}
        for line in lines[1:]:
            if ":" in line:
                name, value = line.split(":", 1)
                headers[name.strip().lower()] = value.strip()
        expected_accept = base64.b64encode(
            hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()
        ).decode("ascii")
        if headers.get("sec-websocket-accept") != expected_accept:
            raise ProbeFailure("WebSocket accept key mismatch")

    def _read_http_header(self) -> bytes:
        while b"\r\n\r\n" not in self._buffer:
            chunk = self._socket.recv(4096)
            if not chunk:
                raise ProbeFailure("connection closed during WebSocket upgrade")
            self._buffer.extend(chunk)
        end = self._buffer.index(b"\r\n\r\n") + 4
        header = bytes(self._buffer[:end])
        del self._buffer[:end]
        return header

    def _read_exact(self, count: int) -> bytes:
        while len(self._buffer) < count:
            chunk = self._socket.recv(max(4096, count - len(self._buffer)))
            if not chunk:
                raise ProbeFailure("WebSocket connection closed unexpectedly")
            self._buffer.extend(chunk)
        result = bytes(self._buffer[:count])
        del self._buffer[:count]
        return result

    def _send_frame(self, opcode: int, payload: bytes = b"") -> None:
        mask = os.urandom(4)
        length = len(payload)
        header = bytearray([0x80 | opcode])
        if length < 126:
            header.append(0x80 | length)
        elif length <= 0xFFFF:
            header.append(0x80 | 126)
            header.extend(struct.pack("!H", length))
        else:
            header.append(0x80 | 127)
            header.extend(struct.pack("!Q", length))
        header.extend(mask)
        masked = bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))
        self._socket.sendall(bytes(header) + masked)

    def send_json(self, value: dict[str, Any]) -> None:
        self._send_frame(0x1, json.dumps(value, separators=(",", ":")).encode("utf-8"))

    def recv_json(self) -> dict[str, Any]:
        fragments = bytearray()
        while True:
            first, second = self._read_exact(2)
            final = (first & 0x80) != 0
            opcode = first & 0x0F
            masked = (second & 0x80) != 0
            length = second & 0x7F
            if length == 126:
                length = struct.unpack("!H", self._read_exact(2))[0]
            elif length == 127:
                length = struct.unpack("!Q", self._read_exact(8))[0]
            mask = self._read_exact(4) if masked else None
            payload = self._read_exact(length)
            if mask is not None:
                payload = bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload))
            if opcode == 0x8:
                raise ProbeFailure("server closed the WebSocket")
            if opcode == 0x9:
                self._send_frame(0xA, payload)
                continue
            if opcode == 0xA:
                continue
            if opcode not in (0x0, 0x1):
                continue
            fragments.extend(payload)
            if not final:
                continue
            try:
                value = json.loads(fragments.decode("utf-8"))
            except (UnicodeDecodeError, json.JSONDecodeError) as error:
                raise ProbeFailure(f"invalid JSON WebSocket payload: {error}") from error
            if not isinstance(value, dict):
                raise ProbeFailure("server WebSocket payload must be a JSON object")
            return value

    def close(self) -> None:
        try:
            self._send_frame(0x8, struct.pack("!H", 1000))
        except OSError:
            pass
        self._socket.close()

    def disconnect(self) -> None:
        """Simulate a transport loss without sending the player-level quit command."""
        try:
            self._socket.shutdown(socket.SHUT_RDWR)
        except OSError:
            pass
        self._socket.close()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ProbeFailure(message)


def receive_until(ws: WebSocket, predicate: Callable[[dict[str, Any]], bool]) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    observed = []
    for _ in range(30):
        message = ws.recv_json()
        observed.append(message)
        if predicate(message):
            return message, observed
    raise ProbeFailure(f"expected WebSocket message was not observed; got {observed!r}")


def authenticate(ws: WebSocket, player_id: str) -> dict[str, Any]:
    ws.send_json({"type": "auth", "player_id": player_id})
    authed, before_auth = receive_until(ws, lambda message: message.get("type") == "authed")
    require(authed.get("player_id") == player_id, "authed player identity mismatch")
    require(not before_auth[:-1], "unexpected message before authed")
    snapshot, after_auth = receive_until(ws, lambda message: message.get("type") == "state_snapshot")
    require(len(after_auth) == 1, "unexpected side message before initial state snapshot")
    return snapshot["state"]


def command(ws: WebSocket, request_id: str, behavior: dict[str, Any]) -> tuple[dict[str, Any], dict[str, Any]]:
    ws.send_json({"type": "command", "request_id": request_id, "behavior": behavior})
    result, before_result = receive_until(
        ws,
        lambda message: message.get("type") == "command_result"
        and message.get("request_id") == request_id,
    )
    require(len(before_result) == 1, f"{request_id}: command result was not first")
    require(result.get("ok") is True, f"{request_id}: command failed: {result!r}")
    snapshot, before_snapshot = receive_until(
        ws, lambda message: message.get("type") == "state_snapshot"
    )
    require(len(before_snapshot) == 1, f"{request_id}: state snapshot did not immediately follow result")
    return result, snapshot["state"]


def admin_enter_event(ws: WebSocket, token: str) -> tuple[dict[str, Any], dict[str, Any]]:
    request_id = "admin-enter-sealed-archive"
    ws.send_json(
        {
            "type": "admin_command",
            "request_id": request_id,
            "token": token,
            "admin": {
                "type": "admin_enter_event",
                "event_id": "sealed_archive_recovery_01",
            },
        }
    )
    result, before_result = receive_until(
        ws,
        lambda message: message.get("type") == "admin_result"
        and message.get("request_id") == request_id,
    )
    require(len(before_result) == 1, "admin result was not first")
    require(result.get("ok") is True, f"admin Event entry failed: {result!r}")
    require(result.get("result_type") == "AdminEnteredEvent", "unexpected admin result type")
    snapshot, before_snapshot = receive_until(
        ws, lambda message: message.get("type") == "state_snapshot"
    )
    require(len(before_snapshot) == 1, "admin state snapshot did not immediately follow result")
    return result, snapshot["state"]


def find_choice(state: dict[str, Any], choice_id: str) -> dict[str, Any]:
    interaction = state["selected_event"]["event"]["interaction"]
    require(interaction.get("type") == "employee_check_choices", "live Event is not a checked scene")
    for choice in interaction["choices"]:
        if choice.get("choice_id") == choice_id:
            return choice
    raise ProbeFailure(f"missing live Event choice {choice_id}")


def run(host: str, port: int, token: str, player_id: str, timeout: float) -> int:
    assertions = 0
    ws = WebSocket(host, port, "/game", timeout)
    try:
        initial = authenticate(ws, player_id)
        require(initial.get("game_state") == "not_started", "probe player must begin fresh")
        assertions += 2

        _, started = command(
            ws,
            "start-live-run",
            {"type": "start_new_game", "game_mode": "Standard"},
        )
        visual_novel = started.get("visual_novel")
        require(isinstance(visual_novel, dict), "run start must expose the briefing")
        _, after_briefing = command(
            ws,
            "finish-live-briefing",
            {
                "type": "advance_visual_novel",
                "session_id": visual_novel["session_id"],
                "current_scene_id": visual_novel["current_scene_id"],
            },
        )
        require(after_briefing.get("visual_novel") is None, "briefing must close")
        _, running = command(
            ws,
            "select-archive-starters",
            {
                "type": "select_starter_employees",
                "employee_definition_ids": [
                    "maintenance_tech",
                    "records_clerk",
                    "field_medic",
                ],
            },
        )
        require(running.get("game_state") == "viewing_map", "starter selection must start run")
        assertions += 3

        _, precheck = admin_enter_event(ws, token)
        selected = precheck["selected_event"]
        require(selected.get("type") == "event", "admin entry must select Event")
        event = selected["event"]
        require(event.get("event_id") == "sealed_archive_recovery_01", "wrong live Event")
        require(event.get("current_scene_id") == "archive_check", "wrong checked scene")
        require(event.get("committed_check") is None, "pre-check snapshot must not expose a roll")
        decode = find_choice(precheck, "decode_archive_seal")
        stabilize = find_choice(precheck, "stabilize_archive_power")
        require((decode["aptitude"], decode["dc"]) == ("Analysis", 16), "decode policy mismatch")
        require(decode.get("required_choice_key") == "sealed_archive_access", "decode key mismatch")
        require((stabilize["aptitude"], stabilize["dc"]) == ("Maintenance", 12), "stabilize policy mismatch")
        require(stabilize.get("required_choice_key") is None, "stabilize must not require a key")
        scanner_candidate = next(
            candidate
            for candidate in decode["employee_candidates"]
            if candidate.get("choice_key_evidence", {}).get("equipment_definition_id")
            == "archive_scanner"
        )
        require(scanner_candidate["eligibility"]["enabled"] is True, "scanner candidate must be enabled")
        require(scanner_candidate["modifier_total"] == 2, "scanner modifier must be +2")
        require(scanner_candidate["success_chance_percent"] == 35, "scanner chance must be 35%")
        missing_key = next(
            candidate
            for candidate in decode["employee_candidates"]
            if candidate["eligibility"].get("disabled_reason_code") == "missing_event_choice_key"
            and candidate["employee_modifier"] == 3
        )
        require(missing_key["success_chance_percent"] == 40, "matching analyst chance must remain 40%")
        assertions += 13

        node_id = selected["node_id"]
        employee_uuid = scanner_candidate["employee_uuid"]
        precheck_copy = json.loads(json.dumps(precheck, sort_keys=True))
    except Exception:
        ws.disconnect()
        raise

    old_ws = ws
    ws = WebSocket(host, port, "/game", timeout)
    try:
        reconnected = authenticate(ws, player_id)
        require(reconnected == precheck_copy, "reconnect must reproduce the exact pre-check state")
        assertions += 1
        old_ws.close()
        behavior = {
            "type": "resolve_event_choice_check",
            "node_id": node_id,
            "event_id": "sealed_archive_recovery_01",
            "current_scene_id": "archive_check",
            "choice_id": "decode_archive_seal",
            "employee_uuid": employee_uuid,
        }
        result, committed_state = command(ws, "resolve-live-archive", behavior)
        require(result.get("result_type") == "EventState", "resolve result type mismatch")
        committed = committed_state["selected_event"]["event"]["committed_check"]
        require(committed["choice_id"] == "decode_archive_seal", "committed choice mismatch")
        require(committed["employee_uuid"] == employee_uuid, "committed employee mismatch")
        require((committed["aptitude"], committed["dc"]) == ("Analysis", 16), "committed policy mismatch")
        require(committed["modifier_total"] == 2, "committed modifier mismatch")
        require(committed["total_roll"] == committed["raw_roll"] + 2, "committed total mismatch")
        require(
            committed["choice_key_evidence"]["equipment_definition_id"] == "archive_scanner",
            "committed key evidence mismatch",
        )
        require(committed["outcome"] in ("critical_failure", "failure", "success", "critical_success"), "invalid outcome")
        enkephalin_after = committed_state["resources"]["enkephalin"]
        assertions += 8

        duplicate_result, duplicate_state = command(ws, "duplicate-live-archive", behavior)
        require(duplicate_result.get("result_type") == "EventState", "duplicate result type mismatch")
        require(
            duplicate_state["selected_event"]["event"]["committed_check"] == committed,
            "duplicate must return exact committed provenance",
        )
        require(
            duplicate_state["resources"]["enkephalin"] == enkephalin_after,
            "duplicate must not grant reward twice",
        )
        assertions += 3
    finally:
        old_ws.close()
        ws.close()

    print(
        json.dumps(
            {
                "status": "PASS",
                "probe": "event_employee_check_ws_probe",
                "assertions": assertions,
                "event_id": "sealed_archive_recovery_01",
                "reconnect": True,
                "duplicate_reused": True,
            },
            sort_keys=True,
        )
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=18083)
    parser.add_argument("--admin-token", default="dev")
    parser.add_argument("--player-id", default="00000000-0000-0000-0000-00000000a11c")
    parser.add_argument("--timeout", type=float, default=10.0)
    args = parser.parse_args()
    try:
        return run(args.host, args.port, args.admin_token, args.player_id, args.timeout)
    except (OSError, KeyError, StopIteration, ProbeFailure) as error:
        print(json.dumps({"status": "FAIL", "error": str(error)}, sort_keys=True))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
