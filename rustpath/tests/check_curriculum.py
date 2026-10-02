"""Compile and execute only trusted bundled curriculum, never user-supplied code."""
import json, os, subprocess, tempfile
from pathlib import Path
root=Path(__file__).resolve().parents[1]
course=json.loads((root/'backend/course.json').read_text())
rustc=os.environ.get('RUSTC',str(Path.home()/'.cargo/bin/rustc'))
results=[]
with tempfile.TemporaryDirectory() as temp:
    for lesson in course['lessons']:
        for kind in ['solution','starter']:
            source=Path(temp)/'main.rs'; binary=Path(temp)/'tests'
            source.write_text(lesson[kind]+'\n'+lesson['tests'])
            compile=subprocess.run([rustc,'--edition=2024','--test',str(source),'-o',str(binary)],capture_output=True,text=True,timeout=30)
            run=subprocess.run([str(binary)],capture_output=True,text=True,timeout=10) if compile.returncode==0 else None
            passed=compile.returncode==0 and run.returncode==0
            expected=kind=='solution'
            if passed!=expected:
                raise RuntimeError(f'{lesson["id"]} {kind}: unexpected result\n{compile.stderr}\n{run.stdout if run else ""}')
        results.append({'lesson':lesson['id'],'solution':'passed','starter':'fails as expected','tests':len(lesson['test_labels'])})
(root/'docs/curriculum-qa.json').write_text(json.dumps(results,ensure_ascii=False,indent=2))
print(f'{len(results)} reference solutions PASS; {len(results)} starters fail as expected; {sum(x["tests"] for x in results)} cases')
