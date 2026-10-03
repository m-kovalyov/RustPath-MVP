"""Only run against your LOCAL Docker deployment, never a production site."""
import http.cookiejar, json, urllib.request, urllib.error, sys, time
from pathlib import Path
base=sys.argv[1] if len(sys.argv)>1 else 'http://localhost:8080'
jar=http.cookiejar.CookieJar();http=urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
course=json.loads((Path(__file__).resolve().parents[1]/'backend/course.json').read_text())
def call(method,path,data=None):
 req=urllib.request.Request(base+'/api'+path,method=method,headers={'Content-Type':'application/json','Origin':base},data=json.dumps(data).encode() if data else None)
 try:
  with http.open(req,timeout=45) as res:return res.status,json.load(res) if res.status!=204 else None
 except urllib.error.HTTPError as res:return res.code,json.load(res)
assert call('POST','/session')[0]==200
assert call('GET','/lessons/arithmetic')[0]==403
assert call('GET','/lessons/adv-min')[0]==200
for lesson in course['lessons']:
 quiz=lesson['kind']=='quiz';endpoint='answer' if quiz else 'submit';payload={'option':lesson['quiz']['correct']} if quiz else {'code':lesson['solution']}
 status,value=call('POST',f'/lessons/{lesson["id"]}/{endpoint}',payload)
 assert status==200 and value['evaluation']['passed'],(lesson['id'],status,value)
 print(lesson['id'],'PASS')
 time.sleep(6.2)
status,value=call('GET','/progress');assert status==200 and value['xp']==sum(l['xp'] for l in course['lessons']) and len(value['completed'])==len(course['lessons'])
assert call('PUT','/lessons/intro-let/bookmark')[0]==204
assert 'intro-let' in call('GET','/bookmarks')[1]
assert call('DELETE','/lessons/intro-let/bookmark')[0]==204
print(f'Full local smoke: PASS ({len(course["lessons"])} levels, {value["xp"]} XP)')
