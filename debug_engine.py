from nhtml_engine.parser import NhtmlParser
import traceback

with open('NCMS/templates/admin.nhtml', 'r', encoding='utf-8') as f:
    source = f.read()

parser = NhtmlParser()
try:
    parser.parse(source)
    print("Success!")
except Exception:
    traceback.print_exc()
