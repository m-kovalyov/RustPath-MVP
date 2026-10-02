"""HTTP contract and sandbox configuration tests with Docker mocked (no isolation claim)."""
import importlib.util, json, sys, threading, unittest, urllib.request, urllib.error
from pathlib import Path
from unittest.mock import Mock, patch
import os
os.environ['RUNNER_SECRET']='unit-test-secret'
client=Mock()
with patch('docker.from_env', return_value=client):
    spec=importlib.util.spec_from_file_location('runner',Path(__file__).resolve().parents[1]/'runner/server.py')
    runner=importlib.util.module_from_spec(spec); spec.loader.exec_module(runner)
server=runner.ThreadingHTTPServer(('127.0.0.1',0),runner.Handler)
threading.Thread(target=server.serve_forever,daemon=True).start()

class RunnerTests(unittest.TestCase):
    def setUp(self):
        client.reset_mock()
        self.container=Mock()
        self.cleanup=threading.Event()
        self.container.remove.side_effect=lambda **kwargs: self.cleanup.set()
        self.container.wait.return_value={'StatusCode':0}
        self.container.logs.return_value=b'test result: ok. 1 passed; 0 failed;'
        client.containers.create.side_effect=None
        client.containers.create.return_value=self.container
    def post(self,payload,token='unit-test-secret'):
        req=urllib.request.Request(f'http://127.0.0.1:{server.server_port}/evaluate',data=json.dumps(payload).encode(),headers={'Authorization':'Bearer '+token,'Content-Type':'application/json'})
        try:
            with urllib.request.urlopen(req) as response: return response.status,json.load(response)
        except urllib.error.HTTPError as response: return response.code,json.load(response)
        finally:
            if client.containers.create.called:
                self.cleanup.wait(timeout=1)
    def test_requires_authentication(self):
        self.assertEqual(self.post({'source':'x'},token='bad')[0],401)
        client.containers.create.assert_not_called()
    def test_sets_all_resource_and_isolation_options(self):
        status,result=self.post({'source':'trusted-test-source'})
        self.assertEqual(status,200); self.assertTrue(result['passed'])
        options=client.containers.create.call_args.kwargs
        self.assertEqual(options['network_mode'],'none'); self.assertTrue(options['read_only'])
        self.assertEqual(options['user'],'65534:65534'); self.assertEqual(options['cap_drop'],['ALL'])
        self.assertEqual(options['pids_limit'],64); self.assertEqual(options['mem_limit'],'512m')
        self.assertIn('no-new-privileges:true',options['security_opt'])
        self.container.remove.assert_called_once_with(force=True)
    def test_exit_zero_without_harness_is_not_a_pass(self):
        self.container.logs.return_value=b''
        self.assertFalse(self.post({'source':'x'})[1]['passed'])
    def test_failed_tests_do_not_pass(self):
        self.container.wait.return_value={'StatusCode':101}
        self.assertFalse(self.post({'source':'x'})[1]['passed'])
    def test_timeout_kills_and_removes(self):
        self.container.wait.side_effect=TimeoutError('test timeout')
        self.assertFalse(self.post({'source':'x'})[1]['passed'])
        self.container.kill.assert_called_once(); self.container.remove.assert_called_once_with(force=True)
    def test_start_failure_still_removes_container(self):
        self.container.start.side_effect=runner.DockerException('test infrastructure failure')
        self.assertEqual(self.post({'source':'x'})[0],503)
        self.container.remove.assert_called_once_with(force=True)
    def test_invalid_source_rejected(self):
        self.assertEqual(self.post({'source':42})[0],400)
        self.assertEqual(self.post({})[0],400)
        client.containers.create.assert_not_called()

if __name__=='__main__':
    try: unittest.main()
    finally: server.shutdown()
