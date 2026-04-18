import sqlite3
import json

db = sqlite3.connect('NCMS/database.sqlite')
db.row_factory = sqlite3.Row
cursor = db.cursor()

tables = cursor.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall()
for table in tables:
    name = table['name']
    print(f"--- Table: {name} ---")
    info = cursor.execute(f"PRAGMA table_info({name})").fetchall()
    for col in info:
        print(dict(col))
    print()

db.close()
