<?php
try {
    $db = new PDO('sqlite:database.sqlite');
    $db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

    echo "Démarrage de la migration v0.4...\n";

    // 1. Création de la table catégories
    $db->exec("CREATE TABLE IF NOT EXISTS categories (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        slug TEXT NOT NULL UNIQUE,
        parent_id INTEGER NULL,
        FOREIGN KEY(parent_id) REFERENCES categories(id)
    )");
    echo "- Table 'categories' prête.\n";

    // 2. Mise à jour de la table posts (si colonnes manquantes)
    $columns = $db->query("PRAGMA table_info(posts)")->fetchAll(PDO::FETCH_COLUMN, 1);
    
    if (!in_array('type', $columns)) {
        $db->exec("ALTER TABLE posts ADD COLUMN type TEXT DEFAULT 'post'");
        echo "- Colonne 'type' ajoutée à 'posts'.\n";
    }

    if (!in_array('category_id', $columns)) {
        $db->exec("ALTER TABLE posts ADD COLUMN category_id INTEGER NULL");
        echo "- Colonne 'category_id' ajoutée à 'posts'.\n";
    }

    // 3. Insertion d'une catégorie par défaut
    $catCount = $db->query("SELECT COUNT(*) FROM categories")->fetchColumn();
    if ($catCount == 0) {
        $db->exec("INSERT INTO categories (name, slug) VALUES ('Général', 'general')");
        echo "- Catégorie par défaut créée.\n";
    }

    echo "Migration v0.4 terminée avec succès.\n";

} catch (Exception $e) {
    echo "ERREUR DE MIGRATION : " . $e->getMessage() . "\n";
}
