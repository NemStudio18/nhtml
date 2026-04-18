<?php
$db = new PDO('sqlite:database.sqlite');
$db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

// Table des articles
$db->exec("CREATE TABLE IF NOT EXISTS posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    status TEXT DEFAULT 'published',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
)");

// Table des commentaires
$db->exec("CREATE TABLE IF NOT EXISTS comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id INTEGER,
    author TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(post_id) REFERENCES posts(id) ON DELETE CASCADE
)");

// Insertion de données de test
$count = $db->query("SELECT COUNT(*) FROM posts")->fetchColumn();
if ($count == 0) {
    $db->exec("INSERT INTO posts (title, content, status) VALUES 
        ('Bienvenue sur NCMS', 'Ceci est votre premier article propulsé par Nhtml et PHP.', 'published'),
        ('Le futur du Web', 'Nhtml rend le développement web à nouveau amusant.', 'draft'),
        ('SOLID en PHP', 'Découvrez comment structurer votre code proprement.', 'published')
    ");
    
    $postId = $db->lastInsertId();
    $db->exec("INSERT INTO comments (post_id, author, content) VALUES 
        (1, 'Alice', 'Super projet !'),
        (1, 'Bob', 'Nhtml est impressionnant.')
    ");
}

echo "Base de données initialisée avec succès.\n";
