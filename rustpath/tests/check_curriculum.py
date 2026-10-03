"""Only execute trusted bundled reference code, never learner submissions."""
import json, os, shutil, subprocess, tempfile
from pathlib import Path
root=Path(__file__).resolve().parents[1]
course=json.loads((root/'backend/course.json').read_text())
rustc=os.environ.get('RUSTC') or shutil.which('rustc') or '/data/.cargo/bin/rustc'
results=[]
with tempfile.TemporaryDirectory() as temp:
 for lesson in course['lessons']:
  if lesson['kind']=='quiz':
   q=lesson['quiz']; assert 0<=q['correct']<len(q['options']) and q['explanation'];results.append({'lesson':lesson['id'],'kind':'quiz','status':'valid answer contract'});continue
  for kind in ['solution','starter']:
   source=Path(temp)/'main.rs';binary=Path(temp)/'tests';source.write_text(lesson[kind]+'\n'+lesson['tests'])
   compile=subprocess.run([rustc,'--edition=2024','--test',str(source),'-o',str(binary)],capture_output=True,text=True,timeout=30)
   run=subprocess.run([str(binary),'--test-threads=1'],capture_output=True,text=True,timeout=15) if compile.returncode==0 else None
   passed=compile.returncode==0 and run.returncode==0
   if passed!=(kind=='solution'): raise RuntimeError(f'{lesson["id"]} {kind}: unexpected result\n{compile.stderr}\n{run.stdout if run else ""}')
  results.append({'lesson':lesson['id'],'kind':'code','solution':'passed','starter':'fails as expected','tests':len(lesson['test_labels'])})
(root/'docs/curriculum-qa.json').write_text(json.dumps(results,ensure_ascii=False,indent=2))
print(f'{sum(x["kind"]=="code" for x in results)} reference solutions PASS; {sum(x.get("tests",0) for x in results)} cases; {sum(x["kind"]=="quiz" for x in results)} quiz contracts')
