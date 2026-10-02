"""Run against a LOCAL Docker deployment. Never send test fixtures to a production service."""
import http.cookiejar, json, urllib.request, urllib.error, sys
from pathlib import Path
base=sys.argv[1] if len(sys.argv)>1 else 'http://localhost:8080'
jar=http.cookiejar.CookieJar(); http=urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
course=json.loads((Path(__file__).resolve().parents[1]/'backend/course.json').read_text())
def call(method,path,data=None):
    req=urllib.request.Request(base+'/api'+path,method=method,headers={'Content-Type':'application/json','Origin':base},data=json.dumps(data).encode() if data else None)
    try:
        with http.open(req,timeout=45) as res: return res.status,json.load(res) if res.status!=204 else None
    except urllib.error.HTTPError as res: return res.code,json.load(res)
assert call('POST','/session')[0]==200
assert call('GET','/lessons/arithmetic')[0]==403
for lesson in course['lessons']:
    status,value=call('POST',f'/lessons/{lesson["id"]}/submit',{'code':lesson['solution']})
    assert status==200 and value['evaluation']['passed'],(lesson['id'],status,value)
    print(lesson['id'],'PASS')
    # API limits each learner to 10 submissions/minute.
    import time; time.sleep(6.2)
status,value=call('GET','/progress'); assert status==200 and value['xp']==800 and len(value['completed'])==16
print('Full local deployment smoke: PASS (16 levels, 800 XP)')
