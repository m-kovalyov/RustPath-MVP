"""Private execution service. Requires a dedicated Docker host; never expose publicly."""
import base64
import hmac
import json
import os
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import docker
from docker.errors import DockerException

SECRET = os.environ['RUNNER_SECRET']
IMAGE = os.environ.get('RUNNER_IMAGE', 'rust:1.99.0-slim-bookworm')
TIMEOUT = int(os.environ.get('RUNNER_TIMEOUT', '20'))
client = docker.from_env()
slots = threading.BoundedSemaphore(2)
COMMAND = ['sh', '-c', 'printf "%s" "$SOURCE_B64" | base64 -d > /work/main.rs; unset SOURCE_B64; rustc --edition=2024 --test /work/main.rs -o /work/tests && /work/tests --test-threads=1']

class Handler(BaseHTTPRequestHandler):
    def reply(self, status, payload):
        raw = json.dumps(payload, ensure_ascii=False).encode()
        self.send_response(status)
        self.send_header('Content-Type', 'application/json; charset=utf-8')
        self.send_header('Content-Length', str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def do_POST(self):
        if self.path != '/evaluate':
            return self.reply(404, {'error': 'not found'})
        if not hmac.compare_digest(self.headers.get('Authorization', ''), 'Bearer '+SECRET):
            return self.reply(401, {'error': 'unauthorized'})
        try:
            length = int(self.headers.get('Content-Length', '0'))
            if length <= 0 or length > 32768:
                return self.reply(413, {'error': 'source too large'})
            source = json.loads(self.rfile.read(length))['source']
            if not isinstance(source, str) or len(source.encode()) > 24000:
                return self.reply(400, {'error': 'invalid source'})
        except (ValueError, KeyError, TypeError):
            return self.reply(400, {'error': 'invalid JSON'})
        if not slots.acquire(blocking=False):
            return self.reply(503, {'error': 'busy'})
        container = None
        started = time.monotonic()
        try:
            container = client.containers.create(
                IMAGE, COMMAND, network_mode='none',
                user='65534:65534', read_only=True,
                environment={'SOURCE_B64': base64.b64encode(source.encode()).decode()},
                tmpfs={'/work':'rw,exec,nosuid,size=256m,mode=1777', '/tmp':'rw,noexec,nosuid,size=64m,mode=1777'},
                cap_drop=['ALL'], security_opt=['no-new-privileges:true'],
                mem_limit='512m', memswap_limit='512m', nano_cpus=1_000_000_000,
                pids_limit=64, working_dir='/work',
                log_config=docker.types.LogConfig(type='json-file',config={'max-size':'64k','max-file':'1'}),
                labels={'app':'rustpath-sandbox'},
                ulimits=[docker.types.Ulimit(name='nofile',soft=128,hard=128),docker.types.Ulimit(name='fsize',soft=33554432,hard=33554432)],
            )
            container.start()
            result = container.wait(timeout=TIMEOUT)
            output = container.logs().decode('utf-8', errors='replace')[:12000]
            # Exit(0) alone is not evidence that the test harness ran.
            passed = result['StatusCode'] == 0 and 'test result: ok.' in output
            self.reply(200, {'passed':passed,'output':output,'duration_ms':int((time.monotonic()-started)*1000)})
        except Exception as error:
            if container is not None:
                try:
                    container.kill()
                except DockerException:
                    pass
            if isinstance(error, DockerException):
                self.reply(503, {'error':'sandbox infrastructure unavailable'})
            else:
                self.reply(200, {'passed':False,'output':'Превышено время проверки. Упростите код и проверьте циклы.','duration_ms':int((time.monotonic()-started)*1000)})
        finally:
            if container is not None:
                try:
                    container.remove(force=True)
                except DockerException:
                    pass
            slots.release()

    def log_message(self, format, *args):
        # No code, cookies, or Authorization headers in logs.
        print('runner', self.command, self.path, flush=True)

if __name__ == '__main__':
    ThreadingHTTPServer(('0.0.0.0', 4000), Handler).serve_forever()
