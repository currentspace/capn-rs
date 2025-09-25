#!/usr/bin/env python3
"""
Minimal server to test Cap'n Web wire protocol with TypeScript client.
This will help isolate whether the issue is the protocol format or something else.
"""

import json
import logging
from http.server import BaseHTTPRequestHandler, HTTPServer

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class CapnWebHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path == '/rpc/batch':
            self.handle_batch_request()
        else:
            self.send_error(404, "Not Found")

    def do_GET(self):
        if self.path == '/health':
            self.handle_health()
        else:
            self.send_error(404, "Not Found")

    def handle_health(self):
        """Handle health check"""
        response = {
            "status": "healthy",
            "server": "minimal-capnweb-test",
            "protocol": "cap'n web wire protocol test"
        }

        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(response).encode())

    def handle_batch_request(self):
        """Handle Cap'n Web batch request"""
        logger.info("=== BATCH REQUEST ===")
        logger.info(f"Headers: {dict(self.headers)}")

        # Read request body
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length).decode('utf-8')

        logger.info(f"Raw body ({len(body)} chars):")
        logger.info(f"'{body}'")

        # Check if body contains newlines (wire protocol format)
        if '\n' in body:
            logger.info("📡 DETECTED: Newline-delimited format (official protocol)")
            lines = [line.strip() for line in body.split('\n') if line.strip()]
            logger.info(f"Found {len(lines)} lines:")

            for i, line in enumerate(lines):
                logger.info(f"  Line {i}: {line}")
                try:
                    parsed = json.loads(line)
                    logger.info(f"    Parsed: {parsed}")

                    # Check if it's a wire protocol array
                    if isinstance(parsed, list) and len(parsed) > 0:
                        msg_type = parsed[0]
                        logger.info(f"    Message type: {msg_type}")

                        # Handle specific message types
                        if msg_type == "push" and len(parsed) >= 2:
                            expr = parsed[1]
                            logger.info(f"    Push expression: {expr}")

                            # Check for pipeline expression
                            if isinstance(expr, list) and len(expr) > 0 and expr[0] == "pipeline":
                                import_id = expr[1] if len(expr) > 1 else None
                                method_path = expr[2] if len(expr) > 2 else []
                                args = expr[3] if len(expr) > 3 else []

                                logger.info(f"    Pipeline: import_id={import_id}, path={method_path}, args={args}")

                                # Simulate method call
                                if method_path and len(method_path) > 0:
                                    method = method_path[0]
                                    if method == "add" and len(args) == 2:
                                        result = args[0] + args[1]
                                        logger.info(f"    Simulated result: {result}")

                                        # Send back wire protocol response
                                        response = f'["resolve",-1,{result}]\n'
                                        logger.info(f"📤 Response: {response}")

                                        self.send_response(200)
                                        self.send_header('Content-Type', 'text/plain')
                                        self.end_headers()
                                        self.wfile.write(response.encode())
                                        return

                        elif msg_type == "pull":
                            import_id = parsed[1] if len(parsed) > 1 else None
                            logger.info(f"    Pull import_id: {import_id}")

                except json.JSONDecodeError as e:
                    logger.error(f"    JSON parse error: {e}")

            # Default response for wire protocol
            response = '["resolve",-1,"ok"]\n'
            logger.info(f"📤 Default response: {response}")

        else:
            logger.info("📋 DETECTED: Single JSON format (legacy)")
            try:
                parsed = json.loads(body)
                logger.info(f"Parsed JSON: {parsed}")
                response = json.dumps([{"result": "test"}])
            except json.JSONDecodeError as e:
                logger.error(f"JSON parse error: {e}")
                response = '["reject",-1,["error","bad_request","Invalid JSON"]]\n'

        # Send response
        self.send_response(200)
        self.send_header('Content-Type', 'text/plain')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'POST, GET, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.end_headers()
        self.wfile.write(response.encode())

    def do_OPTIONS(self):
        """Handle CORS preflight"""
        self.send_response(200)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'POST, GET, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.end_headers()

def run_server():
    server_address = ('127.0.0.1', 8080)
    httpd = HTTPServer(server_address, CapnWebHandler)

    print("🚀 Minimal Cap'n Web Test Server")
    print("===============================")
    print(f"Listening on http://{server_address[0]}:{server_address[1]}")
    print("Endpoints:")
    print("  POST /rpc/batch - Wire protocol test")
    print("  GET  /health    - Health check")
    print()
    print("This server will log exactly what the TypeScript client sends")
    print("and help us understand the protocol format.")
    print()

    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nServer stopped.")

if __name__ == '__main__':
    run_server()