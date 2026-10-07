import subprocess,sys,json
from pathlib import Path
H=Path(__file__).resolve().parent
commands=[
("final_check_001",["check","--all-targets"]),
("final_authoring_001",["test","--lib","authoring"]),
("final_value_expr_001",["test","--lib","value_expr"]),
("final_all_001",["test","--all-targets"]),
("final_feature_001",["test","--all-targets","--features","test-fixtures"]),
("final_event_check_001",["check","--all-targets","--features","event_log_checks"]),
("final_release_001",["build","--release","--bins"]),
("final_clippy_001",["clippy","--all-targets"]),
]
for name,args in commands:
    cmd=["cargo",args[0],"-p","game_core",*args[1:],"--offline","--locked"]
    if args[0]=="clippy":cmd += ["--","-A","clippy::never_loop","-A","clippy::eq_op"]
    result=subprocess.run([sys.executable,str(H/"run.py"),name,"--",*cmd])
    if result.returncode:raise SystemExit(result.returncode)
subprocess.run([sys.executable,str(H/"run.py"),"final_format_001","--","cargo","fmt","--check"])
