<?php
try {
    $db = new PDO('sqlite:database.sqlite');
    $db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

    // 1. Ajouter des catégories
    $db->exec("INSERT OR IGNORE INTO categories (name, slug) VALUES 
        ('Technologie', 'tech'),
        ('Développement PHP', 'php'),
        ('Actualités', 'news')");
    
    // 2. Créer une page statique
    $db->exec("INSERT INTO posts (title, content, status, type) VALUES 
        ('À propos de NCMS', '<p>NCMS est un moteur de contenu innovant basé sur Nhtml.</p><p>Sa force réside dans sa réactivité native et sa simplicité.</p>', 'published', 'page'),
        ('Contact', '<p>Pour nous contacter, envoyez un email à contact@nhtml.test</p>', 'published', 'page')");

    // 3. Assigner des catégories à certains articles
    $db->exec("UPDATE posts SET category_id = 1 WHERE title LIKE '%futur%'");
    $db->exec("UPDATE posts SET category_id = 2 WHERE title LIKE '%PHP%'");

    echo "Données de démonstration v0.4 injectées.\n";

} catch (Exception $e) {
    echo "ERREUR : " . $e->getMessage() . "\n";
}
