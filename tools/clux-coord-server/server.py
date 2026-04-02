"""
clux-coord-server: MCP-based agent coordination server for clux-term.

A single-file JSON-RPC 2.0 over HTTP server implementing the MCP protocol
for coordinating multiple Claude Code worker agents.

Usage:
    python server.py [--port PORT]
"""

import argparse
import json
import sys
import time
import threading
from http.server import HTTPServer, BaseHTTPRequestHandler

PROTOCOL_VERSION = "2025-03-26"
SERVER_NAME = "clux-coord-server"
SERVER_VERSION = "0.1.0"

# In-memory storage, guarded by a lock for thread safety.
lock = threading.Lock()
workers = {}       # worker_id -> WorkerInfo dict
permissions = {}   # pane_id (int) -> PermissionPrompt dict


def make_worker_info(worker_id, task_description, cwd):
    return {
        "worker_id": worker_id,
        "task_description": task_description,
        "cwd": cwd,
        "status": "running",
        "registered_at": int(time.time()),
        "result": None,
    }


def make_task_result(status, summary, details=None):
    return {
        "status": status,
        "summary": summary,
        "details": details,
        "reported_at": int(time.time()),
    }


def make_permission_prompt(pane_id, text):
    return {
        "worker_pane_id": pane_id,
        "prompt_text": text,
        "detected_at": int(time.time()),
    }


# ---------------------------------------------------------------------------
# MCP tool implementations
# ---------------------------------------------------------------------------

def tool_register_worker(params):
    worker_id = params["worker_id"]
    task_description = params["task_description"]
    cwd = params["cwd"]
    with lock:
        workers[worker_id] = make_worker_info(worker_id, task_description, cwd)
    return "Worker '%s' registered successfully" % worker_id


def tool_report_result(params):
    worker_id = params["worker_id"]
    status = params["status"]
    summary = params["summary"]
    details = params.get("details")
    with lock:
        if worker_id not in workers:
            return "Error: worker '%s' not found" % worker_id
        workers[worker_id]["status"] = status
        workers[worker_id]["result"] = make_task_result(status, summary, details)
    return "Result reported for worker '%s'" % worker_id


def tool_list_workers(_params):
    with lock:
        worker_list = list(workers.values())
    return json.dumps(worker_list, indent=2)


def tool_check_permissions(_params):
    with lock:
        prompt_list = list(permissions.values())
    return json.dumps(prompt_list, indent=2)


# ---------------------------------------------------------------------------
# Tool registry
# ---------------------------------------------------------------------------

TOOLS = [
    {
        "name": "clux_register_worker",
        "description": "Register a new worker agent",
        "inputSchema": {
            "type": "object",
            "properties": {
                "worker_id": {"type": "string", "description": "Unique worker identifier"},
                "task_description": {"type": "string", "description": "Description of the task"},
                "cwd": {"type": "string", "description": "Working directory of the worker"},
            },
            "required": ["worker_id", "task_description", "cwd"],
        },
    },
    {
        "name": "clux_report_result",
        "description": "Report the result of a worker's task",
        "inputSchema": {
            "type": "object",
            "properties": {
                "worker_id": {"type": "string", "description": "Worker identifier"},
                "status": {
                    "type": "string",
                    "enum": ["completed", "failed"],
                    "description": "Task outcome",
                },
                "summary": {"type": "string", "description": "Brief summary of the result"},
                "details": {"type": "string", "description": "Optional detailed information"},
            },
            "required": ["worker_id", "status", "summary"],
        },
    },
    {
        "name": "clux_list_workers",
        "description": "List all registered workers and their status",
        "inputSchema": {
            "type": "object",
            "properties": {},
        },
    },
    {
        "name": "clux_check_permissions",
        "description": "Check for pending permission prompts from workers",
        "inputSchema": {
            "type": "object",
            "properties": {},
        },
    },
]

TOOL_DISPATCH = {
    "clux_register_worker": tool_register_worker,
    "clux_report_result": tool_report_result,
    "clux_list_workers": tool_list_workers,
    "clux_check_permissions": tool_check_permissions,
}


# ---------------------------------------------------------------------------
# JSON-RPC helpers
# ---------------------------------------------------------------------------

def jsonrpc_success(id, result):
    return {"jsonrpc": "2.0", "id": id, "result": result}


def jsonrpc_error(id, code, message):
    return {"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}}


def mcp_text_result(text):
    return {"content": [{"type": "text", "text": text}]}


# ---------------------------------------------------------------------------
# MCP method handlers
# ---------------------------------------------------------------------------

def handle_initialize(id, _params):
    result = {
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": {"tools": {}},
        "serverInfo": {"name": SERVER_NAME, "version": SERVER_VERSION},
    }
    return jsonrpc_success(id, result)


def handle_tools_list(id, _params):
    return jsonrpc_success(id, {"tools": TOOLS})


def handle_tools_call(id, params):
    name = params.get("name", "")
    arguments = params.get("arguments", {})
    handler = TOOL_DISPATCH.get(name)
    if handler is None:
        return jsonrpc_error(id, -32602, "Unknown tool: %s" % name)
    try:
        text = handler(arguments)
    except KeyError as e:
        return jsonrpc_error(id, -32602, "Missing required parameter: %s" % e)
    except Exception as e:
        return jsonrpc_error(id, -32603, "Internal error: %s" % e)
    return jsonrpc_success(id, mcp_text_result(text))


MCP_METHODS = {
    "initialize": handle_initialize,
    "tools/list": handle_tools_list,
    "tools/call": handle_tools_call,
}


# ---------------------------------------------------------------------------
# HTTP request handler
# ---------------------------------------------------------------------------

class MCPHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        # Suppress default access log to keep stderr clean.
        pass

    def _send_json(self, status, body):
        payload = json.dumps(body).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def _read_body(self):
        length = int(self.headers.get("Content-Length", 0))
        return self.rfile.read(length)

    # POST /mcp  -- MCP JSON-RPC endpoint
    # POST /internal/permission  -- internal permission API
    def do_POST(self):
        if self.path == "/mcp":
            self._handle_mcp()
        elif self.path == "/internal/permission":
            self._handle_internal_permission()
        else:
            self._send_json(404, {"error": "not found"})

    def _handle_mcp(self):
        try:
            raw = self._read_body()
            request = json.loads(raw)
        except (json.JSONDecodeError, ValueError):
            self._send_json(400, jsonrpc_error(None, -32700, "Parse error"))
            return

        method = request.get("method", "")
        id = request.get("id")
        params = request.get("params", {})

        handler = MCP_METHODS.get(method)
        if handler is None:
            self._send_json(200, jsonrpc_error(id, -32601, "Method not found: %s" % method))
            return

        response = handler(id, params)
        self._send_json(200, response)

    def _handle_internal_permission(self):
        try:
            raw = self._read_body()
            body = json.loads(raw)
        except (json.JSONDecodeError, ValueError):
            self._send_json(400, {"error": "invalid JSON"})
            return

        action = body.get("action", "")

        if action == "register":
            pane_id = body.get("pane_id")
            text = body.get("text", "")
            if pane_id is None:
                self._send_json(400, {"error": "pane_id is required"})
                return
            pane_id = int(pane_id)
            with lock:
                permissions[pane_id] = make_permission_prompt(pane_id, text)
            self._send_json(200, {"ok": True})

        elif action == "clear":
            pane_id = body.get("pane_id")
            if pane_id is None:
                self._send_json(400, {"error": "pane_id is required"})
                return
            pane_id = int(pane_id)
            with lock:
                permissions.pop(pane_id, None)
            self._send_json(200, {"ok": True})

        else:
            self._send_json(400, {"error": "unknown action: %s" % action})


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="clux-coord-server: MCP coordination server")
    parser.add_argument("--port", type=int, default=0, help="Port to bind (default: OS-assigned)")
    args = parser.parse_args()

    server = HTTPServer(("127.0.0.1", args.port), MCPHandler)
    actual_port = server.server_address[1]
    print("clux-coord-server listening on 127.0.0.1:%d" % actual_port, file=sys.stderr)
    sys.stderr.flush()

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
