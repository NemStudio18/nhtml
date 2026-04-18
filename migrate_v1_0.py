import sqlite3

db = sqlite3.connect('NCMS/database.sqlite')
cursor = db.cursor()

try:
    # 1. Menu links table
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS menu_links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            label TEXT NOT NULL,
            url TEXT NOT NULL,
            position INTEGER DEFAULT 0,
            parent_id INTEGER DEFAULT NULL
        )
    ''')
    
    # 2. Forms table
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS forms (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            fields TEXT NOT NULL
        )
    ''')
    
    # 3. Form submissions table
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS form_submissions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            form_id INTEGER NOT NULL,
            data TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
    ''')
    
    # 4. Add form_id to posts
    cursor.execute('ALTER TABLE posts ADD COLUMN form_id INTEGER DEFAULT NULL')
    
    # 5. Seed initial menu if empty
    res = cursor.execute("SELECT COUNT(*) FROM menu_links").fetchone()
    if res[0] == 0:
        cursor.execute("INSERT INTO menu_links (label, url, position) VALUES ('Accueil', '/', 1)")
        print("Seeded initial menu with 'Accueil'.")

    db.commit()
    print("Database migration v1.0 completed successfully.")
except sqlite3.OperationalError as e:
    if "duplicate column name: form_id" in str(e):
        print("Column form_id already exists.")
    else:
        print(f"OperationalError: {e}")
except Exception as e:
    print(f"Error: {e}")
finally:
    db.close()
